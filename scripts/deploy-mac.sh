#!/bin/bash
# deploy-mac.sh — Deploy Minime to the MacBook in one command.
#
# Runs all steps in sequence + verification at each stage:
#   1. git pull          (sync latest code)
#   2. vite build        (frontend; MUST run from app/src)
#   3. tauri build       (backend + bundle .app)
#   4. bundle-skills.sh  (bundle skills into the app)
#   5. restart app       (kill + clear WebView cache + open)
#   6. verify            (app running + skills bundled)
#
# Usage (on the MacBook):
#   bash ~/minime/scripts/deploy-mac.sh
#
# Exits non-zero with a clear message if any stage fails.
# No step can be silently skipped.

set -euo pipefail

# --- Configuration ---
MINIME_DIR="${MINIME_DIR:-$HOME/minime}"
APP_DIR="$MINIME_DIR/app"
SRC_DIR="$APP_DIR/src"
BUNDLE="$APP_DIR/src-tauri/target/debug/bundle/macos/Minime.app"
BUNDLE_ID="id.minime.app"
export PATH="/usr/local/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"

# --- Helper ---
step()  { echo ""; echo "━━━ [$1/6] $2 ━━━"; }
ok()    { echo "✅ $1"; }
fail()  { echo "❌ GAGAL: $1" >&2; exit 1; }

echo "🚀 Deploy Minime — $(date '+%Y-%m-%d %H:%M:%S')"

# ── 1/6: git pull ──────────────────────────────────────────────
step 1 "git pull"
cd "$MINIME_DIR" || fail "directory $MINIME_DIR not found"
git pull || fail "git pull failed — check connection / local conflicts"
ok "latest code pulled ($(git rev-parse --short HEAD))"

# ── 2/6: vite build ────────────────────────────────────────────
step 2 "vite build (frontend)"
cd "$SRC_DIR" || fail "directory $SRC_DIR not found"
VITE_BIN="../src/node_modules/.bin/vite"
[ -x "$VITE_BIN" ] || fail "vite not found at $VITE_BIN — run pnpm install first"
"$VITE_BIN" build || fail "vite build failed"
# Verify: dist must exist and be non-empty
[ -d "dist" ] && [ -n "$(ls -A dist 2>/dev/null)" ] || fail "dist folder empty after vite build"
ok "frontend built → dist/ ($(du -sh dist | cut -f1))"

# ── 3/6: tauri build ───────────────────────────────────────────
step 3 "tauri build --debug (backend + bundle)"
cd "$APP_DIR" || fail "directory $APP_DIR not found"
TAURI_BIN="./src/node_modules/.bin/tauri"
[ -x "$TAURI_BIN" ] || fail "tauri CLI not found — run pnpm install first"
"$TAURI_BIN" build --debug || fail "tauri build failed"
[ -d "$BUNDLE" ] || fail "bundle not created at $BUNDLE"
ok "bundle created: Minime.app"

# ── 4/6: bundle skills ─────────────────────────────────────────
step 4 "bundle-skills.sh"
SKILLS_SCRIPT="$APP_DIR/scripts/bundle-skills.sh"
[ -f "$SKILLS_SCRIPT" ] || fail "bundle-skills.sh not found"
bash "$SKILLS_SCRIPT" || fail "bundle-skills.sh failed"
# Verify: skills must exist inside the bundle
SKILLS_DIR="$BUNDLE/Contents/Resources/skills"
[ -d "$SKILLS_DIR" ] || fail "skills not bundled at $SKILLS_DIR"
SKILL_COUNT=$(ls "$SKILLS_DIR" | wc -l | tr -d ' ')
[ "$SKILL_COUNT" -gt 0 ] || fail "skills folder empty in bundle"
ok "skills bundled ($SKILL_COUNT items)"

# ── 5/6: restart app ───────────────────────────────────────────
step 5 "restart app"
pkill -9 -f "Minime.app" 2>/dev/null || true
rm -rf "$HOME/Library/Caches/$BUNDLE_ID" "$HOME/Library/WebKit/$BUNDLE_ID"
sleep 1
open "$BUNDLE" || fail "failed to open Minime.app"
sleep 8
pgrep -f "Minime.app" >/dev/null || fail "app not running after open"
ok "Minime.app running (PID $(pgrep -f 'Minime.app' | head -1))"

# ── 6/6: verifikasi akhir ──────────────────────────────────────
step 6 "final verification"
# 6a. Process alive
pgrep -f "Minime.app" >/dev/null || fail "Minime.app process not found"
# 6b. Skills still present after restart
[ -d "$SKILLS_DIR" ] || fail "skills missing from bundle after restart"
# 6c. Main binary exists
[ -f "$BUNDLE/Contents/MacOS/minime" ] || fail "main binary missing in bundle"
ok "all verifications passed"

echo ""
echo "🎉 DEPLOY DONE — Minime running with latest code."
echo "   Commit: $(cd "$MINIME_DIR" && git rev-parse --short HEAD)"
echo "   Skills: $SKILL_COUNT items bundled"
