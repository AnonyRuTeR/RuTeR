use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;

static INDEX_CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<SymbolReachabilityIndex>>>> =
    OnceLock::new();

/// Shared lightweight symbol reachability index for R1 heuristic.
#[derive(Debug, Clone, Default)]
pub struct SymbolReachabilityIndex {
    symbol_file_counts: HashMap<String, usize>,
    crate_visible_modules: HashSet<String>,
}

impl SymbolReachabilityIndex {
    /// Build or reuse cached index for one crate root.
    pub fn shared(crate_root: &Path) -> Result<Arc<Self>> {
        let canonical = crate_root
            .canonicalize()
            .unwrap_or_else(|_| crate_root.to_path_buf());
        let cache = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        if let Some(hit) = cache
            .lock()
            .expect("symbol reachability cache poisoned")
            .get(&canonical)
            .cloned()
        {
            return Ok(hit);
        }

        let built = Arc::new(Self::build(&canonical)?);
        let mut guard = cache.lock().expect("symbol reachability cache poisoned");
        let entry = guard
            .entry(canonical)
            .or_insert_with(|| Arc::clone(&built))
            .clone();
        Ok(entry)
    }

    /// Returns true when the symbol is exported by exactly one source file.
    pub fn is_uniquely_reachable(&self, symbol: &str) -> bool {
        self.symbol_file_counts.get(symbol).copied().unwrap_or(0) == 1
    }

    /// Returns true when module is declared at crate root (`src/lib.rs` or `src/main.rs`).
    pub fn is_crate_visible_module(&self, module: &str) -> bool {
        self.crate_visible_modules.contains(module)
    }

    fn build(crate_root: &Path) -> Result<Self> {
        let src_root = crate_root.join("src");
        if !src_root.exists() {
            return Ok(Self::default());
        }

        let pub_item_re = Regex::new(
            r"(?m)^\s*pub(?:\s*\([^)]*\))?\s+(?:async\s+)?(?:fn|struct|enum|trait|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)\b",
        )
        .expect("pub item regex must be valid");
        let pub_use_re = Regex::new(
            r"(?m)^\s*pub\s+use\s+[^;]*::([A-Za-z_][A-Za-z0-9_]*)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;",
        )
        .expect("pub use regex must be valid");
        let root_mod_re = Regex::new(
            r"(?m)^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\b",
        )
        .expect("root module regex must be valid");

        let mut symbol_file_counts: HashMap<String, usize> = HashMap::new();
        let mut crate_visible_modules: HashSet<String> = HashSet::new();

        for entry in WalkDir::new(&src_root) {
            let Ok(entry) = entry else {
                continue;
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }

            let Ok(source) = fs::read_to_string(path) else {
                continue;
            };
            let file_symbols = collect_symbols(&source, &pub_item_re, &pub_use_re);
            for symbol in file_symbols {
                *symbol_file_counts.entry(symbol).or_insert(0) += 1;
            }
        }

        collect_crate_root_modules(&src_root.join("lib.rs"), &root_mod_re, &mut crate_visible_modules);
        collect_crate_root_modules(
            &src_root.join("main.rs"),
            &root_mod_re,
            &mut crate_visible_modules,
        );

        Ok(Self {
            symbol_file_counts,
            crate_visible_modules,
        })
    }
}

fn collect_symbols(source: &str, pub_item_re: &Regex, pub_use_re: &Regex) -> HashSet<String> {
    let mut out = HashSet::new();

    for captures in pub_item_re.captures_iter(source) {
        if let Some(name) = captures.get(1) {
            out.insert(name.as_str().to_string());
        }
    }

    for captures in pub_use_re.captures_iter(source) {
        if let Some(name) = captures.get(1) {
            out.insert(name.as_str().to_string());
        }
        if let Some(alias) = captures.get(2) {
            out.insert(alias.as_str().to_string());
        }
    }

    out
}

fn collect_crate_root_modules(path: &Path, mod_re: &Regex, out: &mut HashSet<String>) {
    let Ok(source) = fs::read_to_string(path) else {
        return;
    };
    for captures in mod_re.captures_iter(&source) {
        if let Some(name) = captures.get(1) {
            out.insert(name.as_str().to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;

    #[test]
    fn reachability_index_reports_unique_pub_symbol() {
        let dir = tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn format_duration() -> usize { 1 }\n",
        )
        .unwrap();

        let idx = SymbolReachabilityIndex::build(dir.path()).unwrap();
        assert!(idx.is_uniquely_reachable("format_duration"));
    }

    #[test]
    fn reachability_index_reports_ambiguous_pub_symbol() {
        let dir = tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/a.rs"),
            "pub fn format_duration() -> usize { 1 }\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("src/b.rs"),
            "pub fn format_duration() -> usize { 2 }\n",
        )
        .unwrap();

        let idx = SymbolReachabilityIndex::build(dir.path()).unwrap();
        assert!(!idx.is_uniquely_reachable("format_duration"));
    }

    #[test]
    fn reachability_index_recognizes_pub_use_reexport() {
        let dir = tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/lib.rs"),
            "mod inner;\npub use crate::inner::Timestamp;\n",
        )
        .unwrap();
        fs::write(dir.path().join("src/inner.rs"), "struct Timestamp;\n").unwrap();

        let idx = SymbolReachabilityIndex::build(dir.path()).unwrap();
        assert!(idx.is_uniquely_reachable("Timestamp"));
    }

    #[test]
    fn reachability_index_recognizes_private_crate_root_module() {
        let dir = tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/lib.rs"),
            "mod wrapper;\n#[cfg(test)] mod tests {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("src/wrapper.rs"), "pub struct Duration;\n").unwrap();

        let idx = SymbolReachabilityIndex::build(dir.path()).unwrap();
        assert!(idx.is_crate_visible_module("wrapper"));
    }

    #[test]
    fn reachability_index_ignores_nested_non_root_module() {
        let dir = tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "mod a;\n").unwrap();
        fs::write(dir.path().join("src/a.rs"), "mod inner;\n").unwrap();
        fs::write(dir.path().join("src/inner.rs"), "pub struct Marker;\n").unwrap();

        let idx = SymbolReachabilityIndex::build(dir.path()).unwrap();
        assert!(!idx.is_crate_visible_module("inner"));
    }
}
