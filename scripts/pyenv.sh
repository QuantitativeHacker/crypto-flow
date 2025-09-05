#!/usr/bin/env bash
set -euo pipefail

# 可在任意终端执行：
#   source scripts/pyenv.sh
# 作用：确保使用 xcrypto conda 环境，并为 pyo3 设置一致的编译/运行环境

# 启用 conda shell 钩子（尽量兼容 bash/zsh）
if command -v conda >/dev/null 2>&1; then
  if [[ -n "${BASH_VERSION:-}" ]]; then
    eval "$(conda shell.bash hook)" || true
  elif [[ -n "${ZSH_VERSION:-}" ]]; then
    eval "$(conda shell.zsh hook)" || true
  else
    eval "$(conda shell.bash hook)" || true
  fi
fi

# 激活 xcrypto 环境（不存在则忽略）
if command -v conda >/dev/null 2>&1; then
  conda activate xcrypto 2>/dev/null || true
fi

# 绑定 pyo3 的 Python 解释器
if command -v python >/dev/null 2>&1; then
  export PYO3_PYTHON="$(which python)"
fi

# 设置运行期动态库搜索路径（macOS）
if [[ -n "${CONDA_PREFIX:-}" ]]; then
  export DYLD_LIBRARY_PATH="${CONDA_PREFIX}/lib${DYLD_LIBRARY_PATH:+:${DYLD_LIBRARY_PATH}}"
  # 可选：写入 rpath，减少对 DYLD_LIBRARY_PATH 的依赖（仅当前会话）
  # export RUSTFLAGS="-C link-args=-Wl,-rpath,${CONDA_PREFIX}/lib${RUSTFLAGS:+ ${RUSTFLAGS}}"
fi

echo "[pyenv] PYO3_PYTHON=${PYO3_PYTHON:-unset}"
echo "[pyenv] CONDA_PREFIX=${CONDA_PREFIX:-unset}"
echo "[pyenv] DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:-unset}"

