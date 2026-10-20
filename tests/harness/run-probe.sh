#!/bin/sh
# run-probe.sh <varian> — jalankan semua tugas Code Builder dengan satu
# varian instruksi, di sandbox terpisah, lalu verifikasi mesin.
# Hasil: ~/harness-runs/<varian>/<tugas>/result.txt + verify.log
set -u
VARIAN="$1"
SRC="$HOME/harness-src/code-builder"
RUNS="$HOME/harness-runs/$VARIAN"
OC="$HOME/.opencode/bin/opencode"

for tugas in tugas1-fix tugas2-build tugas3-kualitas; do
  dir="$RUNS/$tugas"
  rm -rf "$dir"
  mkdir -p "$dir"
  cp -R "$SRC/$tugas/." "$dir/"
  cp "$HOME/harness-src/varian/$VARIAN.md" "$dir/AGENTS.md"
  cd "$dir" || exit 1
  start=$(date +%s)
  "$OC" run "$(cat PROMPT.txt)" > opencode.log 2>&1 &
  oc_pid=$!
  ( sleep 600; kill -9 "$oc_pid" 2>/dev/null ) &
  wd_pid=$!
  wait "$oc_pid"
  oc_exit=$?
  kill "$wd_pid" 2>/dev/null
  sh VERIFY.sh > verify.log 2>&1
  v_exit=$?
  dur=$(( $(date +%s) - start ))
  printf "varian=%s tugas=%s opencode_exit=%s verify_exit=%s durasi_detik=%s\n" \
    "$VARIAN" "$tugas" "$oc_exit" "$v_exit" "$dur" > result.txt
  echo "selesai: $VARIAN/$tugas verify_exit=$v_exit (${dur}s)"
done
touch "$RUNS/DONE"
echo "SEMUA SELESAI: $VARIAN"
