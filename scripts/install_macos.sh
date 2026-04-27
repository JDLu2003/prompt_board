#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${PROMPT_BOARD_INSTALL_DIR:-"$HOME/Applications"}"
APP_NAME="Prompt Board.app"
SOURCE_APP="$ROOT_DIR/target/release/$APP_NAME"
TARGET_APP="$INSTALL_DIR/$APP_NAME"
DATABASE_PATH="$HOME/Library/Application Support/Prompt Board/data.db"

echo "=== Step 1/4: Building Prompt Board ==="
"$ROOT_DIR/scripts/bundle_macos.sh"

if [[ ! -d "$SOURCE_APP" ]]; then
  echo "ERROR: Build failed — $SOURCE_APP does not exist."
  exit 1
fi
echo "  Build OK: $SOURCE_APP"

echo "=== Step 2/4: Installing to $TARGET_APP ==="
mkdir -p "$INSTALL_DIR"

if [[ -d "$TARGET_APP" ]]; then
  rm -rf "$TARGET_APP"
fi

cp -R "$SOURCE_APP" "$TARGET_APP"

echo "=== Step 3/4: Verifying installation ==="
VERIFY_FAILED=0

if [[ ! -d "$TARGET_APP" ]]; then
  echo "  FAIL: $TARGET_APP not found after copy"
  VERIFY_FAILED=1
fi

if [[ ! -f "$TARGET_APP/Contents/MacOS/prompt_board" ]]; then
  echo "  FAIL: executable missing inside bundle"
  VERIFY_FAILED=1
fi

if [[ ! -f "$TARGET_APP/Contents/Info.plist" ]]; then
  echo "  FAIL: Info.plist missing inside bundle"
  VERIFY_FAILED=1
fi

if [[ ! -x "$TARGET_APP/Contents/MacOS/prompt_board" ]]; then
  echo "  FAIL: executable is not executable"
  VERIFY_FAILED=1
fi

if [[ "$VERIFY_FAILED" -ne 0 ]]; then
  echo "Installation verification FAILED."
  exit 1
fi

BUNDLE_SIZE=$(du -sh "$TARGET_APP" | cut -f1)
echo "  OK: $TARGET_APP ($BUNDLE_SIZE)"

echo "=== Step 4/4: Launching Prompt Board ==="
open "$TARGET_APP"

echo
echo "Installed: $TARGET_APP"
echo "Database:  $DATABASE_PATH"
echo "Use Command + Shift + P to show the floating panels."
