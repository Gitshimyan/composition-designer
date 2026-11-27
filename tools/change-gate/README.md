# composition-change-gate

Manifest × git diff × task-card coverage checks. This is a governance check only:
it does not grant permissions, validate Composition semantics, or replace human review.

## Usage

```powershell
# report only (CI default)
cargo run -p composition-change-gate -- --base origin/main --head HEAD --mode report

# blocking mode (not enabled in CI yet)
cargo run -p composition-change-gate -- --base origin/main --head HEAD --mode block
```

Options:

- `--base <ref>` / `--head <ref>`: git refs; defaults `HEAD^` / `HEAD` or
  `GITHUB_BASE_SHA` / `GITHUB_HEAD_SHA`;
- `--mode report|block`: report always exits 0; block exits 1 on errors;
- `--manifest-dir changes` / `--task-dir working/tasks`.

## Checks

- every changed file must be inside at least one manifest `targets`;
- required authorizations must be both declared and approved;
- required operation must be listed: `compose` for `examples/**` and
  `working/pilots/**`, `extend` for new crates, command schema/editor changes, and CI
  workflow changes, `modify` otherwise;
- sensitive paths (`AGENTS.md`, `docs/design/**`, `docs/spec/**`, root
  `Cargo.toml`/`Cargo.lock`, `.cargo/**`, `rust-toolchain*`) require `rearchitect`;
- forbidden path patterns are checked when they look like paths;
- manifests with `coverage_status` are compared with the matching task card
  (`coverage_status` and `open_items`).

## Heuristics and limits

- New crates are detected from added `crates/*/Cargo.toml` or `tools/*/Cargo.toml`;
- Command changes are detected from the editor command module and command schema;
- Adapter/field-level semantic changes are only inferred from paths; this is not a
  replacement for Core validation or diff review;
- Legacy manifests without `coverage_status` are not coverage-checked; add the field
  when touching a manifest.

## CI

`.github/workflows/ci.yml` runs report mode on pull requests and pushes with
`fetch-depth: 0`. Switch to `--mode block` only after collecting false positives and
updating this document. The required status check remains
`Windows build and UI tests`.
