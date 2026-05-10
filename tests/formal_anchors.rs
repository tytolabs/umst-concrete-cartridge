// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! CI guardrail: every `pub fn`, `pub struct`, `pub enum`, and `pub trait` in `src/**/*.rs`
//! must declare a consistent formal documentation block (anchor URI, `formal_status`, and
//! status-specific metadata).

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use syn::visit::Visit;
use syn::{AttrStyle, ImplItem, Item, Meta, TraitItem, Visibility};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Struct,
    Enum,
    Trait,
    Fn,
}

struct Violation {
    pub file: &'static str,
    pub kind: Kind,
    pub symbol: String,
    pub detail: String,
}

struct AnchorVisitor {
    pub file_path: &'static str,
    pub items: Vec<Violation>,
}

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn extract_doc_line(attr: &syn::Attribute) -> Option<String> {
    if !attr.path().is_ident("doc") {
        return None;
    }
    let Meta::NameValue(nv) = &attr.meta else {
        return None;
    };
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.value
    else {
        return None;
    };
    Some(s.value())
}

fn collect_outer_doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| matches!(a.style, AttrStyle::Outer))
        .filter_map(extract_doc_line)
        .collect()
}

fn parse_tagged_doc_lines(lines: &[String]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for raw in lines {
        let t = raw.trim();
        let Some((key, rest)) = t.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if !matches!(
            key,
            "formal_anchor"
                | "formal_status"
                | "formal_axioms"
                | "formal_dataset"
                | "formal_citation"
                | "formal_envelope"
                | "formal_form"
                | "formal_anchor_rationale"
        ) {
            continue;
        }
        map.insert(key.to_string(), rest.trim().to_string());
    }
    map
}

fn validate_formal_block(map: &BTreeMap<String, String>, symbol: &str) -> Result<(), String> {
    let anchor = map
        .get("formal_anchor")
        .ok_or_else(|| format!("{symbol}: missing `/// formal_anchor:`"))?;
    let status = map
        .get("formal_status")
        .ok_or_else(|| format!("{symbol}: missing `/// formal_status:`"))?;

    if status == "Library" {
        return Err(format!(
            "{symbol}: `formal_status: Library` is retired — use NONE, Empirical, or Literature"
        ));
    }

    match status.as_str() {
        "Mechanised" | "Structural" | "Boundary" => {
            if !anchor.starts_with("lean://") {
                return Err(format!(
                    "{symbol}: `{status}` requires `formal_anchor: lean://...`, got `{anchor}`"
                ));
            }
        }
        "Empirical" => {
            if !anchor.starts_with("empirical://") {
                return Err(format!(
                    "{symbol}: Empirical requires `formal_anchor: empirical://...`, got `{anchor}`"
                ));
            }
            for k in ["formal_dataset", "formal_citation", "formal_envelope"] {
                if !map.contains_key(k) {
                    return Err(format!("{symbol}: Empirical requires `/// {k}:`"));
                }
            }
        }
        "Literature" => {
            if !anchor.starts_with("literature://") {
                return Err(format!(
                    "{symbol}: Literature requires `formal_anchor: literature://...`, got `{anchor}`"
                ));
            }
            for k in ["formal_citation", "formal_form"] {
                if !map.contains_key(k) {
                    return Err(format!("{symbol}: Literature requires `/// {k}:`"));
                }
            }
        }
        "NONE" => {
            if !anchor.contains("NONE") {
                return Err(format!(
                    "{symbol}: NONE status requires `formal_anchor: NONE`, got `{anchor}`"
                ));
            }
            if !map.contains_key("formal_anchor_rationale") {
                return Err(format!(
                    "{symbol}: NONE status requires `/// formal_anchor_rationale:`"
                ));
            }
        }
        other => {
            return Err(format!(
                "{symbol}: unknown `formal_status: {other}` (expected Mechanised, Structural, Boundary, Empirical, Literature, or NONE)"
            ));
        }
    }
    Ok(())
}

fn check_pub_fn_attrs(
    sym: String,
    attrs: &[syn::Attribute],
    file_path: &'static str,
    acc: &mut Vec<Violation>,
) {
    let lines = collect_outer_doc_lines(attrs);
    let map = parse_tagged_doc_lines(&lines);
    if let Err(detail) = validate_formal_block(&map, &sym) {
        acc.push(Violation {
            file: file_path,
            kind: Kind::Fn,
            symbol: sym,
            detail,
        });
    }
}

