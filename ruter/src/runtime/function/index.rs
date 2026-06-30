use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use quote::ToTokens;
use ruter::core::{Diagnostic, FixAction, Severity, SpanInfo};
pub use ruter::core::{FunctionDiagnostic, TestFunction};
use syn::spanned::Spanned;
use syn::{Attribute, Item, ItemFn, ItemMod};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeRange {
    pub relative_file: PathBuf,
    pub file_path: PathBuf,
    pub module_path: Vec<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionIndex {
    functions: Vec<TestFunction>,
    file_to_indexes: HashMap<PathBuf, Vec<usize>>,
    test_modules: Vec<TestModuleScope>,
    file_to_test_module_indexes: HashMap<PathBuf, Vec<usize>>,
    id_to_index: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
struct CollectContext {
    module_path: Vec<String>,
    in_cfg_test: bool,
    file_in_tests_dir: bool,
}

#[derive(Debug, Clone)]
struct CollectedFn {
    module_path: Vec<String>,
    fn_name: String,
    line_start: usize,
    line_end: usize,
}

#[derive(Debug, Clone)]
struct CollectedTestModule {
    module_path: Vec<String>,
    line_start: usize,
    line_end: usize,
}

#[derive(Debug, Clone)]
struct TestModuleScope {
    relative_file: PathBuf,
    file_path: PathBuf,
    module_path: Vec<String>,
    line_start: usize,
    line_end: usize,
    byte_start: usize,
    byte_end: usize,
}

impl FunctionIndex {
    pub fn build(crate_path: &Path) -> Result<Self> {
        let crate_path = crate_path
            .canonicalize()
            .unwrap_or_else(|_| crate_path.to_path_buf());
        let mut functions = Vec::new();
        let mut test_modules = Vec::new();

        for entry in WalkDir::new(&crate_path) {
            let entry = entry?;
            let path = entry.path();

            if !entry.file_type().is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            if should_skip_path(path) {
                continue;
            }

            let source = fs::read_to_string(path)
                .with_context(|| format!("failed to read rust source {}", path.display()))?;
            let syntax = syn::parse_file(&source)
                .with_context(|| format!("failed to parse rust source {}", path.display()))?;
            let relative = path.strip_prefix(&crate_path).unwrap_or(path).to_path_buf();
            let mut collected = Vec::new();
            let mut collected_modules = Vec::new();
            let file_in_tests_dir = relative.components().any(|c| c.as_os_str() == "tests");
            collect_test_functions(
                &syntax.items,
                &CollectContext {
                    module_path: Vec::new(),
                    in_cfg_test: false,
                    file_in_tests_dir,
                },
                &mut collected,
                &mut collected_modules,
            );

            if file_in_tests_dir {
                collected_modules.push(CollectedTestModule {
                    module_path: Vec::new(),
                    line_start: 1,
                    line_end: source.lines().count().max(1),
                });
            }

            for item in collected {
                let byte_start = line_col_to_byte(&source, item.line_start, 0);
                let byte_end = line_end_to_byte(&source, item.line_end);
                if byte_start >= byte_end {
                    continue;
                }

                let id = format!(
                    "{}::{}::{}:{}:{}",
                    relative.display(),
                    item.module_path.join("::"),
                    item.fn_name,
                    item.line_start,
                    item.line_end
                );
                functions.push(TestFunction {
                    id,
                    relative_file: relative.clone(),
                    file_path: path.to_path_buf(),
                    module_path: item.module_path,
                    fn_name: item.fn_name,
                    byte_start,
                    byte_end,
                    line_start: item.line_start,
                    line_end: item.line_end,
                });
            }

            for module in collected_modules {
                let byte_start = line_col_to_byte(&source, module.line_start, 0);
                let byte_end = line_end_to_byte(&source, module.line_end);
                if byte_start >= byte_end {
                    continue;
                }
                test_modules.push(TestModuleScope {
                    relative_file: relative.clone(),
                    file_path: path.to_path_buf(),
                    module_path: module.module_path,
                    line_start: module.line_start,
                    line_end: module.line_end,
                    byte_start,
                    byte_end,
                });
            }
        }

        let mut file_to_indexes: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        let mut file_to_test_module_indexes: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        let mut id_to_index = HashMap::new();
        for (idx, function) in functions.iter().enumerate() {
            file_to_indexes
                .entry(function.relative_file.clone())
                .or_default()
                .push(idx);
            id_to_index.insert(function.id.clone(), idx);
        }
        for (idx, module) in test_modules.iter().enumerate() {
            file_to_test_module_indexes
                .entry(module.relative_file.clone())
                .or_default()
                .push(idx);
        }
        for indexes in file_to_indexes.values_mut() {
            indexes.sort_by_key(|idx| functions[*idx].byte_start);
        }
        for indexes in file_to_test_module_indexes.values_mut() {
            indexes.sort_by(|lhs, rhs| {
                test_modules[*rhs]
                    .module_path
                    .len()
                    .cmp(&test_modules[*lhs].module_path.len())
                    .then_with(|| {
                        test_modules[*lhs]
                            .line_start
                            .cmp(&test_modules[*rhs].line_start)
                    })
            });
        }

        Ok(Self {
            functions,
            file_to_indexes,
            test_modules,
            file_to_test_module_indexes,
            id_to_index,
        })
    }

    #[cfg(test)]
    pub fn functions(&self) -> &[TestFunction] {
        &self.functions
    }

    pub fn get(&self, function_id: &str) -> Option<&TestFunction> {
        self.id_to_index
            .get(function_id)
            .and_then(|idx| self.functions.get(*idx))
    }

    /// Resolve the enclosing test module scope for a function id.
    ///
    /// Falls back to the function range itself when no enclosing test module
    /// can be determined.
    pub fn enclosing_test_module_scope_for_function_id(
        &self,
        function_id: &str,
    ) -> Option<ScopeRange> {
        let function = self.get(function_id)?;
        self.enclosing_test_module_scope(function)
    }

    /// Resolve the enclosing test module scope for one function.
    ///
    /// Returns the function range as fallback when no module-level scope exists.
    pub fn enclosing_test_module_scope(&self, function: &TestFunction) -> Option<ScopeRange> {
        if let Some(scope) = self.find_enclosing_test_module(
            &function.relative_file,
            function.line_start,
            function.line_end,
        ) {
            return Some(ScopeRange {
                relative_file: scope.relative_file.clone(),
                file_path: scope.file_path.clone(),
                module_path: scope.module_path.clone(),
                line_start: scope.line_start,
                line_end: scope.line_end,
                byte_start: scope.byte_start,
                byte_end: scope.byte_end,
            });
        }

        Some(ScopeRange {
            relative_file: function.relative_file.clone(),
            file_path: function.file_path.clone(),
            module_path: function.module_path.clone(),
            line_start: function.line_start,
            line_end: function.line_end,
            byte_start: function.byte_start,
            byte_end: function.byte_end,
        })
    }

    /// Collect all test function ids that belong to the same enclosing test
    /// module as the target function.
    ///
    /// Returns a one-item set containing `function_id` when mapping fails.
    pub fn function_ids_in_same_test_module(&self, function_id: &str) -> BTreeSet<String> {
        let mut fallback = BTreeSet::new();
        fallback.insert(function_id.to_string());

        let Some(target) = self.get(function_id) else {
            return fallback;
        };
        let Some(scope) = self.enclosing_test_module_scope(target) else {
            return fallback;
        };

        let mut out = BTreeSet::new();
        for function in &self.functions {
            if function.relative_file != scope.relative_file {
                continue;
            }
            if function.line_start < scope.line_start || function.line_end > scope.line_end {
                continue;
            }
            if !function.module_path.starts_with(&scope.module_path) {
                continue;
            }
            out.insert(function.id.clone());
        }

        if out.is_empty() { fallback } else { out }
    }

    /// Find a function by stable identity fields.
    ///
    /// This helper is used when line ranges may drift after patching, while
    /// file/module/function name identity is still stable.
    pub fn find_by_identity(
        &self,
        relative_file: &Path,
        module_path: &[String],
        fn_name: &str,
    ) -> Option<&TestFunction> {
        self.functions.iter().find(|function| {
            function.relative_file == relative_file
                && function.module_path == module_path
                && function.fn_name == fn_name
        })
    }

    pub fn target_function_ids_for_errors(
        &self,
        diagnostics: &[Diagnostic],
        crate_path: &Path,
    ) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for diagnostic in diagnostics {
            if !matches!(diagnostic.severity, Severity::Error) {
                continue;
            }
            if let Some(span) = primary_span(diagnostic)
                && let Some(function) = self.function_for_span(span, crate_path)
            {
                out.insert(function.id.clone());
            }
        }
        out
    }

    pub fn error_diagnostics_by_function(
        &self,
        diagnostics: &[Diagnostic],
        crate_path: &Path,
    ) -> BTreeMap<String, Vec<FunctionDiagnostic>> {
        let mut out: BTreeMap<String, Vec<FunctionDiagnostic>> = BTreeMap::new();
        for diagnostic in diagnostics {
            if !matches!(diagnostic.severity, Severity::Error) {
                continue;
            }
            let code = diagnostic
                .code
                .as_ref()
                .map(|c| c.code.to_string())
                .unwrap_or_else(|| "NO_CODE".to_string());
            let evidence = FunctionDiagnostic {
                code,
                message: diagnostic.message.clone(),
                primary_span: primary_span(diagnostic).map(|span| {
                    format!(
                        "{}:{}:{}-{}:{}",
                        span.file_path.display(),
                        span.line_start,
                        span.col_start,
                        span.line_end,
                        span.col_end
                    )
                }),
                label: primary_span(diagnostic).and_then(|span| span.label.clone()),
                suggested_replacement: primary_span(diagnostic)
                    .and_then(|span| span.suggested_replacement.clone()),
                children_note_messages: collect_child_messages_by_level(
                    &diagnostic.children,
                    Severity::Note,
                ),
                children_help_messages: collect_child_messages_by_level(
                    &diagnostic.children,
                    Severity::Help,
                ),
                children_suggested_replacements: collect_child_suggested_replacements(
                    &diagnostic.children,
                ),
            };

            if let Some(span) = primary_span(diagnostic)
                && let Some(function) = self.function_for_span(span, crate_path)
            {
                out.entry(function.id.clone()).or_default().push(evidence);
                continue;
            }

            out.entry("__UNMAPPED_ERRORS__".to_string())
                .or_default()
                .push(evidence);
        }
        out
    }

    pub fn actions_grouped_by_function(
        &self,
        plan: &BTreeMap<PathBuf, Vec<FixAction>>,
        crate_path: &Path,
    ) -> BTreeMap<String, Vec<FixAction>> {
        let mut out: BTreeMap<String, Vec<FixAction>> = BTreeMap::new();

        for actions in plan.values() {
            for action in actions {
                if let Some(span) = action_span(action)
                    && let Some(function) = self.function_for_span(span, crate_path)
                {
                    out.entry(function.id.clone())
                        .or_default()
                        .push(action.clone());
                }
            }
        }

        out
    }

    pub fn function_for_span<'a>(
        &'a self,
        span: &SpanInfo,
        crate_path: &Path,
    ) -> Option<&'a TestFunction> {
        let relative = span
            .file_path
            .strip_prefix(crate_path)
            .unwrap_or(&span.file_path)
            .to_path_buf();
        let indexes = self.file_to_indexes.get(&relative)?;

        // Prefer line mapping first: byte offsets may drift after patch application.
        // Line range remains stable for most replace-only edits and gives more
        // consistent function attribution across candidate/union verification.
        for idx in indexes {
            let function = &self.functions[*idx];
            if span.line_start >= function.line_start && span.line_end <= function.line_end {
                return Some(function);
            }
        }

        for idx in indexes {
            let function = &self.functions[*idx];
            if span.byte_start >= function.byte_start && span.byte_end <= function.byte_end {
                return Some(function);
            }
        }

        self.nearest_test_context_function_for_span(&relative, indexes, span)
    }

    fn nearest_test_context_function_for_span<'a>(
        &'a self,
        relative: &Path,
        indexes: &[usize],
        span: &SpanInfo,
    ) -> Option<&'a TestFunction> {
        let scope = self.find_enclosing_test_module(relative, span.line_start, span.line_end)?;
        let mut best: Option<(usize, usize)> = None;

        for idx in indexes {
            let function = &self.functions[*idx];
            if !function.module_path.starts_with(&scope.module_path) {
                continue;
            }

            let line_distance = if span.line_end < function.line_start {
                function.line_start - span.line_end
            } else if span.line_start > function.line_end {
                span.line_start - function.line_end
            } else {
                0
            };

            let current = (line_distance, *idx);
            if best.map(|existing| current < existing).unwrap_or(true) {
                best = Some(current);
            }
        }

        best.map(|(_, idx)| &self.functions[idx])
    }

    fn find_enclosing_test_module(
        &self,
        relative: &Path,
        line_start: usize,
        line_end: usize,
    ) -> Option<&TestModuleScope> {
        let module_indexes = self.file_to_test_module_indexes.get(relative)?;
        module_indexes
            .iter()
            .filter_map(|idx| self.test_modules.get(*idx))
            .filter(|module| line_start >= module.line_start && line_end <= module.line_end)
            .max_by(|lhs, rhs| {
                lhs.module_path
                    .len()
                    .cmp(&rhs.module_path.len())
                    .then_with(|| {
                        let lhs_range = lhs.line_end.saturating_sub(lhs.line_start);
                        let rhs_range = rhs.line_end.saturating_sub(rhs.line_start);
                        rhs_range.cmp(&lhs_range)
                    })
            })
    }
}

