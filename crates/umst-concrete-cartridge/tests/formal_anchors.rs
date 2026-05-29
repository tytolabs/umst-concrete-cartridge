// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! CI guardrail: five-status formal documentation (`Mechanised | Structural | Empirical |
//! Literature | NONE`) on every `pub` surface in repository Rust sources (`src/`, `crates/umst-cli/src/`).

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{AttrStyle, ImplItem, Item, Meta, TraitItem, Visibility};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Struct,
    Enum,
    Trait,
    Fn,
    Const,
    Type,
    Use,
}

struct Violation {
    pub file: &'static str,
    pub line: Option<usize>,
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

fn doc_line_span_line(attr: &syn::Attribute) -> Option<usize> {
    extract_doc_line(attr)
        .is_some()
        .then(|| attr.span().start().line)
}

fn outer_doc_span_line(attrs: &[syn::Attribute]) -> Option<usize> {
    attrs
        .iter()
        .filter(|a| matches!(a.style, AttrStyle::Outer))
        .find_map(doc_line_span_line)
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
                | "catalog_id"
        ) {
            continue;
        }
        map.insert(key.to_string(), rest.trim().to_string());
    }
    map
}

/// `lean://umst-formal/…` URI → manifold `catalog_id` (see `docs/FORMAL_GROUNDING_AUDIT.md`).
const MECHANISED_ANCHOR_CATALOG_IDS: &[(&str, &str)] = &[
    (
        "lean://umst-formal/Lean/Gate.lean#Admissible",
        "umst.gate.cd_transition",
    ),
    (
        "lean://umst-formal/Lean/Powers.lean#PowersState",
        "thermodynamic_mix",
    ),
    (
        "lean://umst-formal/Lean/Powers.lean#S_intrinsic",
        "thermodynamic_mix",
    ),
    (
        "lean://umst-formal/Lean/Powers.lean#powers_monotone",
        "thermodynamic_mix",
    ),
    (
        "lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime",
        "umst.cartridge.concrete.regime",
    ),
    (
        "lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration",
        "umst.cartridge.concrete.acceptance_band",
    ),
    (
        "lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility",
        "umst.cartridge.concrete.acceptance_band",
    ),
    (
        "lean://umst-formal/Lean/JenningsGelSpace.lean#jennings_strength_monotone",
        "umst.cartridge.concrete.jennings_gel",
    ),
    (
        "lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz",
        "umst.gate.cd_transition",
    ),
    (
        "lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy",
        "umst.gate.landauer_cbf",
    ),
];

fn expected_catalog_id_for_mechanised_anchor(anchor: &str) -> Option<&'static str> {
    MECHANISED_ANCHOR_CATALOG_IDS
        .iter()
        .find(|(a, _)| *a == anchor)
        .map(|(_, id)| *id)
}

fn validate_mechanised_axioms(ax: &str) -> Result<(), String> {
    if ax.is_empty() {
        return Err("Mechanised requires non-empty `/// formal_axioms:` (use NONE)".into());
    }
    for part in ax.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if p != "NONE" && p != "physicalSecondLaw" {
            return Err(format!(
                "Mechanised formal_axioms allows only NONE or physicalSecondLaw; got `{p}`"
            ));
        }
    }
    Ok(())
}

