import argparse
import sys
import importlib.util
import types
from pathlib import Path
from types import SimpleNamespace
import pandas as pd
import numpy as np
from typing import Optional


def install_mock_pyalgo():
    """安装一个最小可用的 mock 模块以满足 kline.py 的 from pyalgo import *。"""
    if "pyalgo" in sys.modules:
        return
    m = types.ModuleType("pyalgo")
    # 提供 Demo 注解所需的符号
    class Depth:  # 占位
        pass
    class Kline:
        pass
    class DepthSubscription:
        pass
    class BarSubscription:
        pass
    # 暴露到模块
    m.Depth = Depth
    m.Kline = Kline
    m.DepthSubscription = DepthSubscription
    m.BarSubscription = BarSubscription
    sys.modules["pyalgo"] = m


def load_demo_class(repo_root: Path):
    """从源码路径加载 Demo 类，避免依赖真实 pyalgo 扩展。"""
    install_mock_pyalgo()
    kline_path = repo_root / "xcrypto" / "pyalgo" / "python" / "pyalgo" / "example" / "kline.py"
    if not kline_path.exists():
        raise FileNotFoundError(f"未找到 kline.py: {kline_path}")
    spec = importlib.util.spec_from_file_location("kline_demo_mod", kline_path)
    mod = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(mod)  # type: ignore
    if not hasattr(mod, "Demo"):
        raise AttributeError("kline.py 未定义 Demo 类")
    return getattr(mod, "Demo")


def load_1m_okx(data_dir: Path, sym: str) -> pd.DataFrame:
    p = data_dir / f"{sym}_okx.parquet"
    if not p.exists():
        raise FileNotFoundError(f"not found: {p}")
    df = pd.read_parquet(p)
    # 标准化列
    need_cols = {"ts", "o", "h", "l", "c", "vol", "volCcyQuote"}
    if not need_cols.issubset(df.columns):
        raise ValueError(f"{p} 缺少列: {need_cols - set(df.columns)}")
    df["ts"] = pd.to_datetime(df["ts"])  # tz-naive
    df = df.sort_values("ts")
    return df


def push_kline_series(demo, df: pd.DataFrame, event_stream: str, event_symbol: str,
                      start: Optional[pd.Timestamp], end: Optional[pd.Timestamp]):
    if start:
        df = df[df["ts"] >= start]
    if end:
        df = df[df["ts"] <= end]

    for _, row in df.iterrows():
        evt = SimpleNamespace(
            # Demo.on_kline 需要的字段
            time=int(pd.Timestamp(row["ts"]).value // 1_000_000),  # ms
            open=float(row["o"]),
            high=float(row["h"]),
            low=float(row["l"]),
            close=float(row["c"]),
            volume=float(row["vol"]),
            amount=float(row["volCcyQuote"]),  # 优先使用报价币种金额
            stream=event_stream,
            symbol=event_symbol,
        )
        demo.on_kline(evt)


def read_demo_5m_output(log_dir: Path, event_stream: str) -> pd.DataFrame:
    fname = event_stream.replace("@", "_").replace(":", "") + "_factor_5m.csv"
    p = log_dir / "indicators" / fname
    if not p.exists():
        raise FileNotFoundError(f"未找到 Demo 输出: {p}")
    df = pd.read_csv(p)
    # 兼容混合日期格式，容错并丢弃解析失败的行
    df["ts"] = pd.to_datetime(df["ts"], errors="coerce", format="mixed")
    df = df.dropna(subset=["ts"])
    df = df.set_index("ts").sort_index()
    # 去重索引，保留最新一条
    df = df[~df.index.duplicated(keep="last")]
    return df


def main():
    parser = argparse.ArgumentParser(description="离线回放 kline.py 的 Demo 并与参考因子比较")
    parser.add_argument("--data_dir", type=str, default="SWAP-AsiaShanghai-1m-parquet")
    parser.add_argument("--reference", type=str, default="factor_zs.parquet")
    parser.add_argument("--symbols", type=str, default="btc,eth",
                        help="币种列表（btc,eth），内部映射到事件符号 btcusdt, ethusdt")
    parser.add_argument("--start", type=str, default=None)
    parser.add_argument("--end", type=str, default=None)
    parser.add_argument("--abs_tol", type=float, default=1e-6)
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[0]
    Demo = load_demo_class(repo_root)

    data_dir = Path(args.data_dir)
    ref = pd.read_parquet(Path(args.reference))
    if ref.index.name != "ts":
        if "ts" in ref.columns:
            ref = ref.set_index("ts")
    ref.index = pd.to_datetime(ref.index)
    ref = ref.sort_index()
    # 去重索引，保留最新一条
    ref = ref[~ref.index.duplicated(keep="last")]

    start = pd.to_datetime(args.start) if args.start else None
    end = pd.to_datetime(args.end) if args.end else None

    # 符号映射与事件 stream 命名
    sym_map = {"btc": ("btcusdt", "btc_okx"), "eth": ("ethusdt", "eth_okx")}
    symbols = [s.strip() for s in args.symbols.split(",") if s.strip()]

    rows = []
    for sym in symbols:
        if sym not in sym_map:
            print(f"[WARN] 未支持的 symbol: {sym}")
            continue
        event_symbol, file_prefix = sym_map[sym]
        event_stream = f"{event_symbol}@kline:1m"
        # 预清理目标 5m 输出，避免读取历史残留
        fname_5m = event_stream.replace("@", "_").replace(":", "") + "_factor_5m.csv"
        p_5m = repo_root / "log" / "indicators" / fname_5m
        if p_5m.exists():
            try:
                p_5m.unlink()
            except FileNotFoundError:
                pass

        # 读取 1m 数据
        df_1m = load_1m_okx(data_dir, sym)

        # 构建 Demo（使用最简 stub 订阅对象）
        stub_depth = SimpleNamespace(on_data=None)
        stub_kline = SimpleNamespace(on_data=None)
        demo = Demo(stub_depth, stub_kline)

        # 回放数据
        push_kline_series(demo, df_1m, event_stream, event_symbol, start, end)

        # 读取 5m 输出并对比参考
        out5 = read_demo_5m_output(repo_root / "log", event_stream)
        # 按窗口裁剪 Demo 输出
        if start is not None:
            out5 = out5[out5.index >= start]
        if end is not None:
            out5 = out5[out5.index <= end]
        if "zscore" not in out5.columns:
            print(f"[WARN] 输出缺少 zscore: {event_stream}")
            continue

        # 参考列
        if sym not in ref.columns:
            print(f"[WARN] 参考因子缺少列: {sym}")
            continue

        cmp = pd.concat({"calc": out5["zscore"], "ref": ref[sym]}, axis=1).dropna()
        if cmp.empty:
            print(f"[WARN] 对齐后为空: {sym}")
            continue

        diff = (cmp["calc"] - cmp["ref"]).abs()
        rows.append({
            "symbol": sym,
            "n": int(diff.count()),
            "mean_abs": float(diff.mean()),
            "max_abs": float(diff.max()),
            "corr": float(cmp["calc"].corr(cmp["ref"])) if diff.count() > 1 else np.nan,
        })

        print(f"\n=== {sym} 样例(前10行) ===")
        print(cmp.head(10))

    if rows:
        rep = pd.DataFrame(rows)
        print("\n===== 汇总 =====")
        print(rep.to_string(index=False))
    else:
        print("未得到对比结果，请检查输入范围与生成输出")


if __name__ == "__main__":
    main()