fn collect_test_functions(
    items: &[Item],
    ctx: &CollectContext,
    out: &mut Vec<CollectedFn>,
    out_modules: &mut Vec<CollectedTestModule>,
) {
    for item in items {
        match item {
            Item::Fn(item_fn) => collect_fn(item_fn, ctx, out),
            Item::Mod(item_mod) => collect_mod(item_mod, ctx, out, out_modules),
            _ => {}
        }
    }
}

fn collect_mod(
    item_mod: &ItemMod,
    ctx: &CollectContext,
    out: &mut Vec<CollectedFn>,
    out_modules: &mut Vec<CollectedTestModule>,
) {
    let Some((_, items)) = &item_mod.content else {
        return;
    };

    let mut module_path = ctx.module_path.clone();
    module_path.push(item_mod.ident.to_string());
    let in_cfg_test = ctx.in_cfg_test || has_cfg_test_attr(&item_mod.attrs);
    let next_ctx = CollectContext {
        module_path,
        in_cfg_test,
        file_in_tests_dir: ctx.file_in_tests_dir,
    };
    if next_ctx.file_in_tests_dir || next_ctx.in_cfg_test {
        let span = item_mod.span();
        let start = span.start();
        let end = span.end();
        out_modules.push(CollectedTestModule {
            module_path: next_ctx.module_path.clone(),
            line_start: start.line.max(1),
            line_end: end.line.max(start.line.max(1)),
        });
    }
    collect_test_functions(items, &next_ctx, out, out_modules);
}

