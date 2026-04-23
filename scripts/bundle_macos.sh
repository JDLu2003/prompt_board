#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/target/release/Prompt Board.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"

cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
cp "$ROOT_DIR/target/release/prompt_board" "$MACOS_DIR/prompt_board"
cp "$ROOT_DIR/packaging/macos/Info.plist" "$CONTENTS_DIR/Info.plist"

echo "Created: $APP_DIR"
