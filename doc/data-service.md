# Crypto Data Service

基于pyalgo架构的加密货币实时数据收集和存储服务。

## 功能特性

### 实时数据收集
- 基于pyalgo WebSocket连接实时接收kline和depth数据
- 支持多个交易对同时订阅
- 自动数据持久化到CSV文件
- 线程安全的数据存储

### 历史数据管理
- 自动从Binance API补充缺失的历史数据
- 智能数据缺口检测和填补
- 支持时间范围查询
- CSV按交易对/周期分文件保存，便于增量写入

### 内存数据结构
- 历史数据和实时数据无缝衔接
- 策略友好的数据接口
- 线程安全的内存存储
- 自动数据缓存管理

## 快速开始

### 1. 启动数据服务

```bash
# 启动数据收集服务
python start_data_service.py

# 使用自定义配置
python start_data_service.py --config config/my_config.json

# 测试模式（运行30秒后自动停止）
python start_data_service.py --test
```

### 2. 查询数据

```bash
# 查询最近1天的BTC 1分钟数据
python query_data.py --symbol btcusdt --interval 1m --days 1

# 查询指定时间范围的数据
python query_data.py --symbol ethusdt --interval 5m \
  --start "2024-01-01 00:00:00" --end "2024-01-02 00:00:00"

# 导出数据到CSV
python query_data.py --symbol btcusdt --interval 1m \
  --days 7 --format csv --output btc_week.csv
```

### 3. 在策略中使用

```python
from pyalgo.data.strategy_interface import StrategyDataProvider
from datetime import datetime, timedelta

# 创建数据提供者
data_provider = StrategyDataProvider("btcusdt", "1m")

# 获取最近1000根K线
df = data_provider.get_data(count=1000)

# 获取指定时间范围的数据
start_time = datetime.now() - timedelta(hours=24)
end_time = datetime.now()
df = data_provider.get_data_range(start_time, end_time)

# 获取最新价格
latest_price = data_provider.get_latest_price()

# 获取最新深度
depth = data_provider.get_latest_depth()
```

## 使用场景

### 场景1：策略启动前数据准备

```bash
# 12:00启动数据服务，为12:30的策略做准备
python start_data_service.py
```

策略在12:30启动时将自动获得：
- 从start_time到12:00的完整历史数据
- 12:00到12:30的实时收集数据
- 12:30之后的持续实时数据

### 场景2：历史数据回测

```python
from pyalgo.data.strategy_interface import StrategyDataInterface

data_interface = StrategyDataInterface()

# 获取回测数据（自动补充缺失数据）
backtest_data = data_interface.get_klines(
    symbol="btcusdt",
    interval="1h", 
    start_time=datetime(2024, 1, 1),
    end_time=datetime(2024, 1, 31)
)
```

## 配置说明

### 数据服务配置 (config/data_service.json)

```json
{
  "websocket_addr": "ws://localhost:8111",  // pyalgo WebSocket地址
  "session_id": 1,                          // 会话ID
  "session_name": "data_service",           // 会话名称
  "trading": false,                         // 是否启用交易
  "csv_dir": "data/csv",                    // CSV数据目录
  "subscriptions": [                        // 订阅配置
    {
      "symbol": "btcusdt",
      "stream": "kline:1m"
    },
    {
      "symbol": "btcusdt", 
      "stream": "depth"
    }
  ],
  "engine_interval": 0.001                  // 引擎轮询间隔
}
```

## CSV文件结构

### Klines CSV文件
- symbol: 交易对
- interval: 时间间隔
- open_time/close_time: 开盘/收盘时间
- OHLCV数据: open, high, low, close, volume
- 交易统计与附加字段: quote_volume, trade_count, taker_buy_volume, taker_buy_quote_volume, is_closed, datetime, stored_at, stored_at_ms

### Depths CSV文件
- symbol: 交易对
- timestamp: 时间戳
- bid_prices/bid_volumes: 买盘价格和数量（JSON数组）
- ask_prices/ask_volumes: 卖盘价格和数量（JSON数组）

## API参考

### DataQuery类
```python
from pyalgo.data.query import DataQuery

query = DataQuery("data/csv")

# 获取K线数据（自动补充缺失数据）
klines = query.get_klines("btcusdt", "1m", start_time, end_time)

# 获取深度数据
depths = query.get_depths("btcusdt", start_time, end_time)

# 获取DataFrame格式数据
df = query.get_klines_as_dataframe("btcusdt", "1m", start_time, end_time)
```

### DataManager类
```python
from memory_store import DataManager

manager = DataManager()

# 添加实时数据
manager.add_realtime_kline(kline)
manager.add_realtime_depth(depth)

# 为策略准备数据
df = manager.prepare_strategy_data("btcusdt", "1m", start_time, current_time)
```

## 性能优化

### 存储与I/O
- 顺序追加写入CSV，减少文件打开/关闭开销
- 单交易对/周期单文件，便于增量读取与归档
- 文件写入加锁（RLock）确保并发安全

### 内存管理
- 环形缓冲区限制内存使用
- 智能缓存失效机制
- 线程安全的数据访问

### 网络优化
- Binance API请求限流
- 自动重试机制
- 分批获取大量历史数据

## 监控和日志

### 服务状态监控
```bash
# 查看服务状态
python start_data_service.py --status
```

### 日志输出
- 实时数据接收状态
- 数据存储统计
- 错误和异常信息
- 性能指标

## 故障排除

### 常见问题

1. **WebSocket连接失败**
   - 检查pyalgo服务是否运行在localhost:8111
   - 确认网络连接正常

2. **CSV写入失败或权限问题**
   - 确保只有一个数据服务实例在写入相同CSV
   - 检查CSV目录权限与磁盘空间

3. **Binance API限流**
   - 服务会自动处理限流，请耐心等待
   - 可以调整rate_limit_delay参数

4. **内存使用过高**
   - 避免一次性订阅过多交易对或过短周期
   - 定期归档/压缩CSV以控制数据量

### 调试模式
```python
# 启用详细日志
import logging
logging.basicConfig(level=logging.DEBUG)
```

## 扩展开发

### 添加新的数据源
1. 继承DataCollector类
2. 实现相应的数据转换逻辑
3. 在配置文件中添加新的订阅

### 自定义存储后端
1. 继承或替换DataStorage类
2. 实现store_kline/store_depth接口
3. 在RealtimeDataService中注入你的存储实现

## 许可证

本项目遵循与crypto-flow主项目相同的许可证。