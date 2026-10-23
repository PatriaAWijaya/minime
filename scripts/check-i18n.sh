#!/bin/bash
# check-i18n.sh — Verify every Indonesian (id) key has EN and ZH counterparts.
#
# Fails (non-zero exit) when any id key is missing in en or zh, listing
# the missing keys. Run before commit:
#   bash scripts/check-i18n.sh
#
# Convention: id is the source of truth; en/zh must cover all id keys.

set -euo pipefail

I18N_FILE="${1:-app/src/src/lib/i18n.ts}"
[ -f "$I18N_FILE" ] || { echo "❌ i18n file not found: $I18N_FILE" >&2; exit 1; }

python3 - "$I18N_FILE" <<'PYEOF'
import re, sys

path = sys.argv[1]
src = open(path, encoding="utf-8").read()

def extract(block_name: str) -> set[str]:
    # Match: const <name>: Dict = { ... };  (non-greedy to the closing "};")
    m = re.search(rf"const {block_name}: Dict = \{{(.*?)\n\}};", src, re.DOTALL)
    if not m:
        print(f"ERROR: dict '{block_name}' not found in {path}", file=sys.stderr)
        sys.exit(2)
    keys = set(re.findall(r'^\s*"([^"]+)"\s*:', m.group(1), re.MULTILINE))
    return keys

id_keys = extract("id")
en_keys = extract("en")
zh_keys = extract("zh")

print(f"Keys: id={len(id_keys)} en={len(en_keys)} zh={len(zh_keys)}")

missing_en = sorted(id_keys - en_keys)
missing_zh = sorted(id_keys - zh_keys)

failed = False
if missing_en:
    failed = True
    print(f"\n❌ Missing in EN ({len(missing_en)}):", file=sys.stderr)
    for k in missing_en:
        print(f"   - {k}", file=sys.stderr)
if missing_zh:
    failed = True
    print(f"\n❌ Missing in ZH ({len(missing_zh)}):", file=sys.stderr)
    for k in missing_zh:
        print(f"   - {k}", file=sys.stderr)

# Also warn about orphan keys (in en/zh but not in id) — not fatal.
orphan_en = sorted(en_keys - id_keys)
orphan_zh = sorted(zh_keys - id_keys)
if orphan_en or orphan_zh:
    print("\n⚠️  Orphan keys (not in id, kept for reference):")
    for k in orphan_en:
        print(f"   en: {k}")
    for k in orphan_zh:
        print(f"   zh: {k}")

if failed:
    print("\n❌ i18n check FAILED — add the missing translations before commit.", file=sys.stderr)
    sys.exit(1)
print("\n✅ i18n check passed — all id keys have en/zh counterparts.")
PYEOF
