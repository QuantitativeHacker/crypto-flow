# xcrypto 一键运行指南

## 概述

`run_xcrypto.sh` 脚本提供了一键配置和运行xcrypto项目的功能，支持Docker容器和本地miniconda环境两种方式。

## 使用方法

### 基本用法

```bash
# 使用默认配置运行（spot模式，自动选择Docker或本地环境）
./run_xcrypto.sh

# 指定模式运行
./run_xcrypto.sh spot    # 现货交易模式
./run_xcrypto.sh usdt    # USDT期货模式
```

### 环境配置

```bash
# 一键配置本地miniconda环境
./run_xcrypto.sh --setup

# 配置特定模式的环境
./run_xcrypto.sh spot --setup
./run_xcrypto.sh usdt --setup
```

### 强制指定运行方式

```bash
# 强制使用Docker容器运行
./run_xcrypto.sh spot --docker
./run_xcrypto.sh usdt --docker

# 强制使用本地miniconda环境运行
./run_xcrypto.sh spot --local
./run_xcrypto.sh usdt --local
```

### 获取帮助

```bash
./run_xcrypto.sh --help
```

## 环境要求

### Docker方式（推荐）
- 安装Docker
- 脚本会自动构建和管理容器

### 本地miniconda方式
- 安装miniconda或anaconda
- 运行 `./run_xcrypto.sh --setup` 配置环境

## 文件说明

- `run_xcrypto.sh` - 主运行脚本
- `environment.yml` - conda环境配置文件
- `Dockerfile.dev` - Docker开发环境配置
- `requirements.txt` - Python依赖配置

## 端口说明

默认WebSocket服务端口：`8111`
- 本地访问：`ws://localhost:8111`
- 容器内外端口映射：`8111:8111`

## 故障排除

1. **conda环境不存在**：运行 `./run_xcrypto.sh --setup`
2. **Docker镜像构建失败**：检查Dockerfile.dev和environment.yml
3. **端口冲突**：确保8111端口未被占用
4. **配置文件缺失**：确保存在对应的.json配置文件和private_key.pem

## 环境说明

使用miniconda管理所有依赖（Python、Rust、系统库），避免污染主机环境。
