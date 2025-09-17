from pyalgo import *
from pyalgo.data.strategy_interface import StrategyDataInterface
from datetime import datetime, timedelta
from typing import Optional, List, Dict
import threading


class LiveKlinesArray:
    """
    维护一个“从 start_time 到当前时刻”的 K 线数组，并随实时数据自动更新。

    使用方式：
      - 先用 StrategyDataInterface 预加载历史区间[start_time, now]
      - 订阅 kline:interval 流，收到关闭的K线(is_closed=True)时，追加到数组
      - 数组元素是 dict（open_time/close_time/OHLCV/...），满足策略快速遍历/计算的需要
    """

    def __init__(self, sub: 'BarSubscription', symbol: str, interval: str,
                 start_dt: datetime, csv_dir: Optional[str] = None):
        self.sub = sub
        self.symbol = symbol
        self.interval = interval
        self.start_ts = int(start_dt.timestamp() * 1000)
        self._lock = threading.RLock()
        self.arr: List[Dict] = []

        # 1) 预加载历史数据（会自动从CSV/币安REST补齐缺口）
        self.interface = StrategyDataInterface(csv_dir)
        df = self.interface.get_klines(symbol, interval, start_time=start_dt, end_time=datetime.now())
        if not df.empty:
            cols = [
                'open_time', 'close_time', 'open', 'high', 'low', 'close',
                'volume', 'quote_volume', 'trade_count',
                'taker_buy_volume', 'taker_buy_quote_volume', 'is_closed'
            ]
            # 保证字段存在
            existing = [c for c in cols if c in df.columns]
            with self._lock:
                for _, row in df[existing].iterrows():
                    item = {k: row[k] for k in existing}
                    self.arr.append(item)

        # 2) 绑定实时回调
        self.sub.on_data = self._on_kline

        print(f"✅ LiveKlinesArray ready | symbol={symbol} interval={interval} start={start_dt} preload={len(self.arr)}")

    def _on_kline(self, k: 'Kline'):
        # 仅在K线收盘时写入，避免重复/噪声
        if not k.is_closed:
            return
        if k.time < self.start_ts:
            return

        item = {
            'open_time': k.start_time,
            'close_time': k.time,
            'open': k.open,
            'high': k.high,
            'low': k.low,
            'close': k.close,
            'volume': k.volume,
            'quote_volume': k.amount,
            'trade_count': k.trade_count,
            'taker_buy_volume': k.buy_volume,
            'taker_buy_quote_volume': k.buy_amount,
            'is_closed': k.is_closed,
        }

        with self._lock:
            # 若最后一根同 open_time，则覆盖，否则追加
            if self.arr and self.arr[-1].get('open_time') == item['open_time']:
                self.arr[-1] = item
            else:
                self.arr.append(item)
            print(f"📈 Appended kline | {k.symbol} {k.interval} | size={len(self.arr)} close={k.close}")

    def get_array(self) -> List[Dict]:
        """线程安全地获取当前数组的快照"""
        with self._lock:
            return list(self.arr)


def _parse_dt(s: Optional[str]) -> datetime:
    if not s:
        return datetime.now() - timedelta(hours=24)
    try:
        # 支持 "YYYY-mm-dd HH:MM:SS" 或 ISO 格式
        return datetime.fromisoformat(s)
    except ValueError:
        # 兼容简单日期，只给个兜底
        return datetime.now() - timedelta(hours=24)


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="在一个数组里维护[start_time, now]的K线，并实时更新")
    parser.add_argument('--symbol', default='btcusdt')
    parser.add_argument('--interval', default='1m')
    parser.add_argument('--start', default=None, help='起始时间，例如 2025-09-17 14:00:00（留空默认过去24小时）')
    parser.add_argument('--csv-dir', default='data/csv', help='历史CSV目录（默认data/csv）')
    parser.add_argument('--engine-interval', type=float, default=0.001)
    parser.add_argument('--ws', default='ws://localhost:8111', help='pyalgo WebSocket 地址')
    parser.add_argument('--session-id', type=int, default=1)
    parser.add_argument('--name', default='live_klines_array')
    args = parser.parse_args()

    start_dt = _parse_dt(args.start)

    # 启动 pyalgo 引擎并订阅K线
    eng = Engine(args.engine_interval)
    session = eng.make_session(
        addr=args.ws, session_id=args.session_id, name=args.name, trading=False
    )
    kline_sub = session.subscribe(args.symbol, f"kline:{args.interval}")

    # 构建“实时可更新”的数组
    live = LiveKlinesArray(kline_sub, args.symbol, args.interval, start_dt, csv_dir=args.csv_dir)

    print("🚀 Engine running... 按 Ctrl+C 退出")
    # 你可以在调试器中或另一个线程中定期读取 live.get_array() 以获取从 start_time 到现在的全部K线
    eng.run()