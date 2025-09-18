# cryptoflow Conda 开发指南

## 概述

在local模式下，cryptoflow项目完全使用conda来管理Python、Rust、以及所有系统依赖，避免污染主机环境。

## 快速开始

```bash
# 首次使用 - 自动配置conda环境
./run_cryptoflow.sh --setup
```

这个命令会检测并配置conda路径，创建名为 `cryptoflow` 的conda环境，安装所有依赖（Python、Rust、系统库等）。
可以通过下面的命令激活环境
```bash
conda activate cryptoflow
```

## Conda环境详解


项目使用 `environment.yml` 定义conda环境：

#### 激活环境
```bash
conda activate cryptoflow
```

#### 查看环境信息
```bash
conda info --envs
conda list
```

#### 更新环境
```bash
conda env update -n cryptoflow -f environment.yml
```

#### 删除环境（重新开始）
```bash
conda env remove -n cryptoflow
```

## 🛠 开发工作流

### 方式1: 通过脚本开发（推荐）
```bash
# 修改代码后，直接运行
./run_cryptoflow.sh spot --local  # 自动编译和运行
```

### 方式2: 手动激活环境开发
```bash
# 激活conda环境
conda activate cryptoflow

# 手动编译Rust代码
cd cryptoflow/binance/spot
cargo build --release

# 编译Python扩展
cd ../../pyalgo
maturin develop --release

# 运行程序
cd ..
./target/release/spot -c=spot.json -l=info
```

## 开发环境结构

```
cryptoflow/
├── environment.yml          # conda环境定义（引用requirements.txt）
├── requirements.txt         # Python包依赖管理
├── run_cryptoflow.sh          # 一键运行脚本
├── spot.json           # 配置文件
├── private_key.pem     # 私钥文件
├── binance/            # Rust代码
├── pyalgo/             # Python扩展
└── target/             # 编译输出
```

### 依赖管理策略

本项目采用 **conda + pip 混合管理** 的方式：

- **environment.yml**: 管理系统级依赖（Rust、OpenSSL、CMake等）和 conda 环境
- **requirements.txt**: 专门管理 Python 包依赖和版本约束
- **集成方式**: environment.yml 通过 `pip: -r requirements.txt` 引用 Python 依赖

**优势：**
1. 🔧 系统工具通过 conda 安装，确保兼容性
2. 🐍 Python 包通过 pip 管理，版本控制更精确
3. 🚀 CI/CD 可以单独使用 requirements.txt
4. 📦 开发者可以选择只用 conda 或 conda+pip

## 🔍 调试和开发技巧

### 1. 检查环境状态
```bash
# 快速检查
./run_cryptoflow.sh --local

# 详细环境信息
conda activate cryptoflow
which python
which cargo
conda list | grep -E "(rust|python|maturin)"
```

## 常见问题

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
conda env remove -n cryptoflow
./run_cryptoflow.sh --setup
```

### Q: 想要使用不同的Python版本
**A:** 修改 `environment.yml` 中的Python版本，然后重新创建环境：
```bash
conda env remove -n cryptoflow
./run_cryptoflow.sh --setup
```
