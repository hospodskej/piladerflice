# Sapwood

The framework this site is built on, one crate per folder. See [sapwood/README.md](sapwood/README.md) for what it is; each crate's README says what it does.

| Folder | Crate |
|--------|-------|
| `sapwood/` | `sapwood`: all of the below as one dependency, and `examples/guestbook.rs` |
| `sapwood-session/` | encrypted cookie sessions, CSRF token, flash, login cookie |
| `sapwood-forms/` | CSRF-checked form extractors, validation errors |
| `sapwood-i18n/` | YAML translations (a MiniJinja `t()` with the `minijinja` feature) |
| `sapwood-assets/` | fingerprinted static files, import map |

Publishing (by the owner of the crates.io account): `cargo publish --workspace` from the repository root publishes them in the right order; `cargo publish --workspace --dry-run` checks everything first.
