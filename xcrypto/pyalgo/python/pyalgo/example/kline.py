from pyalgo import *

# --- New imports for indicator computation and persistence ---
from pathlib import Path
import pandas as pd
import numpy as np
from collections import deque  # 新增：环形缓冲队列


class Demo:
    """Subscribe klines and compute/store a correlation-based factor.

    Indicator definition (streaming):
    - Maintain 1m OHLCV series
    - Rolling mean/std use min_periods=100 for both close and volume
    - Clip close and volume by their rolling mean ± 0.5 * std (window=1200)
    - Rolling correlation of clipped close vs clipped volume (window=1200, min_periods=200)
    - Then reindex corr series to 5-minute grid and compute:
      fast EMA(span=48), slow EMA(span=8*48),
      zscore = (fast - slow) / rolling_std(window=8*48, min_periods=10)
    Persist latest columns to disk per symbol-stream.
    """

    def __init__(self, depth: DepthSubscription, kline: BarSubscription):
        self.depth = depth
        self.kline = kline

        # set callback
        self.depth.on_data = self.on_depth
        self.kline.on_data = self.on_kline

        # 使用环形缓冲，仅保留最近 N 分钟数据（覆盖最大窗口 1200，并留少量冗余）
        self.keep_window = 1300
        self.buf = deque(maxlen=self.keep_window)  # 每个元素为 dict：{"ts", "o","h","l","c","vol","amount"}

        # 当前分钟的临时快照（同一分钟内覆盖更新；分钟切换时推入 buf）
        self.cur_ts = None
        self.cur_row = None
        
        # Track last processed minute to avoid duplicates
        self.last_minute = None

        # parameters (align with user's notebook)
        self.window = 1200  # 1200 minutes = 20 hours
        self.min_mean = 100
        self.min_corr = 200
        self.norm_window = 48

        # prepare output path lazily (need stream from first kline event)
        self.out_dir = Path("log/indicators")
        self.out_dir.mkdir(parents=True, exist_ok=True)
        
        # 程序启动时清理旧的输出文件
        print("🧹 清理旧的因子文件...")
        for old_file in self.out_dir.glob("*_factor.csv"):
            try:
                old_file.unlink()
                print(f"   删除: {old_file.name}")
            except FileNotFoundError:
                pass  # 文件不存在，忽略
        for old_file in self.out_dir.glob("*_factor.parquet"):
            try:
                old_file.unlink()
                print(f"   删除: {old_file.name}")
            except FileNotFoundError:
                pass  # 文件不存在，忽略
        print("✅ 清理完成，准备开始新的因子计算")
        
        self._out_parquet = None
        self._out_csv = None
        self._out_csv_5m = None
        self._last_5m_ts = None

    # ----------------- callbacks -----------------
    def on_kline(self, kline: Kline):
        # lazily derive output filenames using kline.stream on first data
        if self._out_parquet is None or self._out_csv is None:
            symbol_stream = kline.stream if hasattr(kline, "stream") else f"{kline.symbol}@kline"
            fname = symbol_stream.replace("@", "_").replace(":", "")
            self._out_parquet = self.out_dir / f"{fname}_factor.parquet"
            self._out_csv = self.out_dir / f"{fname}_factor.csv"
            self._out_csv_5m = self.out_dir / f"{fname}_factor_5m.csv"

        # Convert timestamp to minute-level precision
        timestamp_ms = int(kline.time)
        minute_timestamp = (timestamp_ms // 60000) * 60000  # Round down to minute
        current_minute = pd.to_datetime(minute_timestamp, unit="ms")
        
        # 第一次初始化当前分钟
        if self.cur_ts is None:
            self.cur_ts = current_minute
            self.cur_row = {
                "ts": current_minute,
                "o": kline.open,
                "h": kline.high,
                "l": kline.low,
                "c": kline.close,
                "vol": kline.volume,
                "amount": kline.amount,
            }
            print(f"📊 New minute data: {current_minute} | Close: {kline.close} | Buffer: {len(self.buf)}")
            return

        # 新分钟开始：先结算上一分钟（已收盘），再开启新分钟
        if current_minute > self.cur_ts:
            # 把上一分钟最终快照推入环形缓冲
            self.buf.append(self.cur_row)

            # 计算指标（当缓冲长度满足最小窗口）
            if len(self.buf) >= max(self.min_corr, self.min_mean):
                self._compute_indicator()  # 从 buf 构建 df 并计算 pv_corr/zscore 等
                if hasattr(self, 'df') and "zscore" in self.df.columns:
                    last = self.df.iloc[-1]
                    print(
                        f"📈 {self.cur_ts} | close={last.get('c')} | pv_corr={round(last.get('pv_corr', np.nan), 4)} | z={round(last.get('zscore', np.nan), 4)}"
                    )

            # 持久化上一分钟（只追加一行）
            self._persist()

            # 开启新分钟快照（同一分钟内仅覆盖，不落盘不计算）
            self.cur_ts = current_minute
            self.cur_row = {
                "ts": current_minute,
                "o": kline.open,
                "h": kline.high,
                "l": kline.low,
                "c": kline.close,
                "vol": kline.volume,
                "amount": kline.amount,
            }
            print(f"📊 New minute data: {current_minute} | Close: {kline.close} | Buffer: {len(self.buf)}")
        else:
            # 同一分钟内：只覆盖为最新快照（交易所逐秒 kline 的最新值）
            self.cur_row.update({
                "h": kline.high,
                "l": kline.low,
                "c": kline.close,
                "vol": kline.volume,
                "amount": kline.amount,
            })
            print(f"🔄 Update: {current_minute} | Close: {kline.close}")

    def on_depth(self, depth: Depth):
        # keep the original simple print for depth
        print(f"📊 Depth: {depth.datetime} {depth.symbol}")

    # ----------------- core logic -----------------
    def _build_df_from_buf(self):
        """将环形缓冲的数据构建为 DataFrame（仅用于计算/持久化时的临时视图）"""
        if not self.buf:
            return pd.DataFrame()
        df = pd.DataFrame(list(self.buf))
        df = df.set_index("ts")
        return df

    def _compute_indicator(self):
        # 从环形缓冲构建临时 df
        self.df = self._build_df_from_buf()
        if self.df.empty:
            return

        close = self.df["c"].astype(float)
        # Prefer quote volume (amount) if available to match offline factor using volCcyQuote
        volume = (
            self.df["amount"].astype(float)
            if "amount" in self.df.columns
            else self.df["vol"].astype(float)
        )

        # rolling stats with explicit min_periods to match notebook
        c_mean = close.rolling(self.window, min_periods=self.min_mean).mean()
        c_std = close.rolling(self.window, min_periods=self.min_mean).std()
        v_mean = volume.rolling(self.window, min_periods=self.min_mean).mean()
        v_std = volume.rolling(self.window, min_periods=self.min_mean).std()

        c_upper = c_mean + 0.5 * c_std
        c_lower = c_mean - 0.5 * c_std
        v_upper = v_mean + 0.5 * v_std
        v_lower = v_mean - 0.5 * v_std

        clip_c = close.clip(lower=c_lower, upper=c_upper)
        clip_v = volume.clip(lower=v_lower, upper=v_upper)

        pv_corr_1m = clip_c.rolling(self.window, min_periods=self.min_corr).corr(clip_v)
        self.df["pv_corr"] = pv_corr_1m

        # 5-minute normalization (align with notebook)
        index_5m = close.resample("5min").last().dropna().index
        factor_5m = pv_corr_1m.reindex(index_5m)

        # STRICT per user's formula: use ewm(norm_window) positional arg (com)
        fast_ema = factor_5m.ewm(self.norm_window, min_periods=10).mean()
        slow_ema = factor_5m.ewm(8 * self.norm_window, min_periods=10).mean()
        slow_std = factor_5m.rolling(8 * self.norm_window, min_periods=10).std()

        # z = (fast - slow) / slow_std
        z = (fast_ema - slow_ema) / slow_std.replace(0, np.nan)

        # Store 5m dataframe for persistence/inspection
        self.df5 = pd.DataFrame(
            {
                "pv_corr": factor_5m,
                "fast_ema": fast_ema,
                "slow_ema": slow_ema,
                "slow_std": slow_std,
                "zscore": z,
            },
            index=index_5m,
        )

    def _persist(self):
        # 优先使用已计算好的 df；如果尚未达到最小窗口，则用环形缓冲构建 df（仅 OHLCV）
        df = getattr(self, "df", None)
        if df is None or df.empty or (df.index[-1] if not df.empty else None) != (self.buf[-1]["ts"] if self.buf else None):
            df = self._build_df_from_buf()
        if df.empty:
            return

        # 只持久化"上一分钟（环形缓冲最后一行）"
        last_row = df.iloc[[-1]].copy()
        last_row.index.name = "ts"

        desired_cols = [
            "o", "h", "l", "c", "vol", "amount",
            "pv_corr", "fast_ema", "slow_ema", "slow_std", "zscore",
        ]
        out_cols = [c for c in desired_cols if c in last_row.columns]
        last_row = last_row[out_cols]

        # 采用 CSV 追加写入，保留全历史且不占用过多内存；首次写入带表头
        if self._out_csv is not None:
            header = not self._out_csv.exists()
            last_row.to_csv(self._out_csv, mode="a", header=header)

        # 附加：5m 结果持久化，仅在产生新5m刻度时追加一行
        df5 = getattr(self, "df5", None)
        if df5 is not None and not df5.empty and self._out_csv_5m is not None:
            # 取最后一个非NaN的 zscore 点
            non_na = df5[~df5["zscore"].isna()]
            if not non_na.empty:
                last_5m_ts = non_na.index[-1]
                if self._last_5m_ts is None or last_5m_ts > self._last_5m_ts:
                    out_row_5m = non_na.iloc[[-1]].copy()
                    out_row_5m.index.name = "ts"
                    header5 = not self._out_csv_5m.exists()
                    out_row_5m.to_csv(self._out_csv_5m, mode="a", header=header5)
                    self._last_5m_ts = last_5m_ts


if __name__ == "__main__":
    eng = Engine(0.001)
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )

    depth = session.subscribe("dogeusdt", "depth")
    kline = session.subscribe("btcusdt", "kline:1m")
    demo = Demo(depth, kline)
    eng.run()
