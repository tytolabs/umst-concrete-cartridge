// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Regenerates `docs/PROOF-STATUS.md` from a deterministic scan of formal doc blocks under
//! `src/**/*.rs` and `crates/umst-cli/src/**/*.rs`.

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{AttrStyle, ImplItem, Item, Meta, TraitItem, Visibility};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Row {
    file: String,
    line: usize,
    symbol: String,
    status: String,
    anchor: String,
    catalog_id: String,
    note: String,
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

fn ident_line(attrs: &[syn::Attribute], fallback: usize) -> usize {
    attrs
        .iter()
        .filter(|a| matches!(a.style, AttrStyle::Outer))
        .find_map(|a| extract_doc_line(a).is_some().then(|| a.span().start().line))
        .unwrap_or(fallback)
}

fn parse_map(lines: &[String]) -> BTreeMap<String, String> {
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

fn collect_outer_doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| matches!(a.style, AttrStyle::Outer))
        .filter_map(extract_doc_line)
        .collect()
}

fn note_from_map(map: &BTreeMap<String, String>) -> String {
    let st = map.get("formal_status").map(String::as_str).unwrap_or("");
    match st {
        "Empirical" => format!(
            "{} | {}",
            map.get("formal_citation").cloned().unwrap_or_default(),
            map.get("formal_envelope").cloned().unwrap_or_default()
        ),
        "Literature" => format!(
            "{} | {}",
            map.get("formal_citation").cloned().unwrap_or_default(),
            map.get("formal_form").cloned().unwrap_or_default()
        ),
        "Structural" => map
            .get("formal_anchor_rationale")
            .cloned()
            .unwrap_or_default(),
        "NONE" => map
            .get("formal_anchor_rationale")
            .cloned()
            .unwrap_or_default(),
        "Mechanised" => map.get("formal_axioms").cloned().unwrap_or_default(),
        _ => String::new(),
    }
}

fn push_row(
    rows: &mut Vec<Row>,
    file: &str,
    line: usize,
    symbol: String,
    attrs: &[syn::Attribute],
) {
    let lines = collect_outer_doc_lines(attrs);
    let map = parse_map(&lines);
    let status = map.get("formal_status").cloned().unwrap_or_default();
    let anchor = map.get("formal_anchor").cloned().unwrap_or_default();
    if status.is_empty() {
        return;
    }
    rows.push(Row {
        file: file.to_string(),
        line,
        symbol,
        status,
        anchor,
        catalog_id: map.get("catalog_id").cloned().unwrap_or_default(),
        note: note_from_map(&map),
    });
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
        syn::UseTree::Name(n) => out.push(n.ident.to_string()),
        syn::UseTree::Rename(r) => out.push(format!("{} as {}", r.ident, r.rename)),
        syn::UseTree::Glob(_) => out.push(format!("{prefix}::*")),
        syn::UseTree::Group(g) => {
            for t in &g.items {
                use_tree_leaf_names(t, prefix, out);
            }
        }
    }
}

struct CollectVisitor {
    pub file_path: String,
    pub rows: Vec<Row>,
}

