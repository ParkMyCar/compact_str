#!/usr/bin/env bash
# Compare two asm-dump dirs and show instruction-count deltas.
# Usage: scripts/diff_asm.sh <before_dir> <after_dir>
set -euo pipefail
A="$1"; B="$2"
join -j1 <(tail -n+2 "$A/sizes.txt" | sort) <(tail -n+2 "$B/sizes.txt" | sort) \
  | awk '{dx=$4-$2; da=$5-$3; if(dx||da) printf "%-28s x86 %3d -> %3d (%+d)   arm %3d -> %3d (%+d)\n",$1,$2,$4,dx,$3,$5,da}'
