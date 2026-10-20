#!/bin/bash
# Bundle the real skill catalog (repo-root skills/) into the built Minime.app
# bundles so the Rust backend finds it at runtime (<exe>/../Resources/skills).
# Run this after every `tauri build` (debug or release); the DMG/.dmg
# installer inherits the .app contents automatically.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/../skills"
if [ ! -d "$SRC" ]; then
  echo "bundle-skills: skills/ not found at $SRC" >&2
  exit 1
fi
for PROFILE in debug release; do
  APP="$ROOT/src-tauri/target/$PROFILE/bundle/macos/Minime.app"
  if [ -d "$APP" ]; then
    rm -rf "$APP/Contents/Resources/skills"
    cp -R "$SRC" "$APP/Contents/Resources/skills"
    echo "bundle-skills: skills -> $APP"
  fi
done