impl<'ast> Visit<'ast> for CollectVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let file = self.file_path.as_str();
        match i {
            Item::Struct(s) if is_pub(&s.vis) => {
                let ln = ident_line(&s.attrs, s.ident.span().start().line);
                push_row(&mut self.rows, file, ln, s.ident.to_string(), &s.attrs);
            }
            Item::Enum(e) if is_pub(&e.vis) => {
                let ln = ident_line(&e.attrs, e.ident.span().start().line);
                push_row(&mut self.rows, file, ln, e.ident.to_string(), &e.attrs);
            }
            Item::Fn(f) if is_pub(&f.vis) => {
                let ln = ident_line(&f.attrs, f.sig.ident.span().start().line);
                push_row(&mut self.rows, file, ln, f.sig.ident.to_string(), &f.attrs);
            }
            Item::Const(c) if is_pub(&c.vis) => {
                let ln = ident_line(&c.attrs, c.ident.span().start().line);
                push_row(&mut self.rows, file, ln, c.ident.to_string(), &c.attrs);
            }
            Item::Type(t) if is_pub(&t.vis) => {
                let ln = ident_line(&t.attrs, t.ident.span().start().line);
                push_row(&mut self.rows, file, ln, t.ident.to_string(), &t.attrs);
            }
            Item::Use(u) if is_pub(&u.vis) => {
                let mut pfx = String::new();
                let mut names = Vec::new();
                use_tree_leaf_names(&u.tree, &mut pfx, &mut names);
                let sym = names.join(", ");
                let ln = ident_line(&u.attrs, u.span().start().line);
                push_row(&mut self.rows, file, ln, sym, &u.attrs);
            }
            Item::Impl(impl_block) => {
                for it in &impl_block.items {
                    if let ImplItem::Fn(m) = it {
                        if is_pub(&m.vis) {
                            let ln = ident_line(&m.attrs, m.sig.ident.span().start().line);
                            push_row(&mut self.rows, file, ln, m.sig.ident.to_string(), &m.attrs);
                        }
                    }
                }
                syn::visit::visit_item_impl(self, impl_block);
                return;
            }
            Item::Trait(tr) => {
                if is_pub(&tr.vis) {
                    let ln = ident_line(&tr.attrs, tr.ident.span().start().line);
                    push_row(&mut self.rows, file, ln, tr.ident.to_string(), &tr.attrs);
                }
                for it in &tr.items {
                    if let TraitItem::Fn(m) = it {
                        let ln = ident_line(&m.attrs, m.sig.ident.span().start().line);
                        push_row(&mut self.rows, file, ln, m.sig.ident.to_string(), &m.attrs);
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

fn collect_all_rows() -> Result<Vec<Row>, Box<dyn Error>> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("crate in workspace");
    let roots = [
        manifest.join("src"),
        workspace.join("crates/umst-cli/src"),
        workspace.join("crates/umst-mcp/src"),
        workspace.join("crates/umst-py/src"),
    ];
    let mut rows = Vec::new();
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
                    let rel = pth.strip_prefix(workspace)?;
                    let file_path = rel.to_string_lossy().into_owned();
                    let src = fs::read_to_string(&pth)?;
                    let syn_file = syn::parse_file(&src)?;
                    let mut v = CollectVisitor {
                        file_path,
                        rows: Vec::new(),
                    };
                    for item in syn_file.items {
                        v.visit_item(&item);
                    }
                    rows.extend(v.rows);
                }
            }
        }
    }
    rows.sort();
    Ok(rows)
}

fn render_markdown(rows: &[Row]) -> String {
    let order = [
        "Mechanised",
        "Structural",
        "Empirical",
        "Literature",
        "NONE",
    ];
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        *counts.entry(r.status.as_str()).or_insert(0) += 1;
    }

    let mut md = String::from(
        r#"<!--
-->

# Proof status (Rust cartridge sources)

Generated from `crates/umst-concrete-cartridge/src/**/*.rs`, `crates/umst-cli/src/**/*.rs`, `crates/umst-mcp/src/**/*.rs`, and `crates/umst-py/src/**/*.rs` formal documentation blocks. Regenerate with:

```bash
cargo test -p umst-concrete-cartridge --test proof_status_doc \
  proof_status_refresh_markdown_on_disk -- --ignored --nocapture
```

## Bucket counts

| formal_status | Symbols |
|---------------|---------|
"#,
    );
    for bucket in order {
        let c = counts.get(bucket).copied().unwrap_or(0);
        md.push_str(&format!("| **{bucket}** | {c} |\n"));
    }
    md.push('\n');

    for bucket in order {
        md.push_str(&format!("## {bucket}\n\n"));
        md.push_str(
            "| Symbol | File | formal_anchor | catalog_id | Citation / envelope / rationale |\n",
        );
        md.push_str(
            "|--------|------|---------------|------------|-----------------------------------|\n",
        );
        for r in rows.iter().filter(|x| x.status == bucket) {
            let note = r.note.replace('|', "\\|");
            let anch = r.anchor.replace('|', "\\|");
            let cid = r.catalog_id.replace('|', "\\|");
            md.push_str(&format!(
                "| `{}` | `{}:{}` | `{}` | {} | {} |\n",
                r.symbol,
                r.file,
                r.line,
                anch,
                if cid.is_empty() { "—" } else { &cid },
                note
            ));
        }
        md.push('\n');
    }

    md
}

fn generate_proof_status_documentation() -> Result<String, Box<dyn Error>> {
    let rows = collect_all_rows()?;
    Ok(render_markdown(&rows))
}

#[test]
fn proof_status_markdown_matches_committed_snapshot() -> Result<(), Box<dyn Error>> {
    let gen = generate_proof_status_documentation()?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/PROOF-STATUS.md")
        .canonicalize()
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/PROOF-STATUS.md")
        });
    let on_disk =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {} ({e})", path.display()));
    assert_eq!(
        gen, on_disk,
        "documentation drift: regenerate with `cargo test -p umst-concrete-cartridge --test proof_status_doc proof_status_refresh_markdown_on_disk -- --ignored`"
    );
    Ok(())
}

#[test]
#[ignore = "Writes docs/PROOF-STATUS.md; run intentionally after edits to Rust formal_status doc lines."]
fn proof_status_refresh_markdown_on_disk() -> Result<(), Box<dyn Error>> {
    let gen = generate_proof_status_documentation()?;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/PROOF-STATUS.md");
    fs::write(&path, gen).expect("write PROOF-STATUS.md");
    Ok(())
}
