#!/usr/bin/env bash
# Compare two mca.txt files (cycles column).
set -euo pipefail
A="$1"; B="$2"
paste <(tail -n+3 "$A/mca.txt") <(tail -n+3 "$B/mca.txt") \
  | awk '{x1=$4; x2=$15; a1=$9; a2=$20; if(x1!=x2||a1!=a2) printf "%-24s x86 %5d -> %5d (%+5d, %+4.0f%%)  arm %5d -> %5d (%+5d, %+4.0f%%)\n",$1,x1,x2,x2-x1,(x2-x1)*100.0/x1,a1,a2,a2-a1,(a2-a1)*100.0/a1}'
