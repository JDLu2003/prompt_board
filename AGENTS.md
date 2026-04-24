# AGENTS.md

A macOS prompt-management MVP built with Rust + egui/eframe. Three floating panels, SQLite-backed, global hotkey activation.

## Commands

```bash
cargo fmt -- --check    # format check
cargo check --locked    # compile check
cargo test --locked     # unit tests
cargo run               # run GUI (NOT in CI)
bash -n scripts/bundle_macos.sh   # validate bundle script
./scripts/bundle_macos.sh         # build .app bundle
```

Pre-merge minimum:
```bash
cargo fmt -- --check && cargo check --locked && cargo test --locked && bash -n scripts/bundle_macos.sh
```

If changes touch windows, hotkeys, clipboard, Chinese fonts, or panel layout, also `cargo run` and manually verify the interactions listed in README.md.

## Architecture

- **Single binary**: `src/main.rs` -> `PromptBoardApp` in `src/app.rs`
- `src/db.rs` — SQLite via rusqlite (bundled feature, `~/Library/Application Support/Prompt Board/data.db`)
- `src/system.rs` — macOS global hotkey, clipboard, window-focus events
- `src/template.rs` — variable extraction (`[name]` / `[name|default]`) + rendering; has unit tests that **must be updated** when template logic changes
- `.app` built by `scripts/bundle_macos.sh`, installed to `~/Applications/Prompt Board.app`
- `packaging/macos/Info.plist` — uses `LSUIElement` (no Dock icon); version strings must match `Cargo.toml` on release

## Key Constraints

- **macOS-only**: uses `global-hotkey`, AppKit, arboard. CI runs on `macos-latest`. Windows/Linux won't compile.
- `--locked` everywhere to respect `Cargo.lock`.
- Never add `target/` to version control.
- GUI dep upgrades (eframe, global-hotkey, arboard, egui_commonmark) must pass a manual `cargo run` smoke test.
- macOS system logs like `Connection invalid` from input methods are **not crashes**.
- Update replaces only the `.app` bundle; the database directory is never touched.

## Keyboard Shortcuts (to verify when modifying UI)

| Key | Action |
|---|---|
| `Cmd+Shift+P` | Toggle/refocus floating panels |
| `Up`/`Down` | Switch selected prompt |
| `Enter` | Open Panel C (variable fill) or advance field; last field copies to clipboard |
| `Shift+Enter` | Newline in current fill field |
| `Esc` | Hide panels or return from Panel C to selection |
| `Cmd+C` | Copy Panel B preview |
| `Cmd+N` | New prompt |
| `Cmd+E` | Edit prompt |
| `Cmd+S` | Save (edit form) |
| `Cmd+Backspace` | Delete prompt |

## CI

Two workflows exist in `.github/workflows/`:
- **ci.yml** — `cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, bundle script validation, release build, artifact upload. Runs on push/PR to `main`.
- **release.yml** — triggered by `v*` tags. Builds, zips unsigned `.app`, creates GitHub Release.

## Releasing

1. Bump version in `Cargo.toml` `package.version`, and both `CFBundleShortVersionString` and `CFBundleVersion` in `packaging/macos/Info.plist`.
2. Commit, tag `vX.Y.Z`, push both `main` and the tag.
