#!/usr/bin/env bash
set -euo pipefail

# 作用：在 macOS/conda 环境下一键运行 pyalgo/src/bin/gen_stub.rs
# - 自动激活 conda（若可用）并绑定 PyO3 到当前 python
# - 自动设置 CC/CXX/SDKROOT（修复 “找不到 cc/clang”）
# 用法：
#   ./scripts/gen_stub.sh            # release 运行

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# 载入 Python/conda 环境（若存在）
if [[ -f "${PROJECT_ROOT}/scripts/pyenv.sh" ]]; then
  # shellcheck disable=SC1090
  source "${PROJECT_ROOT}/scripts/pyenv.sh"
fi

# macOS: 确保 CC/CXX/SDKROOT 可用
if ! command -v cc >/dev/null 2>&1; then
  if command -v xcrun >/dev/null 2>&1; then
    export CC="${CC:-$(xcrun -f clang)}"
    export CXX="${CXX:-$(xcrun -f clang++)}"
    export SDKROOT="${SDKROOT:-$(xcrun --sdk macosx --show-sdk-path 2>/dev/null || true)}"
  else
    echo "[错误] 未找到 cc/clang。请先安装 Xcode Command Line Tools: xcode-select --install" >&2
    exit 1
  fi
else
  export CC="${CC:-$(command -v cc)}"
  if [[ -z "${CXX:-}" ]]; then
    if command -v c++ >/dev/null 2>&1; then
      export CXX="$(command -v c++)"
    elif command -v xcrun >/dev/null 2>&1; then
      export CXX="$(xcrun -f clang++)"
    fi
  fi
  if command -v xcrun >/dev/null 2>&1 && [[ -z "${SDKROOT:-}" ]]; then
    export SDKROOT="$(xcrun --sdk macosx --show-sdk-path 2>/dev/null || true)"
  fi
fi

echo "[info] Python : $(python --version 2>/dev/null || echo N/A)"
echo "[info] Rust   : $(rustc --version 2>/dev/null || echo N/A)"
echo "[info] Cargo  : $(cargo --version 2>/dev/null || echo N/A)"
echo "[info] CC     : ${CC:-unset}"
echo "[info] CXX    : ${CXX:-unset}"
echo "[info] SDKROOT: ${SDKROOT:-unset}"

# 避免在本地构建 Python 扩展时要求链接 libpython，使用动态查找
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-undefined -C link-arg=dynamic_lookup"
echo "[info] RUSTFLAGS: ${RUSTFLAGS}"

# 切换到 pyalgo 并运行生成器
cd "${PROJECT_ROOT}/pyalgo"

PROFILE_FLAG="--release"

exec cargo run ${PROFILE_FLAG} --bin gen_stub


