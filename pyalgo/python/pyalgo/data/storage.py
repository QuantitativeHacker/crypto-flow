"""
用于存储收到的kline和depth数据
我们只存储is_close是true的kline，存储的文件命名是：
{symbol}_{interval}.csv例如：btcusdt_1m.csv
"""

from typing import Optional, Dict, Any, List
import json
import csv
from pathlib import Path
import threading
from datetime import datetime

# Import types at runtime to avoid circular import
try:
    from pyalgo.pyalgo import Kline, Depth
except ImportError:
    # Fallback for when pyalgo module is not available
    Kline = None
    Depth = None


class DataStorage:
    """Handles real-time data storage to CSV files (CSV-only backend)"""
    
    def __init__(self, csv_dir: Optional[str] = None):
        # CSV-only backend
        base = Path(csv_dir) if csv_dir else Path("data/csv")
        self.base_csv_dir = base
        self.kline_dir = self.base_csv_dir / "klines"
        self.depth_dir = self.base_csv_dir / "depths"
        self.kline_dir.mkdir(parents=True, exist_ok=True)
        self.depth_dir.mkdir(parents=True, exist_ok=True)

        # Thread-safety for file writes
        self._lock = threading.RLock()
        
        # Statistics
        self.klines_stored = 0
        self.depths_stored = 0
    
    def _write_row(self, file_path: Path, fieldnames: List[str], row: Dict[str, Any]):
        file_exists = file_path.exists()
        with file_path.open("a", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            if not file_exists:
                writer.writeheader()
            writer.writerow(row)
    
    def store_kline(self, kline: Kline) -> bool:
        """Store a kline data point into CSV (one file per symbol-interval)"""
        try:
            # Convert pyalgo Kline to dict format
            kline_data = {
                'symbol': kline.symbol,
                'interval': kline.interval,
                'open_time': kline.start_time,
                'close_time': kline.time,
                'open': kline.open,
                'high': kline.high,
                'low': kline.low,
                'close': kline.close,
                'volume': kline.volume,
                'quote_volume': kline.amount,  # amount is quote volume
                'trade_count': kline.trade_count,
                'taker_buy_volume': kline.buy_volume,
                'taker_buy_quote_volume': kline.buy_amount,
                'is_closed': kline.is_closed,
                'datetime': getattr(kline, 'datetime', None),
            }

            # Record storage time for latency observation
            stored_at_ms = int(datetime.now().timestamp() * 1000)
            stored_at = datetime.now().strftime('%Y-%m-%d %H:%M:%S.%f')[:-3]
            kline_data['stored_at'] = stored_at
            kline_data['stored_at_ms'] = stored_at_ms

            # File path: data/csv/klines/{symbol}_{interval}.csv
            file_path = self.kline_dir / f"{kline.symbol}_{kline.interval}.csv"
            fieldnames = [
                'symbol','interval','open_time','close_time','open','high','low','close',
                'volume','quote_volume','trade_count','taker_buy_volume','taker_buy_quote_volume','is_closed','datetime','stored_at','stored_at_ms'
            ]

            with self._lock:
                self._write_row(file_path, fieldnames, kline_data)
                self.klines_stored += 1
                if self.klines_stored % 100 == 0:
                    print(f"📊 Stored {self.klines_stored} klines (CSV)")
            
            return True
            
        except Exception as e:
            print(f"Error storing kline: {e}")
            return False
    
    def store_depth(self, depth: Depth) -> bool:
        """Store a depth data point into CSV (one file per symbol)"""
        try:
            # Extract bid/ask data from pyalgo Depth
            bid_prices = []
            bid_volumes = []
            ask_prices = []
            ask_volumes = []
            
            # Get all available levels
            for level in range(depth.bid_level):
                bid_prices.append(depth.bid_prc(level))
                bid_volumes.append(depth.bid_vol(level))
            
            for level in range(depth.ask_level):
                ask_prices.append(depth.ask_prc(level))
                ask_volumes.append(depth.ask_vol(level))
            
            # Prepare CSV row (store arrays as JSON strings)
            row = {
                'symbol': depth.symbol,
                'timestamp': depth.time,
                'datetime': getattr(depth, 'datetime', None),
                'bid_prices': json.dumps(bid_prices),
                'bid_volumes': json.dumps(bid_volumes),
                'ask_prices': json.dumps(ask_prices),
                'ask_volumes': json.dumps(ask_volumes)
            }
            if row['datetime'] is None and row['timestamp']:
                row['datetime'] = datetime.fromtimestamp(row['timestamp']/1000).strftime('%Y-%m-%d %H:%M:%S')

            # Record storage time for latency observation
            stored_at_ms = int(datetime.now().timestamp() * 1000)
            stored_at = datetime.now().strftime('%Y-%m-%d %H:%M:%S.%f')[:-3]
            row['stored_at'] = stored_at
            row['stored_at_ms'] = stored_at_ms

            # File path: data/csv/depths/{symbol}_depths.csv
            file_path = self.depth_dir / f"{depth.symbol}_depths.csv"
            fieldnames = ['symbol','timestamp','datetime','bid_prices','bid_volumes','ask_prices','ask_volumes','stored_at','stored_at_ms']

            with self._lock:
                self._write_row(file_path, fieldnames, row)
                self.depths_stored += 1
                if self.depths_stored % 1000 == 0:
                    print(f"📈 Stored {self.depths_stored} depths (CSV)")
            
            return True
            
        except Exception as e:
            print(f"Error storing depth: {e}")
            return False
    
    def get_storage_stats(self) -> Dict[str, int]:
        """Get storage statistics"""
        return {
            'klines_stored': self.klines_stored,
            'depths_stored': self.depths_stored
        }
    
    def close(self):
        """Close storage (no-op for CSV)"""
        print(f"💾 Storage closed. Final stats: {self.get_storage_stats()}")


class DataCollector:
    """Collects and stores real-time market data"""
    
    def __init__(self, storage: DataStorage):
        self.storage = storage
        self.active = True
    
    def on_kline(self, kline: Kline):
        """Callback for kline data"""
        if not self.active:
            return
        
        # Only store closed klines to avoid duplicates
        if kline.is_closed:
            success = self.storage.store_kline(kline)
            if success:
                print(f"✅ Stored kline: {kline.symbol} {kline.interval} {kline.datetime}")
            else:
                print(f"❌ Failed to store kline: {kline.symbol} {kline.interval}")
    
    def on_depth(self, depth: Depth):
        """Callback for depth data"""
        if not self.active:
            return
        
        # Store depth data (can be more frequent)
        success = self.storage.store_depth(depth)
        if success:
            print(f"✅ Stored depth: {depth.symbol} {depth.datetime}")
        else:
            print(f"❌ Failed to store depth: {depth.symbol} {depth.datetime}")

    def stop(self):
        """Stop data collection"""
        self.active = False
        print("🛑 Data collection stopped")
