#!/usr/bin/env bash
set -euo pipefail

# 一键运行策略脚本 - 支持Docker容器和本地conda环境
# 使用方法:
#   ./run_strategy.sh safe_demo.py                 # 运行安全演示策略
#   ./run_strategy.sh small_trade_demo.py          # 运行小额交易策略  
#   ./run_strategy.sh compare_factors.py --help    # 带参数运行任意脚本
#   ./run_strategy.sh --local safe_demo.py         # 强制使用本地环境
#   ./run_strategy.sh --docker safe_demo.py        # 强制使用Docker环境

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 解析参数
USE_LOCAL=false
USE_DOCKER=false
STRATEGY_FILE=""
EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
  case $1 in
    --local)
      USE_LOCAL=true
      shift
      ;;
    --docker)
      USE_DOCKER=true
      shift
      ;;
    --help|-h)
      echo "xcrypto 策略运行脚本"
      echo "用法: $0 [--local|--docker] <策略文件> [策略参数...]"
      echo ""
      echo "选项:"
      echo "  --local   使用本地conda环境运行"
      echo "  --docker  使用Docker容器运行"
      echo "  --help    显示此帮助信息"
      echo ""
      echo "示例:"
      echo "  $0 safe_demo.py"
      echo "  $0 --local kline.py"
      echo "  $0 xcrypto/pyalgo/python/pyalgo/example/kline.py"
      exit 0
      ;;
    *)
      if [[ -z "$STRATEGY_FILE" ]]; then
        STRATEGY_FILE="$1"
      else
        EXTRA_ARGS+=("$1")
      fi
      shift
      ;;
  esac
done

# 默认策略文件
if [[ -z "$STRATEGY_FILE" ]]; then
  STRATEGY_FILE="safe_demo.py"
fi

echo "🔍 正在查找运行环境..."

# 查找运行中的 xcrypto 容器（新的镜像名）
CONTAINER_ID=$(docker ps --filter "ancestor=xcrypto-dev" --format "{{.ID}}" | head -1)

# 决定运行方式
RUN_IN_DOCKER=false
if [[ "$USE_DOCKER" == "true" ]]; then
  RUN_IN_DOCKER=true
elif [[ "$USE_LOCAL" == "true" ]]; then
  RUN_IN_DOCKER=false
elif [[ -n "$CONTAINER_ID" ]]; then
  echo "✅ 找到运行中的容器: $CONTAINER_ID"
  RUN_IN_DOCKER=true
else
  echo "❌ 没有找到运行中的 xcrypto 容器"
  echo "💡 将使用本地conda环境运行"
  RUN_IN_DOCKER=false
fi

# 查找策略文件
STRATEGY_PATH=""
if [[ -f "$SCRIPT_DIR/$STRATEGY_FILE" ]]; then
  STRATEGY_PATH="$SCRIPT_DIR/$STRATEGY_FILE"
elif [[ -f "$STRATEGY_FILE" ]]; then
  STRATEGY_PATH="$STRATEGY_FILE"
else
  echo "❌ 策略文件不存在: $STRATEGY_FILE"
  echo "📁 当前目录可用的策略文件:"
  find "$SCRIPT_DIR" -name "*.py" -type f | head -10
  exit 1
fi

echo "📋 策略文件: $STRATEGY_PATH"
if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
  echo "🧩 透传参数: ${EXTRA_ARGS[*]}"
fi

if [[ "$RUN_IN_DOCKER" == "true" ]]; then
  echo "🚀 正在容器中运行策略..."
  echo "=================================================="
  
  # 在容器中运行策略（使用conda环境）
  # 将宿主机路径转换为容器内路径
  CONTAINER_STRATEGY_PATH="/app/$STRATEGY_FILE"
  
  if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
    ARGS_STR="${EXTRA_ARGS[*]}"
    docker exec -it "$CONTAINER_ID" bash -c "
      echo '🐍 Python 版本:'
      source activate xcrypto
      python --version
      echo '📦 conda环境:'
      conda info --envs | grep '*'
      echo '📦 pyalgo 可用性检查:'
      python -c \"import importlib.util; print('✅ pyalgo 已安装' if importlib.util.find_spec('pyalgo') else '❌ pyalgo 未安装')\"
      echo ''
      echo '🎯 开始运行策略: $CONTAINER_STRATEGY_PATH'
      echo '💡 提示: 按 Ctrl+C 停止策略'
      echo ''
      cd /app
      python '$CONTAINER_STRATEGY_PATH' $ARGS_STR
    "
  else
    docker exec -it "$CONTAINER_ID" bash -c "
      echo '🐍 Python 版本:'
      source activate xcrypto
      python --version
      echo '📦 conda环境:'
      conda info --envs | grep '*'
      echo '📦 pyalgo 可用性检查:'
      python -c \"import importlib.util; print('✅ pyalgo 已安装' if importlib.util.find_spec('pyalgo') else '❌ pyalgo 未安装')\"
      echo ''
      echo '🎯 开始运行策略: $CONTAINER_STRATEGY_PATH'
      echo '💡 提示: 按 Ctrl+C 停止策略'
      echo ''
      cd /app
      python '$CONTAINER_STRATEGY_PATH'
    "
  fi
else
  echo "🚀 正在本地conda环境中运行策略..."
  echo "=================================================="
  
  # 检查conda环境
  if ! command -v conda >/dev/null 2>&1; then
    if [[ -f "$HOME/miniconda3/bin/conda" ]]; then
      export PATH="$HOME/miniconda3/bin:$PATH"
    elif [[ -f "$HOME/anaconda3/bin/conda" ]]; then
      export PATH="$HOME/anaconda3/bin:$PATH"
    else
      echo "❌ 未找到conda。请运行 ./run_xcrypto.sh --setup 配置环境"
      exit 1
    fi
  fi
  
  # 检查xcrypto环境是否存在
  if ! conda env list | grep -q "^xcrypto "; then
    echo "❌ conda环境 'xcrypto' 不存在"
    echo "请运行: ./run_xcrypto.sh --setup 来创建环境"
    exit 1
  fi
  
  echo "📦 激活conda环境: xcrypto"
  eval "$(conda shell.bash hook)"
  conda activate xcrypto
  
  echo "🐍 Python 版本: $(python --version)"
  echo "📦 pyalgo 可用性检查:"
  python -c "import importlib.util; print('✅ pyalgo 已安装' if importlib.util.find_spec('pyalgo') else '❌ pyalgo 未安装')"
  echo ""
  echo "🎯 开始运行策略: $STRATEGY_PATH"
  echo "💡 提示: 按 Ctrl+C 停止策略"
  echo ""
  
  # 运行策略
  cd "$SCRIPT_DIR"
  if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
    python "$STRATEGY_PATH" "${EXTRA_ARGS[@]}"
  else
    python "$STRATEGY_PATH"
  fi
fi

echo ""
echo "👋 策略运行结束"