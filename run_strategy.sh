#!/usr/bin/env bash
set -euo pipefail

# 一键在 Docker 容器中运行策略脚本
# 使用方法:
#   ./run_strategy.sh safe_demo.py                 # 运行安全演示策略
#   ./run_strategy.sh small_trade_demo.py          # 运行小额交易策略
#   ./run_strategy.sh compare_factors.py --help    # 带参数运行任意脚本

STRATEGY_FILE="${1:-safe_demo.py}"
# 透传给脚本的额外参数（从第 2 个参数开始）
EXTRA_ARGS="${*:2}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔍 正在查找运行中的 xcrypto 容器..."

# 查找运行中的 rust 容器
CONTAINER_ID=$(docker ps --filter "ancestor=rust:1.80-bookworm" --format "{{.ID}}" | head -1)

if [[ -z "$CONTAINER_ID" ]]; then
    echo "❌ 没有找到运行中的 xcrypto 容器"
    echo "💡 请先运行: ./run_xcrypto.sh"
    exit 1
fi

echo "✅ 找到容器: $CONTAINER_ID"

# 检查策略文件是否存在
if [[ ! -f "$SCRIPT_DIR/$STRATEGY_FILE" ]]; then
    echo "❌ 策略文件不存在: $STRATEGY_FILE"
    echo "📁 可用的策略文件:"
    ls -1 "$SCRIPT_DIR"/*.py 2>/dev/null || echo "   (没有找到 .py 文件)"
    exit 1
fi

echo "📋 策略文件: $STRATEGY_FILE"
if [[ -n "$EXTRA_ARGS" ]]; then
  echo "🧩 透传参数: $EXTRA_ARGS"
fi
echo "🚀 正在容器中运行策略..."
echo "=================================================="

# 在容器中运行策略
# 注意：此处使用双引号，$EXTRA_ARGS 会在宿主侧扩展，再在容器中作为参数传入
#       如果参数中包含需要特殊转义的字符，请改用 docker exec -it ... python3 手动运行

docker exec -it "$CONTAINER_ID" bash -c "
    echo '🐍 Python 版本:'
    python3 --version
    echo '📦 激活 pyalgo 虚拟环境...'
    cd /workspace/pyalgo && source .venv/bin/activate
    echo '📦 pyalgo 可用性检查:'
    python3 -c \"import importlib.util; print('✅ pyalgo 已安装' if importlib.util.find_spec('pyalgo') else '❌ pyalgo 未安装')\"
    echo ''
    echo '🎯 开始运行策略: $STRATEGY_FILE'
    echo '💡 提示: 按 Ctrl+C 停止策略'
    echo ''
    cd /strategies
    python3 '$STRATEGY_FILE' ${EXTRA_ARGS}
"

echo ""
echo "👋 策略运行结束"