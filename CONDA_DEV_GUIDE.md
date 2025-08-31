# xcrypto Conda 开发指南

## 概述

在local模式下，xcrypto项目完全使用conda来管理Python、Rust、以及所有系统依赖，避免污染主机环境。

## 🚀 快速开始

### 1. 一键环境配置
```bash
# 首次使用 - 自动配置conda环境
./run_xcrypto.sh --setup
```

这个命令会：
- 检测并配置conda路径
- 创建名为 `xcrypto` 的conda环境
- 安装所有依赖（Python、Rust、系统库等）

### 2. 日常开发使用
```bash
# 使用conda环境运行项目
./run_xcrypto.sh spot --local
./run_xcrypto.sh usdt --local
```

## 🔧 Conda环境详解

### 环境配置文件
项目使用 `environment.yml` 定义conda环境：

```yaml
name: xcrypto
channels:
  - conda-forge
  - defaults
dependencies:
  - python=3.11
  - rust
  - maturin
  - pip
  - pkg-config
  - openssl
  - sqlite
  - ca-certificates
  - pip:
    - -r requirements.txt
```

### 手动环境管理

```bash
# 激活环境（开发时使用）
conda activate xcrypto

# 查看环境信息
conda info --envs
conda list

# 更新环境
conda env update -n xcrypto -f environment.yml

# 删除环境（重新开始）
conda env remove -n xcrypto
```

## 🛠 开发工作流

### 方式1: 通过脚本开发（推荐）
```bash
# 修改代码后，直接运行
./run_xcrypto.sh spot --local  # 自动编译和运行
```

### 方式2: 手动激活环境开发
```bash
# 激活conda环境
conda activate xcrypto

# 手动编译Rust代码
cd xcrypto/binance/spot
cargo build --release

# 编译Python扩展
cd ../../pyalgo
maturin develop --release

# 运行程序
cd ..
./target/release/spot -c=spot.json -l=info
```

## 📁 开发环境结构

```
xcrypto/
├── environment.yml          # conda环境定义
├── requirements.txt         # pip依赖
├── run_xcrypto.sh          # 一键运行脚本
├── spot.json           # 配置文件
├── private_key.pem     # 私钥文件
├── binance/            # Rust代码
├── pyalgo/             # Python扩展
└── target/             # 编译输出
```

## 🔍 调试和开发技巧

### 1. 检查环境状态
```bash
# 快速检查
./run_xcrypto.sh --local

# 详细环境信息
conda activate xcrypto
which python
which cargo
conda list | grep -E "(rust|python|maturin)"
```

### 2. 增量编译
```bash
conda activate xcrypto
cd xcrypto/binance/spot
cargo build --release  # 只编译changed的部分
```

### 3. Python扩展开发
```bash
conda activate xcrypto
cd xcrypto/pyalgo
maturin develop --release  # 重新编译Python扩展
python -c "import pyalgo; print(pyalgo.__file__)"  # 验证
```

### 4. 依赖管理
```bash
# 添加新的conda包
conda install -n xcrypto new-package

# 添加新的pip包
conda activate xcrypto
pip install new-package

# 导出当前环境（更新environment.yml）
conda env export -n xcrypto > environment.yml
```

## 🚨 常见问题

### Q: conda命令找不到
**A:** 脚本会自动检测以下位置的conda：
- 系统PATH中的conda
- `$HOME/miniconda3/bin/conda`
- `$HOME/anaconda3/bin/conda`

如果都没找到，请手动安装miniconda。

### Q: 环境创建失败
**A:** 
```bash
# 清理并重新创建
conda env remove -n xcrypto
./run_xcrypto.sh --setup
```

### Q: Rust编译失败
**A:** 确保conda环境中包含必要的系统库：
```bash
conda activate xcrypto
conda install pkg-config openssl sqlite
```

### Q: 想要使用不同的Python版本
**A:** 修改 `environment.yml` 中的Python版本，然后重新创建环境：
```bash
conda env remove -n xcrypto
./run_xcrypto.sh --setup
```

## 🎯 性能优化

### 1. 并行编译
```bash
conda activate xcrypto
export CARGO_BUILD_JOBS=$(nproc)  # 使用所有CPU核心
```

### 2. 编译缓存
conda环境会自动缓存编译结果，避免重复编译。

### 3. 镜像加速
```bash
# 配置conda镜像（可选）
conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main
conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/free
conda config --add channels https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge
```

## 📝 总结

使用conda进行开发的优势：
- ✅ **完全隔离**：不污染主机Python/Rust环境
- ✅ **一键配置**：自动安装所有依赖
- ✅ **跨平台**：Windows/macOS/Linux统一体验
- ✅ **版本固定**：确保团队环境一致
- ✅ **易于维护**：通过environment.yml版本控制

这种方式特别适合多人协作和CI/CD环境！
