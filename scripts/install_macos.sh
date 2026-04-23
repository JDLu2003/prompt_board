#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${PROMPT_BOARD_INSTALL_DIR:-"$HOME/Applications"}"
APP_NAME="Prompt Board.app"
SOURCE_APP="$ROOT_DIR/target/release/$APP_NAME"
TARGET_APP="$INSTALL_DIR/$APP_NAME"
DATABASE_PATH="$HOME/Library/Application Support/Prompt Board/data.db"

echo "Building Prompt Board..."
"$ROOT_DIR/scripts/bundle_macos.sh"

echo "Installing to $TARGET_APP..."
mkdir -p "$INSTALL_DIR"

if [[ -d "$TARGET_APP" ]]; then
  rm -rf "$TARGET_APP"
fi

cp -R "$SOURCE_APP" "$TARGET_APP"

echo "Launching Prompt Board..."
open "$TARGET_APP"

echo
echo "Installed: $TARGET_APP"
echo "Database:  $DATABASE_PATH"
echo "Use Command + Shift + P to show the floating panels."
