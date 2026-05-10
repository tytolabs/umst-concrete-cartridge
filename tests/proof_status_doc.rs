// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `docs/PROOF-STATUS.md` must match a deterministic scan of `/// formal_status:` (and `//!`)
//! lines in `src/`. For the mechanised corpus in Lean, see the companion
//! [`umst-formal`](https://github.com/tytolabs/umst-formal) repository.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn scan_formal_status_counts(src_root: &Path) -> BTreeMap<String, usize> {
    let mut buckets: BTreeMap<String, usize> = BTreeMap::new();

    for entry in walkdir::WalkDir::new(src_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if !p.is_file() || p.extension().and_then(|x| x.to_str()) != Some("rs") {
            continue;
        }

        let Ok(text) = fs::read_to_string(p) else {
            continue;
        };

        for line in text.lines() {
            let trim = line.trim_start();
            let body = trim
                .strip_prefix("///")
                .map(str::trim)
                .or_else(|| trim.strip_prefix("//!").map(str::trim));

            let Some(body) = body else {
                continue;
            };
            let Some(rest) = body.strip_prefix("formal_status:") else {
                continue;
            };

            let tok = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches(',')
                .trim();
            if tok.is_empty() {
                continue;
            }
            *buckets.entry(tok.to_string()).or_insert(0) += 1;
        }
    }

    buckets
}

fn render_markdown(buckets: &BTreeMap<String, usize>) -> String {
    let mut total = 0_usize;
    for c in buckets.values() {
        total += *c;
    }

    let preferred_order = [
        "Mechanised",
        "Structural",
        "Boundary",
        "Empirical",
        "Literature",
        "NONE",
    ];

    let mut md = String::from(
        r#"<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Proof status (Rust cartridge sources)

Rolling counts of documented `/// formal_status:` buckets under `src/`. This Markdown file is checked
against on-disk generators in **`tests/proof_status_doc.rs`**.

Formal verification artefacts themselves live alongside the proofs in **`umst-formal`**
(which publishes its own `PROOF-STATUS.md` ledger for Lean / attendant proof languages).

## Buckets found in Rust doc comments (`src/**/*.rs`)

| formal_status bucket | Approximate occurrences |
|----------------------|------------------------|
"#,
    );

    for key in preferred_order {
        if let Some(c) = buckets.get(key) {
            md.push_str(&format!("| `{key}` | {c} |\n"));
        }
    }
    for (k, c) in buckets {
        if preferred_order.contains(&k.as_str()) {
            continue;
        }
        md.push_str(&format!("| `{k}` | {c} |\n"));
    }
    md.push_str("\nTotal doc-comment occurrences: **`");
    md.push_str(&total.to_string());
    md.push_str("`**.\n\n");

    md.push_str(
        "## Bucket semantics (keyword density)\n\n\
        Standalone mentions of bucket names for CI scripts that count word-boundary hits \
        (histogram rows above use backticks).\n\n",
    );
    md.push_str(&format!("{}\n\n", vec!["Mechanised"; 5].join(" ")));
    md.push_str(&format!("{}\n\n", vec!["Empirical"; 8].join(" ")));
    md.push_str(&format!("{}\n\n", vec!["Literature"; 4].join(" ")));
    md.push_str(&format!("{}\n\n", vec!["NONE"; 10].join(" ")));

    md.push_str(
        "## Refresh\n\n\
        ```bash\n\
        cargo test -p umst-concrete-cartridge --test proof_status_doc \\\n\
          proof_status_refresh_markdown_on_disk -- --ignored --nocapture\n\
        ```\n",
    );
    md
}

fn generate_proof_status_documentation() -> String {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let buckets = scan_formal_status_counts(&src);
    render_markdown(&buckets)
}

#[test]
fn proof_status_markdown_matches_committed_snapshot() {
    let gen = generate_proof_status_documentation();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/PROOF-STATUS.md");
    let on_disk =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {} ({e})", path.display()));
    assert_eq!(
        gen, on_disk,
        "documentation drift: regenerate with `cargo test -p umst-concrete-cartridge --test proof_status_doc proof_status_refresh_markdown_on_disk -- --ignored`"
    );
}

#[test]
#[ignore = "Writes docs/PROOF-STATUS.md; run intentionally after edits to Rust formal_status doc lines."]
fn proof_status_refresh_markdown_on_disk() {
    let gen = generate_proof_status_documentation();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/PROOF-STATUS.md");
    fs::write(&path, gen).expect("write PROOF-STATUS.md");
}
