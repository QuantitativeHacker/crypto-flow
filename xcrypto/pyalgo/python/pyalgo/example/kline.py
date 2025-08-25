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
        # 初始化输出文件名（首次调用）
        if self._out_csv is None:
            fname = kline.stream.replace("@", "_").replace(":", "")
            self._out_parquet = self.out_dir / f"{fname}_factor.parquet"
            self._out_csv = self.out_dir / f"{fname}_factor.csv"
            self._out_csv_5m = self.out_dir / f"{fname}_factor_5m.csv"

        # 计算分钟级时间戳
        current_minute = pd.to_datetime((kline.time // 60000) * 60000, unit="ms")
        
        # 构建状态信息
        status = "🟢CLOSED" if kline.is_closed else "🔵LIVE"
        buy_pct = f" | Buy%: {kline.buy_volume/kline.volume*100:.1f}%" if kline.volume > 0 else ""
        info = f" | Trades: {kline.trade_count}{buy_pct}"
        
        if kline.is_closed:
            # K线完结：处理 → 计算 → 保存
            self.buf.append({
                "ts": current_minute, "o": kline.open, "h": kline.high, 
                "l": kline.low, "c": kline.close, "vol": kline.volume, "amount": kline.amount
            })
            
            # 计算指标并输出信号
            if len(self.buf) >= max(self.min_corr, self.min_mean):
                self._compute_indicator()
                if hasattr(self, 'df') and "zscore" in self.df.columns:
                    last = self.df.iloc[-1]
                    print(f"📈 SIGNAL: {current_minute} | close={last.get('c')} | "
                          f"pv_corr={last.get('pv_corr', np.nan):.4f} | z={last.get('zscore', np.nan):.4f}")
            
            self._persist()
            print(f"📊 {status} {current_minute} | Close: {kline.close}{info}")
        else:
            # 实时更新：仅显示
            print(f"🔄 {status} {current_minute} | Close: {kline.close}{info}")

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
        # 直接使用计算好的df或从缓冲构建
        if not self.buf:
            return
            
        df = getattr(self, "df", None) or self._build_df_from_buf()
        if df.empty:
            return

        # 保存最新的一行数据
        last_row = df.iloc[[-1]].copy()
        last_row.index.name = "ts"

        # 选择需要保存的列
        available_cols = [c for c in ["o", "h", "l", "c", "vol", "amount", "pv_corr", "fast_ema", "slow_ema", "slow_std", "zscore"] if c in last_row.columns]
        last_row = last_row[available_cols]

        # 追加保存到CSV
        header = not self._out_csv.exists()
        last_row.to_csv(self._out_csv, mode="a", header=header)

        # 保存5分钟数据（如果有新的zscore）
        df5 = getattr(self, "df5", None)
        if df5 is not None and not df5.empty:
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
