use std::collections::BTreeSet;

use quote::ToTokens;
use syn::visit::Visit;
use syn::{File, Item, Type};

use super::FunctionDiagnostic;

pub(super) fn collect_direct_symbols(function_text: &str) -> BTreeSet<String> {
    let item = syn::parse_str::<syn::ItemFn>(function_text);
    let Ok(item) = item else {
        return BTreeSet::new();
    };

    let mut collector = SymbolCollector::default();
    collector.visit_item_fn(&item);
    collector.symbols
}

#[derive(Default)]
struct SymbolCollector {
    symbols: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for SymbolCollector {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if let Some(head) = node.path.segments.first() {
            self.symbols.insert(head.ident.to_string());
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        // For paths like `foo::Parser { ... }`, keep type tail as primary symbol
        // and module head as fallback to preserve existing recall behavior.
        if let Some(tail) = node.path.segments.last() {
            self.symbols.insert(tail.ident.to_string());
        }
        if let (Some(head), Some(tail)) = (node.path.segments.first(), node.path.segments.last())
            && head.ident != tail.ident
        {
            self.symbols.insert(head.ident.to_string());
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if let Some(head) = node.path.segments.first() {
            self.symbols.insert(head.ident.to_string());
        }
        syn::visit::visit_type_path(self, node);
    }
}

pub(super) fn collect_related_imports(syntax: &File, symbols: &BTreeSet<String>) -> Vec<String> {
    let mut out = collect_items_recursive(&syntax.items)
        .into_iter()
        .filter_map(|item| match item {
            Item::Use(item_use) => Some(item_use.to_token_stream().to_string()),
            _ => None,
        })
        .filter(|text| text_matches_any_symbol(text, symbols))
        .collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

pub(super) fn collect_related_type_defs(syntax: &File, symbols: &BTreeSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in collect_items_recursive(&syntax.items) {
        match item {
            Item::Struct(item_struct) if symbols.contains(&item_struct.ident.to_string()) => {
                out.push(item_struct.to_token_stream().to_string());
            }
            Item::Enum(item_enum) if symbols.contains(&item_enum.ident.to_string()) => {
                out.push(item_enum.to_token_stream().to_string());
            }
            Item::Trait(item_trait) if symbols.contains(&item_trait.ident.to_string()) => {
                out.push(item_trait.to_token_stream().to_string());
            }
            Item::Type(item_type) if symbols.contains(&item_type.ident.to_string()) => {
                out.push(item_type.to_token_stream().to_string());
            }
            _ => {}
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(super) fn collect_related_impl_blocks(
    syntax: &File,
    symbols: &BTreeSet<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for item in collect_items_recursive(&syntax.items) {
        if let Item::Impl(item_impl) = item
            && let Some(self_ty_ident) = impl_self_ty_ident(&item_impl.self_ty)
            && symbols.contains(&self_ty_ident)
        {
            out.push(item_impl.to_token_stream().to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(super) fn collect_related_fn_defs(
    syntax: &File,
    symbol_hints: &BTreeSet<String>,
) -> Vec<String> {
    if symbol_hints.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for item in collect_items_recursive(&syntax.items) {
        if let Item::Fn(item_fn) = item
            && symbol_hints.contains(&item_fn.sig.ident.to_string())
        {
            out.push(format_item_pretty(Item::Fn(item_fn.clone())));
        }
    }
    out.sort();
    out.dedup();
    out
}

fn format_item_pretty(item: Item) -> String {
    let file = File {
        shebang: None,
        attrs: Vec::new(),
        items: vec![item],
    };
    prettyplease::unparse(&file)
}

fn impl_self_ty_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|seg| seg.ident.to_string()),
        _ => None,
    }
}

fn collect_items_recursive<'a>(items: &'a [Item]) -> Vec<&'a Item> {
    let mut out = Vec::new();
    for item in items {
        out.push(item);
        if let Item::Mod(item_mod) = item
            && let Some((_, sub_items)) = &item_mod.content
        {
            out.extend(collect_items_recursive(sub_items));
        }
    }
    out
}

fn text_matches_any_symbol(text: &str, symbols: &BTreeSet<String>) -> bool {
    let tokens = text
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|part| !part.is_empty())
        .collect::<BTreeSet<_>>();

    symbols.iter().any(|sym| tokens.contains(sym.as_str()))
}

pub(super) fn collect_diagnostic_symbol_hints(
    diagnostics: &[FunctionDiagnostic],
) -> BTreeSet<String> {
    let mut hints = BTreeSet::new();
    for diag in diagnostics {
        for symbol in extract_backticked_identifiers(&diag.message) {
            hints.insert(symbol);
        }
        if let Some(label) = diag.label.as_deref() {
            for symbol in extract_backticked_identifiers(label) {
                hints.insert(symbol);
            }
        }
    }
    hints
}

fn extract_backticked_identifiers(text: &str) -> Vec<String> {
    text.split('`')
        .enumerate()
        .filter_map(|(idx, chunk)| {
            if idx % 2 == 1 && looks_like_ident(chunk) {
                Some(chunk.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn looks_like_ident(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