fn collect_fn(item_fn: &ItemFn, ctx: &CollectContext, out: &mut Vec<CollectedFn>) {
    let is_test = ctx.file_in_tests_dir || ctx.in_cfg_test || has_test_like_attr(&item_fn.attrs);
    if !is_test {
        return;
    }

    let span = item_fn.span();
    let start = span.start();
    let end = span.end();
    let line_start = start.line.max(1);
    let line_end = end.line.max(line_start);
    out.push(CollectedFn {
        module_path: ctx.module_path.clone(),
        fn_name: item_fn.sig.ident.to_string(),
        line_start,
        line_end,
    });
}

fn has_cfg_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg") && attr.to_token_stream().to_string().contains("test")
    })
}

fn has_test_like_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|segment| segment.ident == "test")
            .unwrap_or(false)
    })
}

fn collect_child_messages_by_level(children: &[Diagnostic], level: Severity) -> Vec<String> {
    let mut out = Vec::new();
    collect_child_messages_recursive(children, &level, &mut out);
    dedup_keep_order(out)
}

fn collect_child_messages_recursive(
    children: &[Diagnostic],
    level: &Severity,
    out: &mut Vec<String>,
) {
    for child in children {
        if &child.severity == level {
            let line = child.message.trim();
            if !line.is_empty() {
                out.push(line.to_string());
            }
        }
        collect_child_messages_recursive(&child.children, level, out);
    }
}