impl<'ast> Visit<'ast> for AnchorVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let file = self.file_path;
        match i {
            Item::Struct(s) if is_pub(&s.vis) => {
                let lines = collect_outer_doc_lines(&s.attrs);
                let map = parse_tagged_doc_lines(&lines);
                if let Err(detail) = validate_formal_block(&map, &s.ident.to_string()) {
                    self.items.push(Violation {
                        file,
                        kind: Kind::Struct,
                        symbol: s.ident.to_string(),
                        detail,
                    });
                }
            }
            Item::Enum(e) if is_pub(&e.vis) => {
                let lines = collect_outer_doc_lines(&e.attrs);
                let map = parse_tagged_doc_lines(&lines);
                if let Err(detail) = validate_formal_block(&map, &e.ident.to_string()) {
                    self.items.push(Violation {
                        file,
                        kind: Kind::Enum,
                        symbol: e.ident.to_string(),
                        detail,
                    });
                }
            }
            Item::Fn(f) if is_pub(&f.vis) => {
                check_pub_fn_attrs(f.sig.ident.to_string(), &f.attrs, file, &mut self.items);
            }
            Item::Impl(impl_block) => {
                for it in &impl_block.items {
                    if let ImplItem::Fn(m) = it {
                        if is_pub(&m.vis) {
                            check_pub_fn_attrs(
                                m.sig.ident.to_string(),
                                &m.attrs,
                                file,
                                &mut self.items,
                            );
                        }
                    }
                }
                syn::visit::visit_item_impl(self, impl_block);
                return;
            }
            Item::Trait(tr) => {
                if is_pub(&tr.vis) {
                    let lines = collect_outer_doc_lines(&tr.attrs);
                    let map = parse_tagged_doc_lines(&lines);
                    if let Err(detail) = validate_formal_block(&map, &tr.ident.to_string()) {
                        self.items.push(Violation {
                            file,
                            kind: Kind::Trait,
                            symbol: tr.ident.to_string(),
                            detail,
                        });
                    }
                }
                for it in &tr.items {
                    if let TraitItem::Fn(m) = it {
                        check_pub_fn_attrs(
                            m.sig.ident.to_string(),
                            &m.attrs,
                            file,
                            &mut self.items,
                        );
                    }
                }
                syn::visit::visit_item_trait(self, tr);
                return;
            }
            _ => {}
        }
        syn::visit::visit_item(self, i);
    }
}

fn visit_file(
    path: &Path,
    file_path: &'static str,
    acc: &mut Vec<Violation>,
) -> Result<(), Box<dyn Error>> {
    if path.extension() != Some(OsStr::new("rs")) {
        return Ok(());
    }
    let src = fs::read_to_string(path)?;
    let syn_file = syn::parse_file(&src)?;
    let mut v = AnchorVisitor {
        file_path,
        items: Vec::new(),
    };
    for item in syn_file.items {
        v.visit_item(&item);
    }
    acc.extend(v.items);
    Ok(())
}

fn walk_src() -> Result<Vec<Violation>, Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(p) = stack.pop() {
        for e in fs::read_dir(&p)? {
            let e = e?;
            let pth = e.path();
            if pth.is_dir() {
                stack.push(pth);
            } else {
                let rel = pth.strip_prefix(env!("CARGO_MANIFEST_DIR"))?;
                let static_rel: &'static str =
                    Box::leak(rel.to_string_lossy().into_owned().into_boxed_str());
                visit_file(&pth, static_rel, &mut out)?;
            }
        }
    }
    Ok(out)
}

#[test]
fn all_public_symbols_have_formal_anchor_doc() -> Result<(), Box<dyn Error>> {
    let missing = walk_src()?;
    if missing.is_empty() {
        return Ok(());
    }
    let mut by_file: BTreeMap<&str, Vec<&Violation>> = BTreeMap::new();
    for m in &missing {
        by_file.entry(m.file).or_default().push(m);
    }
    let mut msg =
        String::from("Formal documentation violations (anchor grammar / formal_status rules):\n");
    for (f, syms) in by_file {
        msg.push_str(f);
        msg.push('\n');
        for s in syms {
            msg.push_str(&format!("  {:?} {} — {}\n", s.kind, s.symbol, s.detail));
        }
    }
    panic!("{msg}");
}
