#!/usr/bin/env bash
set -euo pipefail

# 一键配置和运行xcrypto项目 - 使用miniconda环境管理
# 默认：使用Docker容器隔离环境（推荐）
# 备选：使用本地miniconda环境
# 用法:
#   ./run_xcrypto.sh [spot|usdt] [--setup|--docker|--local|-f]
# 示例:
#   ./run_xcrypto.sh                    # 默认spot模式，自动选择Docker或本地
#   ./run_xcrypto.sh usdt               # USDT期货模式
#   ./run_xcrypto.sh spot --setup       # 配置本地miniconda环境
#   ./run_xcrypto.sh spot --docker      # 强制使用Docker
#   ./run_xcrypto.sh spot --local       # 强制使用本地环境
#   ./run_xcrypto.sh spot -f            # 强制重新构建Docker镜像并运行
#   ./run_xcrypto.sh usdt --docker -f   # USDT模式，强制重新构建

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
WORKSPACE_DIR="$PROJECT_ROOT"

# 解析参数
MODE="spot"
FORCE_MODE=""
FORCE_REBUILD=false

for arg in "$@"; do
  case $arg in
    spot|usdt)
      MODE="$arg"
      ;;
    --setup)
      FORCE_MODE="setup"
      ;;
    --docker)
      FORCE_MODE="docker"
      ;;
    --local)
      FORCE_MODE="local"
      ;;
    -f|--force)
      FORCE_REBUILD=true
      ;;
    --help|-h)
      echo "xcrypto 一键运行脚本"
      echo "用法: $0 [spot|usdt] [--setup|--docker|--local|-f|--help]"
      echo ""
      echo "模式:"
      echo "  spot    现货交易模式 (默认)"
      echo "  usdt    USDT期货模式"
      echo ""
      echo "选项:"
      echo "  --setup   配置本地miniconda环境"
      echo "  --docker  强制使用Docker容器"
      echo "  --local   强制使用本地miniconda环境"
      echo "  -f, --force  强制重新构建Docker镜像"
      echo "  --help    显示此帮助信息"
      exit 0
      ;;
    *)
      echo "[错误] 未知参数: $arg" >&2
      echo "使用 --help 查看用法" >&2
      exit 1
      ;;
  esac
done

if [[ "$MODE" != "spot" && "$MODE" != "usdt" ]]; then
  echo "[错误] 未知模式: $MODE (期望 'spot' 或 'usdt')" >&2
  exit 1
fi

CONFIG_JSON="$PROJECT_ROOT/${MODE}.json"
if [[ ! -f "$CONFIG_JSON" ]]; then
  echo "[ERROR] Missing config file: $CONFIG_JSON" >&2
  exit 1
fi

PEM_FILE="$PROJECT_ROOT/private_key.pem"
if [[ ! -f "$PEM_FILE" ]]; then
  echo "[ERROR] Missing private key: $PEM_FILE" >&2
  exit 1
fi

CONFIG_BASENAME="$(basename "$CONFIG_JSON")"
ENV_NAME="xcrypto"

# 检查conda是否可用
check_conda() {
  if command -v conda >/dev/null 2>&1; then
    return 0
  elif [[ -f "$HOME/miniconda3/bin/conda" ]]; then
    export PATH="$HOME/miniconda3/bin:$PATH"
    return 0
  elif [[ -f "$HOME/anaconda3/bin/conda" ]]; then
    export PATH="$HOME/anaconda3/bin:$PATH"
    return 0
  else
    return 1
  fi
}

# 设置本地miniconda环境
setup_local_environment() {
  echo "[信息] 配置本地miniconda环境..."
  
  if ! check_conda; then
    echo "[错误] 未找到conda。请先安装miniconda或anaconda："
    echo "  下载地址: https://docs.conda.io/en/latest/miniconda.html"
    echo "  或运行: wget https://repo.anaconda.com/miniconda/Miniconda3-latest-$(uname -s)-$(uname -m).sh"
    echo "         bash Miniconda3-latest-$(uname -s)-$(uname -m).sh"
    exit 1
  fi

  echo "[信息] 检测到conda: $(command -v conda)"
  
  # 检查环境是否已存在
  if conda env list | grep -q "^$ENV_NAME "; then
    echo "[信息] conda环境 '$ENV_NAME' 已存在"
    read -p "是否重新创建环境? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      echo "[信息] 删除现有环境..."
      conda env remove -n "$ENV_NAME" -y
    else
      echo "[信息] 使用现有环境"
      return 0
    fi
  fi

  # 创建conda环境
  echo "[信息] 创建conda环境 '$ENV_NAME'..."
  if [[ -f "$SCRIPT_DIR/environment.yml" ]]; then
    conda env create -n "$ENV_NAME" -f "$SCRIPT_DIR/environment.yml"
  else
    echo "[错误] 未找到environment.yml文件: $SCRIPT_DIR/environment.yml"
    exit 1
  fi

  echo "[成功] 本地miniconda环境配置完成!"
  echo "[信息] 激活环境: conda activate $ENV_NAME"
}

