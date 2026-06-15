#!/usr/bin/env bash
# 诚实对比 uvr（原生二进制）与 pak（在 R 里）解析依赖的「调用延迟」。
# 不用 hyperfine（避免 brew 安装）：用 perl 的 Time::HiRes 取亚毫秒，跑 N 次取最小值。
#
# 用法: scripts/bench.sh [package] [N]
set -uo pipefail

PKG="${1:-dotenv}"
N="${2:-10}"
REPO="https://gaborcsardi.r-universe.dev"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKGS="/tmp/uvr_bench_PACKAGES"

# 真实 PACKAGES 抓到本地，供 uvr 离线解析；release 构建 uvr
curl -s "$REPO/src/contrib/PACKAGES" -o "$PKGS"
cargo build --release --quiet --manifest-path "$ROOT/Cargo.toml"
UVR="$ROOT/target/release/uvr"

# 跑 "$@" N 次，打印最小耗时（毫秒）。
# 用 perl 把毫秒写到独立文件，命令自身的 stdout/stderr 全部丢弃，避免污染计时值。
min_ms() {
  local vals=() i msfile
  msfile="$(mktemp)"
  for ((i = 0; i < N; i++)); do
    MSFILE="$msfile" perl -MTime::HiRes=time -e \
      'open(my $f, ">", $ENV{MSFILE}); my $s = time; system(@ARGV); printf $f "%.1f", (time - $s) * 1000;' \
      -- "$@" >/dev/null 2>&1
    vals+=("$(cat "$msfile")")
  done
  rm -f "$msfile"
  printf '%s\n' "${vals[@]}" | sort -n | head -1
}

# 预热 pak 的元数据缓存（不计时）
Rscript -e "invisible(pak::pkg_deps('$PKG'))" >/dev/null 2>&1 || true

UVR_MS=$(min_ms "$UVR" lock "$PKGS" "$PKG")
PAK_MS=$(min_ms Rscript -e "invisible(pak::pkg_deps('$PKG'))")
R_MS=$(min_ms Rscript -e "invisible(1)")

echo "# uvr vs pak — 解析依赖的调用延迟 / resolve invocation latency"
echo "package=$PKG  runs=$N  (min, ms)"
echo
echo "| 工具 / tool | min ms |"
echo "|---|---|"
echo "| uvr (原生 release / native) | $UVR_MS |"
echo "| pak (在 R 里 / in R) | $PAK_MS |"
echo "| 仅 R 启动 (Rscript -e 1) | $R_MS |"