fn collect_child_suggested_replacements(children: &[Diagnostic]) -> Vec<String> {
    let mut out = Vec::new();
    collect_child_suggested_replacements_recursive(children, &mut out);
    dedup_keep_order(out)
}

fn collect_child_suggested_replacements_recursive(children: &[Diagnostic], out: &mut Vec<String>) {
    for child in children {
        for span in &child.span {
            if let Some(replacement) = span.suggested_replacement.as_ref() {
                let line = replacement.trim();
                if !line.is_empty() {
                    out.push(line.to_string());
                }
            }
        }
        collect_child_suggested_replacements_recursive(&child.children, out);
    }
}

fn dedup_keep_order(items: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for item in items {
        if seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
}

fn line_col_to_byte(source: &str, line: usize, col: usize) -> usize {
    let target_line = line.max(1);
    let mut current_line = 1usize;
    let mut byte_offset = 0usize;

    for line_content in source.split_inclusive('\n') {
        if current_line == target_line {
            let line_bytes = line_content.as_bytes();
            let col_clamped = col.min(line_bytes.len());
            return byte_offset + col_clamped;
        }
        byte_offset += line_content.len();
        current_line += 1;
    }

    source.len()
}

fn line_end_to_byte(source: &str, line: usize) -> usize {
    let mut current_line = 1usize;
    let mut byte_offset = 0usize;

    for line_content in source.split_inclusive('\n') {
        if current_line == line {
            return byte_offset + line_content.len();
        }
        byte_offset += line_content.len();
        current_line += 1;
    }

    source.len()
}

fn action_span(action: &FixAction) -> Option<&SpanInfo> {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => Some(span),
    }
}

fn primary_span(diagnostic: &Diagnostic) -> Option<&SpanInfo> {
    diagnostic
        .span
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| diagnostic.span.first())
}

fn should_skip_path(path: &Path) -> bool {
    path.components().any(|component| {
        let part = component.as_os_str();
        part == "target" || part == ".git" || part == ".ruter"
    })
}

#[cfg(test)]
mod tests;
