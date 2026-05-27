#!/usr/bin/env bash
# Emit a single .s file per arch from the asm-harness rlib (no link step), then
# split it into one file per cs_* symbol and print an instruction-count table.
# Usage: scripts/dump_asm.sh <output_dir>
set -euo pipefail
cd "$(dirname "$0")/../asm-harness"
OUT="${1:-/tmp/cs_asm}"
mkdir -p "$OUT/x86" "$OUT/arm"

emit() {
  local arch="$1" rustflags="$2" target="$3"
  local sfile="$OUT/$arch/_all.s"
  RUSTFLAGS="$rustflags" cargo rustc --release ${target:+--target "$target"} -- \
    --emit asm -C llvm-args=-x86-asm-syntax=intel -C codegen-units=1 -C opt-level=3 \
    2>&1 | grep -E "^error" || true
  local tdir="target/${target:+$target/}release/deps"
  cp "$(ls -t "$tdir"/asm_harness-*.s | head -1)" "$sfile"
  awk -v dir="$OUT/$arch" '
    /^cs_[a-z_0-9]+:/ { if (f) close(f); name=$1; sub(/:$/,"",name); f=dir"/"name".s" }
    f { print > f }
    /^\.Lfunc_end/ { if (f) { close(f); f="" } }
  ' "$sfile"
  # resolve `cs_X = cs_Y` aliases
  awk -v dir="$OUT/$arch" -F'[ =]+' '/^cs_.* = cs_/ { system("cp "dir"/"$2".s "dir"/"$1".s") }' "$sfile"
}

emit x86 "-C target-cpu=x86-64-v3" ""
emit arm "" "aarch64-unknown-linux-gnu"

SYMS=$(grep -oP '(?<=pub fn )cs_\w+' src/lib.rs | sort)
{
  printf "%-28s %8s %8s\n" "symbol" "x86_ins" "arm_ins"
  for s in $SYMS; do
    x=$(grep -cP '^\t[a-z]' "$OUT/x86/$s.s" 2>/dev/null) || x=0
    a=$(grep -cP '^\t[a-z]' "$OUT/arm/$s.s" 2>/dev/null) || a=0
    printf "%-28s %8s %8s\n" "$s" "$x" "$a"
  done
} | tee "$OUT/sizes.txt"
