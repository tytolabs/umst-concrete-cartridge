# Contributing to UMST Concrete Cartridge

We welcome contributions to the Unified Material-State Tensor (UMST) Concrete Cartridge.

## Code of Conduct
Please ensure all discussions and pull requests remain respectful, academic, and rigorous.

## Reporting Issues
When opening an issue, please provide a minimal reproducible example (MRE).

## Pull Requests
All pull requests must adhere to the following standards:
1. **Rust Toolchain**: Pass `cargo clippy -- -D warnings` and `cargo fmt -- --check`. Our Minimum Supported Rust Version (MSRV) is **1.75**.
2. **Computational Rigour**: Include unit tests for any new physics module ensuring purely functional tensor operations without memory leaks.
3. **Scientific Citations**: Provide citations (DOI or published paper) for newly added constitutive models in the PR description and in `docs/Constitutive-Equations.md`. Unreferenced "folklore" heuristics will be rejected.
