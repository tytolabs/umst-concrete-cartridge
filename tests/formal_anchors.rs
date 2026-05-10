// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! CI guardrail: every `pub fn`, `pub struct`, `pub enum`, and `pub trait` in `src/**/*.rs`
//! must declare a `/// formal_anchor:` line in its immediately preceding doc comment block.

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

struct PubItem {
    pub file: &'static str,
    pub kind: Kind,
    pub symbol: String,
}

struct AnchorVisitor {
    pub file_path: &'static str,
    pub items: Vec<PubItem>,
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

fn doc_contains_formal_anchor(attrs: &[syn::Attribute]) -> (bool, bool) {
    let mut anchor_none = false;
    let has_anchor = attrs.iter().any(|a| {
        if matches!(a.style, AttrStyle::Inner(_)) {
            return false;
        }
        extract_doc_line(a).is_some_and(|ln| ln.trim().starts_with("formal_anchor:"))
    });
    for a in attrs {
        if let Some(ln) = extract_doc_line(a) {
            let t = ln.trim();
            if t.starts_with("formal_anchor:") && t.contains("NONE") {
                anchor_none = true;
            }
        }
    }
    (has_anchor, anchor_none)
}

fn doc_has_none_rationale(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        extract_doc_line(a).is_some_and(|ln| ln.trim().starts_with("formal_anchor_rationale:"))
    })
}

fn check_pub_fn_attrs(
    sym: String,
    attrs: &[syn::Attribute],
    file_path: &'static str,
    acc: &mut Vec<PubItem>,
) {
    let (has_anchor, anchor_none) = doc_contains_formal_anchor(attrs);
    if !has_anchor {
        acc.push(PubItem {
            file: file_path,
            kind: Kind::Fn,
            symbol: sym.clone(),
        });
    }
    if has_anchor && anchor_none && !doc_has_none_rationale(attrs) {
        acc.push(PubItem {
            file: file_path,
            kind: Kind::Fn,
            symbol: sym,
        });
    }
}

impl<'ast> Visit<'ast> for AnchorVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let file = self.file_path;
        match i {
            Item::Struct(s) if is_pub(&s.vis) => {
                let (ok, none) = doc_contains_formal_anchor(&s.attrs);
                if !ok || (none && !doc_has_none_rationale(&s.attrs)) {
                    self.items.push(PubItem {
                        file,
                        kind: Kind::Struct,
                        symbol: s.ident.to_string(),
                    });
                }
            }
            Item::Enum(e) if is_pub(&e.vis) => {
                let (ok, none) = doc_contains_formal_anchor(&e.attrs);
                if !ok || (none && !doc_has_none_rationale(&e.attrs)) {
                    self.items.push(PubItem {
                        file,
                        kind: Kind::Enum,
                        symbol: e.ident.to_string(),
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
                    let (ok, none) = doc_contains_formal_anchor(&tr.attrs);
                    if !ok || (none && !doc_has_none_rationale(&tr.attrs)) {
                        self.items.push(PubItem {
                            file,
                            kind: Kind::Trait,
                            symbol: tr.ident.to_string(),
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
    acc: &mut Vec<PubItem>,
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

fn walk_src() -> Result<Vec<PubItem>, Box<dyn Error>> {
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
    let mut by_file: BTreeMap<&str, Vec<&PubItem>> = BTreeMap::new();
    for m in &missing {
        by_file.entry(m.file).or_default().push(m);
    }
    let mut msg = String::from("Missing formal_anchor or formal_anchor_rationale (for NONE):\n");
    for (f, syms) in by_file {
        msg.push_str(f);
        msg.push('\n');
        for s in syms {
            msg.push_str(&format!("  {:?} {}\n", s.kind, s.symbol));
        }
    }
    panic!("{msg}");
}
