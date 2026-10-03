#!/usr/bin/env bash
#
# Builds and runs the benchmark, plus the C++ baselines, and prints enough about the machine and the
# toolchains to make sense of the numbers later.
#
# Usage: ./run.sh [CPU]
#
# On Linux everything is pinned to a single CPU, which defaults to the last one we're allowed to run
# on. Pick one by hand if that one is busy, shares a core with something that is, or if it's an
# efficiency core, which hybrid CPUs tend to list last.

set -euo pipefail
cd "$(dirname "$0")"

section() { printf '\n## %s\n\n' "$1"; }
row() { printf '%-24s %s\n' "$1:" "$2"; }
# Prints the contents of a file, if it exists.
show() { if [ -r "$2" ]; then row "$1" "$(cat "$2")"; fi; }

CPU=${1:-}
PIN=""
if command -v taskset >/dev/null; then
  CPU=${CPU:-$(taskset -cp $$ | sed -n 's/.*affinity list:.*[^0-9]\([0-9][0-9]*\)$/\1/p')}
  if [ -n "$CPU" ]; then PIN="taskset -c $CPU"; fi
fi

# Takes precedence over any flags from a cargo config, so what we print below is what gets used.
unset CARGO_ENCODED_RUSTFLAGS
export RUSTFLAGS="${RUSTFLAGS:-}"

# $1 is the name of the binary, the rest is the compiler invocation.
cpp_build() {
  local bin=target/$1
  shift
  rm -f "$bin"
  if command -v "$1" >/dev/null; then "$@" -O3 -std=c++20 -o "$bin" bench.cpp 2>/dev/null || true; fi
}

# $1 is the name of the binary, $2 describes it.
cpp_run() {
  section "C++, $2"
  if [ -x "target/$1" ]; then
    $PIN "./target/$1"
  else
    echo "skipped, not installed or failed to compile"
  fi
}

# Build everything upfront, so the machine is idle again by the time we start to measure.
cargo build --quiet --release
cargo build --quiet --profile release-lto
mkdir -p target
if [ "$(uname -s)" = "Darwin" ]; then
  # `g++` is just another name for Apple's `clang++`, which always uses libc++.
  cpp_build clang_libcxx clang++
else
  cpp_build gcc_libstdcxx g++
  cpp_build clang_libstdcxx clang++ -stdlib=libstdc++
  cpp_build clang_libcxx clang++ -stdlib=libc++
fi

section "Machine"
if [ "$(uname -s)" = "Darwin" ]; then
  row "Model name" "$(sysctl -n machdep.cpu.brand_string)"
  row "macOS" "$(sw_vers -productVersion)"
else
  if command -v lscpu >/dev/null; then
    LC_ALL=C lscpu |
      grep -E '^(Architecture|Vendor ID|Model name|CPU family|Model|Stepping|Hypervisor vendor|L1d cache|L2 cache|L3 cache):' |
      sed -E 's/:[[:space:]]+/:/' |
      while IFS=: read -r key value; do row "$key" "$value"; done
  fi
  row "libc" "$(getconf GNU_LIBC_VERSION 2>/dev/null || echo unknown)"
  row "Pinned to CPU" "${CPU:-no}"
  # Everything that tends to make numbers wobble. Virtual machines generally don't expose these.
  show "SMT active" /sys/devices/system/cpu/smt/active
  show "Governor" "/sys/devices/system/cpu/cpu${CPU:-0}/cpufreq/scaling_governor"
  show "Boost" /sys/devices/system/cpu/cpufreq/boost
  show "No turbo" /sys/devices/system/cpu/intel_pstate/no_turbo
  show "Load average" /proc/loadavg
fi

section "Versions"
rustc --version
echo "RUSTFLAGS=$RUSTFLAGS"
{ cargo pkgid itoa | sed 's/.*#//; s/@/ /'; } || true
echo "compact_str $(git describe --always --dirty --abbrev=10)"

# Everything is built already. Going through `cargo run` means we don't need to know where it put
# the binaries, and they inherit the affinity.
section "Rust, release"
$PIN cargo run --quiet --release

section "Rust, release + fat LTO + codegen-units=1"
$PIN cargo run --quiet --profile release-lto

if [ "$(uname -s)" != "Darwin" ]; then
  cpp_run gcc_libstdcxx "g++ with libstdc++"
  cpp_run clang_libstdcxx "clang++ with libstdc++"
fi
cpp_run clang_libcxx "clang++ with libc++"