fn validate_formal_block(map: &BTreeMap<String, String>, symbol: &str) -> Result<(), String> {
    let anchor = map
        .get("formal_anchor")
        .ok_or_else(|| format!("{symbol}: missing `/// formal_anchor:`"))?;
    let status_raw = map
        .get("formal_status")
        .ok_or_else(|| format!("{symbol}: missing `/// formal_status:`"))?;
    let status = status_raw.trim();

    if status.contains(' ') || status.is_empty() {
        return Err(format!(
            "{symbol}: malformed formal_status `{status_raw}` (must be a single token)"
        ));
    }

    match status {
        "Library" => {
            return Err(format!(
                "{symbol}: formal_status Library is retired — use Empirical, Literature, or NONE"
            ));
        }
        "Boundary" => {
            return Err(format!(
                "{symbol}: Boundary is a verification_status (TOML), not a formal_status (Rust)"
            ));
        }
        "Option" => {
            return Err(format!(
                "{symbol}: formal_status Option is invalid (likely a serde field false-positive)"
            ));
        }
        _ => {}
    }

    const ALLOWED: &[&str] = &[
        "Mechanised",
        "Structural",
        "Empirical",
        "Literature",
        "NONE",
    ];
    if !ALLOWED.contains(&status) {
        return Err(format!(
            "{symbol}: unknown formal_status `{status}` — allowed: {}",
            ALLOWED.join(", ")
        ));
    }

    match status {
        "Mechanised" => {
            if !anchor.starts_with("lean://") {
                return Err(format!(
                    "{symbol}: Mechanised requires lean:// anchor, got `{anchor}`"
                ));
            }
            let ax = map.get("formal_axioms").map(String::as_str).unwrap_or("");
            validate_mechanised_axioms(ax)?;
            let Some(expected) = expected_catalog_id_for_mechanised_anchor(anchor) else {
                return Err(format!(
                    "{symbol}: Mechanised lean:// anchor not in FORMAL_GROUNDING_AUDIT map: `{anchor}`"
                ));
            };
            let got = map
                .get("catalog_id")
                .map(String::as_str)
                .unwrap_or("")
                .trim();
            if got.is_empty() {
                return Err(format!(
                    "{symbol}: Mechanised requires `/// catalog_id:` (expected `{expected}`)"
                ));
            }
            if got != expected {
                return Err(format!(
                    "{symbol}: catalog_id `{got}` must match audit map `{expected}` for `{anchor}`"
                ));
            }
        }
        "Structural" => {
            if anchor.trim() != "STRUCTURAL" {
                return Err(format!(
                    "{symbol}: Structural requires `formal_anchor: STRUCTURAL`, got `{anchor}`"
                ));
            }
            let rat = map
                .get("formal_anchor_rationale")
                .map(String::as_str)
                .unwrap_or("");
            if rat.is_empty() {
                return Err(format!(
                    "{symbol}: Structural requires `/// formal_anchor_rationale:` (Rust carrying the property)"
                ));
            }
        }
        "Empirical" => {
            if !anchor.starts_with("empirical://") {
                return Err(format!(
                    "{symbol}: Empirical requires empirical:// anchor, got `{anchor}`"
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
                    "{symbol}: Literature requires literature:// anchor, got `{anchor}`"
                ));
            }
            for k in ["formal_citation", "formal_form"] {
                if !map.contains_key(k) {
                    return Err(format!("{symbol}: Literature requires `/// {k}:`"));
                }
            }
        }
        "NONE" => {
            if anchor.trim() != "NONE" {
                return Err(format!(
                    "{symbol}: NONE status requires `formal_anchor: NONE`, got `{anchor}`"
                ));
            }
            let rat = map
                .get("formal_anchor_rationale")
                .map(String::as_str)
                .unwrap_or("");
            if rat.is_empty() {
                return Err(format!(
                    "{symbol}: NONE requires `/// formal_anchor_rationale:`"
                ));
            }
            let rl = rat.to_ascii_lowercase();
            if rl.contains("differentiable training") || rl.contains("training pathway") {
                return Err(format!(
                    "{symbol}: NONE rationale must not contain Differentiable training / training pathway"
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn check_attrs(
    sym: String,
    attrs: &[syn::Attribute],
    file_path: &'static str,
    kind: Kind,
    acc: &mut Vec<Violation>,
) {
    let lines = collect_outer_doc_lines(attrs);
    let map = parse_tagged_doc_lines(&lines);
    let line = outer_doc_span_line(attrs);
    if let Err(detail) = validate_formal_block(&map, &sym) {
        acc.push(Violation {
            file: file_path,
            line,
            kind,
            symbol: sym,
            detail,
        });
    }
}

fn use_tree_leaf_names(tree: &syn::UseTree, prefix: &mut String, out: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(p) => {
            let seg = p.ident.to_string();
            let prev = prefix.clone();
            if prefix.is_empty() {
                *prefix = seg;
            } else {
                prefix.push_str("::");
                prefix.push_str(&seg);
            }
            use_tree_leaf_names(&p.tree, prefix, out);
            *prefix = prev;
        }
        syn::UseTree::Name(n) => {
            out.push(n.ident.to_string());
        }
        syn::UseTree::Rename(r) => {
            out.push(format!("{} as {}", r.ident, r.rename));
        }
        syn::UseTree::Glob(_) => {
            out.push(format!("{prefix}::*"));
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                use_tree_leaf_names(t, prefix, out);
            }
        }
    }
}

impl<'ast> Visit<'ast> for AnchorVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let file = self.file_path;
        match i {
            Item::Struct(s) if is_pub(&s.vis) => {
                check_attrs(
                    s.ident.to_string(),
                    &s.attrs,
                    file,
                    Kind::Struct,
                    &mut self.items,
                );
            }
            Item::Enum(e) if is_pub(&e.vis) => {
                check_attrs(
                    e.ident.to_string(),
                    &e.attrs,
                    file,
                    Kind::Enum,
                    &mut self.items,
                );
            }
            Item::Fn(f) if is_pub(&f.vis) => {
                check_attrs(
                    f.sig.ident.to_string(),
                    &f.attrs,
                    file,
                    Kind::Fn,
                    &mut self.items,
                );
            }
            Item::Const(c) if is_pub(&c.vis) => {
                check_attrs(
                    c.ident.to_string(),
                    &c.attrs,
                    file,
                    Kind::Const,
                    &mut self.items,
                );
            }
            Item::Type(t) if is_pub(&t.vis) => {
                check_attrs(
                    t.ident.to_string(),
                    &t.attrs,
                    file,
                    Kind::Type,
                    &mut self.items,
                );
            }
            Item::Use(u) if is_pub(&u.vis) => {
                let mut pfx = String::new();
                let mut names = Vec::new();
                use_tree_leaf_names(&u.tree, &mut pfx, &mut names);
                let sym = if names.is_empty() {
                    "use".into()
                } else {
                    names.join(", ")
                };
                check_attrs(sym, &u.attrs, file, Kind::Use, &mut self.items);
            }
            Item::Impl(impl_block) => {
                for it in &impl_block.items {
                    if let ImplItem::Fn(m) = it {
                        if is_pub(&m.vis) {
                            check_attrs(
                                m.sig.ident.to_string(),
                                &m.attrs,
                                file,
                                Kind::Fn,
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
                    check_attrs(
                        tr.ident.to_string(),
                        &tr.attrs,
                        file,
                        Kind::Trait,
                        &mut self.items,
                    );
                }
                for it in &tr.items {
                    if let TraitItem::Fn(m) = it {
                        check_attrs(
                            m.sig.ident.to_string(),
                            &m.attrs,
                            file,
                            Kind::Fn,
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
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(Path::parent)
        .expect("crate in workspace");
    let roots = [
        manifest.join("src"),
        workspace.join("crates/umst-cli/src"),
        workspace.join("crates/umst-mcp/src"),
        workspace.join("crates/umst-py/src"),
    ];
    let mut out = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        let mut stack = vec![root];
        while let Some(p) = stack.pop() {
            for e in fs::read_dir(&p)? {
                let e = e?;
                let pth = e.path();
                if pth.is_dir() {
                    stack.push(pth);
                } else {
                    let rel = pth.strip_prefix(workspace).unwrap_or_else(|_| {
                        panic!("walk_src path {pth:?} not under workspace root {workspace:?}",)
                    });
                    let static_rel: &'static str =
                        Box::leak(rel.to_string_lossy().into_owned().into_boxed_str());
                    visit_file(&pth, static_rel, &mut out)?;
                }
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
    let mut msg = String::from("Formal documentation violations (five-status grammar):\n");
    for (f, syms) in by_file {
        msg.push_str(f);
        msg.push('\n');
        for s in syms {
            let ln = s.line.map(|n| format!("line {n}: ")).unwrap_or_default();
            msg.push_str(&format!("  {:?} {} — {ln}{}\n", s.kind, s.symbol, s.detail));
        }
    }
    panic!("{msg}");
}

#[test]
fn src_formal_status_histogram_sanity() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(Path::parent)
        .expect("crate in workspace");
    let roots = [
        manifest.join("src"),
        workspace.join("crates/umst-cli/src"),
        workspace.join("crates/umst-mcp/src"),
        workspace.join("crates/umst-py/src"),
    ];
    let mut combined = String::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        let mut stack = vec![root];
        while let Some(p) = stack.pop() {
            for e in fs::read_dir(&p)? {
                let e = e?;
                let pth = e.path();
                if pth.is_dir() {
                    stack.push(pth);
                } else if pth.extension() == Some(OsStr::new("rs")) {
                    combined.push_str(&fs::read_to_string(&pth)?);
                    combined.push('\n');
                }
            }
        }
    }
    fn count(hay: &str, status: &str) -> usize {
        let pat = format!("formal_status: {status}");
        hay.matches(&pat).count()
    }
    assert!(
        count(&combined, "Mechanised") >= 14,
        "Mechanised {}",
        count(&combined, "Mechanised")
    );
    assert!(
        count(&combined, "Structural") >= 12,
        "Structural {}",
        count(&combined, "Structural")
    );
    assert!(
        count(&combined, "Empirical") >= 10,
        "Empirical {}",
        count(&combined, "Empirical")
    );
    assert!(
        count(&combined, "Literature") >= 8,
        "Literature {}",
        count(&combined, "Literature")
    );
    assert!(
        count(&combined, "NONE") >= 8,
        "NONE {}",
        count(&combined, "NONE")
    );
    assert_eq!(count(&combined, "Library"), 0);
    assert_eq!(count(&combined, "Boundary"), 0);
    Ok(())
}

#[test]
fn lint_rejects_boundary_status_token() {
    let mut m = BTreeMap::new();
    m.insert("formal_anchor".into(), "NONE".into());
    m.insert("formal_status".into(), "Boundary".into());
    m.insert("formal_anchor_rationale".into(), "stub".into());
    assert!(validate_formal_block(&m, "test").is_err());
}

#[test]
fn lint_rejects_library_status_token() {
    let mut m = BTreeMap::new();
    m.insert("formal_anchor".into(), "NONE".into());
    m.insert("formal_status".into(), "Library".into());
    m.insert("formal_anchor_rationale".into(), "stub".into());
    assert!(validate_formal_block(&m, "test").is_err());
}

#[test]
fn lint_structural_requires_structural_anchor() {
    let mut m = BTreeMap::new();
    m.insert("formal_anchor".into(), "lean://x".into());
    m.insert("formal_status".into(), "Structural".into());
    m.insert("formal_anchor_rationale".into(), "bad".into());
    assert!(validate_formal_block(&m, "test").is_err());

    let mut m = BTreeMap::new();
    m.insert("formal_anchor".into(), "STRUCTURAL".into());
    m.insert("formal_status".into(), "Structural".into());
    m.insert("formal_anchor_rationale".into(), "exhaustive enum".into());
    assert!(validate_formal_block(&m, "test").is_ok());
}

#[test]
fn lint_none_bans_training_boilerplate_substrings() {
    let mut m = BTreeMap::new();
    m.insert("formal_anchor".into(), "NONE".into());
    m.insert("formal_status".into(), "NONE".into());
    m.insert(
        "formal_anchor_rationale".into(),
        "Differentiable training pathway".into(),
    );
    assert!(validate_formal_block(&m, "test").is_err());
}
