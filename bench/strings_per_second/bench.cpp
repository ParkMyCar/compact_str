// The C++ side of Daniel Lemire's "How many strings can you create per second?", copied verbatim so
// we have a small-string-optimized baseline from the same machine. The author makes no copyright
// claim on it.
//
// https://github.com/lemire/Code-used-on-Daniel-Lemire-s-blog/blob/master/2026/09/24/bench.cpp

#include <chrono>
#include <cstdint>
#include <cstdio>
#include <string>
#include <vector>

std::vector<std::string> buf(1024);

__attribute__((noinline)) void from_int(uint64_t n) {
  for (uint64_t i = 0; i < n; i++) buf[i & 1023] = std::to_string(i);
}

int main() {
  const uint64_t N = 100'000'000;
  from_int(1'000'000); // warm up
  double best = 1e300;
  for (int r = 0; r < 5; r++) {
    auto t0 = std::chrono::steady_clock::now();
    from_int(N);
    auto t1 = std::chrono::steady_clock::now();
    double dt = std::chrono::duration<double, std::nano>(t1 - t0).count();
    if (dt < best) best = dt;
  }
  size_t total = 0;
  for (auto &s : buf) total += s.size();
  double ns = best / N;
  std::printf("C++ (%s)\n", __VERSION__);
  std::printf("%-24s %7.2f ns/string  %8.1f M/s   (check %zu)\n",
              "to_string(i)", ns, 1e3 / ns, total);
}