run_in_docker() {
  echo "[信息] 使用Docker容器运行 (推荐，环境隔离)"
  
  IMAGE_NAME="xcrypto-dev:latest"
  
  # 检查是否需要强制重新构建
  if [[ "$FORCE_REBUILD" == true ]]; then
    echo "[信息] 强制重新构建Docker镜像..."
    # 删除现有镜像（如果存在）
    if docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
      echo "[信息] 删除现有镜像..."
      docker rmi "$IMAGE_NAME" || true
    fi
  fi
  
  # 检查Docker镜像是否存在，如果不存在则构建
  if ! docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
    echo "[信息] Docker镜像不存在，正在构建..."
    if [[ ! -f "$SCRIPT_DIR/Dockerfile.dev" ]]; then
      echo "[错误] 未找到Dockerfile.dev: $SCRIPT_DIR/Dockerfile.dev"
      exit 1
    fi
    
    echo "[信息] 开始构建Docker镜像，这可能需要几分钟..."
    docker build -f "$SCRIPT_DIR/Dockerfile.dev" -t "$IMAGE_NAME" "$SCRIPT_DIR" || {
      echo "[错误] Docker镜像构建失败"
      exit 1
    }
    echo "[成功] Docker镜像构建完成!"
  elif [[ "$FORCE_REBUILD" != true ]]; then
    echo "[信息] 使用现有Docker镜像 (使用 -f 强制重新构建)"
  fi
  
  echo "[信息] 启动Docker容器，端口映射 8111:8111"
  # 映射8111端口用于WS服务，挂载工作目录，使用缓存卷避免重复编译
  docker run --rm -it \
    -p 8111:8111 \
    -v "$SCRIPT_DIR":/app \
    -v xcrypto-cargo-cache:/usr/local/cargo/registry \
    -v xcrypto-target-cache:/app/target \
    -w /app \
    "$IMAGE_NAME" \
    bash -c "set -e; \
      source activate $ENV_NAME; \
      echo '[信息] 使用conda环境: $ENV_NAME'; \
      conda info --envs; \
      # 构建Rust二进制文件
      cd binance/$MODE && cargo build -r; \
      # 构建Python策略包
      cd ../pyalgo && maturin develop -r; \
      # 安装Python依赖
      python -m pip install -r /app/requirements.txt || true; \
      # 从项目根目录运行，以便发现配置文件
      cd /app && \
      echo '[信息] 启动 $MODE 服务器，地址: ws://localhost:8111' && \
      ./target/release/$MODE -c=$CONFIG_BASENAME -l=info"
}

run_locally() {
  echo "[信息] 使用本地miniconda环境运行"
  
  # 检查conda环境
  if ! check_conda; then
    echo "[错误] 未找到conda。请运行以下命令之一："
    echo "  $0 --setup    # 自动安装miniconda环境"
    echo "  或手动安装: https://docs.conda.io/en/latest/miniconda.html"
    exit 1
  fi

  # 检查xcrypto环境是否存在
  if ! conda env list | grep -q "^$ENV_NAME "; then
    echo "[错误] conda环境 '$ENV_NAME' 不存在"
    echo "请运行: $0 --setup 来创建环境"
    exit 1
  fi

  echo "[信息] 激活conda环境: $ENV_NAME"
  
  # 使用conda环境中的工具
  eval "$(conda shell.bash hook)"
  conda activate "$ENV_NAME"
  
  # 验证工具可用性
  if ! command -v cargo >/dev/null 2>&1; then
    echo "[错误] cargo未在conda环境中找到，请检查environment.yml配置"
    exit 1
  fi
  
  if ! command -v python >/dev/null 2>&1; then
    echo "[错误] python未在conda环境中找到"
    exit 1
  fi

  echo "[信息] 使用工具版本:"
  echo "  Python: $(python --version)"
  echo "  Cargo: $(cargo --version)"
  echo "  Conda环境: $(conda info --envs | grep '*' | awk '{print $1}')"

  # 构建Rust二进制文件
  echo "[信息] 构建Rust二进制文件..."
  pushd "$PROJECT_ROOT/binance/$MODE" >/dev/null
  cargo build -r
  popd >/dev/null

  # 构建Python策略包
  echo "[信息] 构建Python策略包..."
  pushd "$PROJECT_ROOT/pyalgo" >/dev/null
  python -m pip install --upgrade pip
  maturin develop -r
  # 安装Python依赖
  if [[ -f "$PROJECT_ROOT/requirements.txt" ]]; then
    python -m pip install -r "$PROJECT_ROOT/requirements.txt" || true
  fi
  popd >/dev/null

  # 从项目根目录运行，确保private_key.pem路径正确
  echo "[信息] 启动 $MODE 服务器，地址: ws://localhost:8111"
  cd "$PROJECT_ROOT"
  "./target/release/$MODE" -c="$CONFIG_BASENAME" -l=info
}

# 主执行逻辑
case "$FORCE_MODE" in
  setup)
    setup_local_environment
    echo ""
    echo "[信息] 环境配置完成！现在可以运行："
    echo "  $0 $MODE --local    # 使用本地环境运行"
    echo "  $0 $MODE --docker   # 使用Docker容器运行"
    ;;
  docker)
    if ! command -v docker >/dev/null 2>&1; then
      echo "[错误] Docker未安装。请先安装Docker或使用 --local 选项"
      exit 1
    fi
    run_in_docker
    ;;
  local)
    run_locally
    ;;
  *)
    # 自动选择运行方式
    if command -v docker >/dev/null 2>&1; then
      echo "[信息] 检测到Docker，使用容器运行（推荐）"
      echo "[信息] 如需使用本地环境，请添加 --local 选项"
      run_in_docker
    else
      echo "[信息] 未检测到Docker，使用本地miniconda环境"
      run_locally
    fi
    ;;
esac