#!/usr/bin/env bash
# Run llvm-mca over each cs_* asm snippet and emit a cycle/uop summary table.
# Usage: scripts/mca.sh <asm_dir>   (e.g. /tmp/cs_asm/baseline)
set -euo pipefail
DIR="${1:-/tmp/cs_asm/baseline}"

mca_one() {
  local f="$1" triple="$2" cpu="$3"
  # Strip directives mca chokes on; keep only label + instructions
  grep -P '^\t[a-z]' "$f" \
    | llvm-mca -mtriple="$triple" -mcpu="$cpu" --x86-asm-syntax=intel -iterations=100 2>/dev/null \
    | awk '/^Instructions:/{i=$2} /^Total Cycles:/{c=$3} /^Total uOps:/{u=$3} /^Block RThroughput:/{rt=$3} END{printf "%s %s %s %s", i,c,u,rt}'
}

{
  printf "%-24s | %22s | %22s\n" "" "x86 sapphirerapids" "arm neoverse-v1"
  printf "%-24s | %5s %6s %5s %4s | %5s %6s %5s %4s\n" "symbol" "ins" "cyc" "uops" "rTP" "ins" "cyc" "uops" "rTP"
  for fx in "$DIR"/x86/cs_*.s; do
    s=$(basename "$fx" .s)
    fa="$DIR/arm/$s.s"
    read -r xi xc xu xt <<<"$(mca_one "$fx" x86_64 sapphirerapids)"
    read -r ai ac au at <<<"$(mca_one "$fa" aarch64 neoverse-v1)"
    printf "%-24s | %5s %6s %5s %4s | %5s %6s %5s %4s\n" "$s" "${xi:--}" "${xc:--}" "${xu:--}" "${xt:--}" "${ai:--}" "${ac:--}" "${au:--}" "${at:--}"
  done
} | tee "$DIR/mca.txt"
