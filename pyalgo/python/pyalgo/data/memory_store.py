"""Memory data structure for seamless historical and real-time data integration"""

import pandas as pd
import numpy as np
from typing import Dict, List, Optional, Tuple, Union
from collections import deque
from datetime import datetime, timedelta
import threading
from .query import DataQuery

# Import types at runtime to avoid circular import
try:
    from pyalgo.pyalgo import Kline, Depth
except ImportError:
    # Fallback for when pyalgo module is not available
    Kline = None
    Depth = None


class KlineMemoryStore:
    """In-memory store for kline data with historical data integration"""
    
    def __init__(self, symbol: str, interval: str, max_size: int = 10000, csv_dir: Optional[str] = None):
        self.symbol = symbol
        self.interval = interval
        self.max_size = max_size
        
        # Thread-safe data storage
        self._lock = threading.RLock()
        self._data = deque(maxlen=max_size)
        self._df_cache = None
        self._cache_valid = False
        
        # CSV query interface
        self.data_query = DataQuery(csv_dir)
        
        # Track data range
        self._min_time = None
        self._max_time = None
        
        print(f"📊 Initialized KlineMemoryStore for {symbol} {interval}")
    
    def load_historical_data(self, start_time: int, end_time: int) -> int:
        """Load historical data from CSV/Binance into memory"""
        print(f"📥 Loading historical data for {self.symbol} {self.interval} from {datetime.fromtimestamp(start_time/1000)} to {datetime.fromtimestamp(end_time/1000)}")
        
        # Get historical data (with auto-fetch from Binance if needed)
        historical_klines = self.data_query.get_klines(
            self.symbol, self.interval, start_time, end_time, auto_fetch=True
        )
        
        with self._lock:
            # Clear existing data
            self._data.clear()
            
            # Add historical data
            for kline_dict in historical_klines:
                self._add_kline_dict(kline_dict)
            
            self._cache_valid = False
        
        print(f"✅ Loaded {len(historical_klines)} historical klines")
        return len(historical_klines)
    
    def add_realtime_kline(self, kline: Kline) -> bool:
        """Add real-time kline data"""
        with self._lock:
            kline_dict = {
                'open_time': kline.start_time,
                'close_time': kline.time,
                'open': kline.open,
                'high': kline.high,
                'low': kline.low,
                'close': kline.close,
                'volume': kline.volume,
                'quote_volume': kline.amount,
                'trade_count': kline.trade_count,
                'taker_buy_volume': kline.buy_volume,
                'taker_buy_quote_volume': kline.buy_amount,
                'is_closed': kline.is_closed
            }
            
            # Check if this kline already exists (update case)
            updated = False
            for i, existing in enumerate(self._data):
                if existing['open_time'] == kline_dict['open_time']:
                    self._data[i] = kline_dict
                    updated = True
                    break
            
            if not updated:
                self._add_kline_dict(kline_dict)
            
            self._cache_valid = False
            return True
    
    def _add_kline_dict(self, kline_dict: Dict):
        """Add kline dictionary to internal storage"""
        self._data.append(kline_dict)
        
        # Update time range
        open_time = kline_dict['open_time']
        if self._min_time is None or open_time < self._min_time:
            self._min_time = open_time
        if self._max_time is None or open_time > self._max_time:
            self._max_time = open_time
    
    def get_dataframe(self, start_time: Optional[int] = None, end_time: Optional[int] = None) -> pd.DataFrame:
        """Get data as pandas DataFrame"""
        with self._lock:
            if not self._cache_valid or self._df_cache is None:
                self._rebuild_cache()
            
            df = self._df_cache.copy()
            
            # If cache is empty, return immediately to avoid KeyError on filtering
            if df.empty:
                return df
            
            # Filter by time range if specified
            if start_time is not None:
                df = df[df['open_time'] >= start_time]
            if end_time is not None:
                df = df[df['open_time'] <= end_time]
            
            return df
    
    def _rebuild_cache(self):
        """Rebuild DataFrame cache"""
        if not self._data:
            # Ensure an empty DataFrame still has the expected schema to prevent KeyError downstream
            columns = [
                'open_time', 'close_time', 'open', 'high', 'low', 'close',
                'volume', 'quote_volume', 'trade_count',
                'taker_buy_volume', 'taker_buy_quote_volume', 'is_closed',
                'datetime'
            ]
            self._df_cache = pd.DataFrame(columns=columns)
            self._cache_valid = True
            return
        
        # Convert to DataFrame
        df = pd.DataFrame(list(self._data))
        df['datetime'] = pd.to_datetime(df['open_time'], unit='ms')
        df = df.sort_values('open_time').reset_index(drop=True)
        
        self._df_cache = df
        self._cache_valid = True
    
    def get_latest_kline(self) -> Optional[Dict]:
        """Get the latest kline"""
        with self._lock:
            if not self._data:
                return None
            return dict(self._data[-1])
    
    def get_kline_count(self) -> int:
        """Get number of klines in memory"""
        with self._lock:
            return len(self._data)
    
    def get_time_range(self) -> Tuple[Optional[int], Optional[int]]:
        """Get time range of data in memory"""
        with self._lock:
            return self._min_time, self._max_time
    
    def ensure_data_coverage(self, start_time: int, end_time: int) -> bool:
        """Ensure data coverage for specified time range"""
        min_time, max_time = self.get_time_range()
        
        # Check if we need to load more data
        need_load = False
        load_start = start_time
        load_end = end_time
        
        if min_time is None or max_time is None:
            need_load = True
        else:
            if start_time < min_time:
                need_load = True
                load_end = min_time - 1
            elif end_time > max_time:
                need_load = True
                load_start = max_time + 1
        
        if need_load:
            self.load_historical_data(load_start, load_end)
            return True
        
        return False


class DepthMemoryStore:
    """In-memory store for depth data"""
    
    def __init__(self, symbol: str, max_size: int = 1000):
        self.symbol = symbol
        self.max_size = max_size
        
        # Thread-safe data storage
        self._lock = threading.RLock()
        self._data = deque(maxlen=max_size)
        
        print(f"📈 Initialized DepthMemoryStore for {symbol}")
    
    def add_depth(self, depth: Depth) -> bool:
        """Add depth data"""
        with self._lock:
            # Extract bid/ask data
            bid_prices = [depth.bid_prc(i) for i in range(depth.bid_level)]
            bid_volumes = [depth.bid_vol(i) for i in range(depth.bid_level)]
            ask_prices = [depth.ask_prc(i) for i in range(depth.ask_level)]
            ask_volumes = [depth.ask_vol(i) for i in range(depth.ask_level)]
            
            depth_dict = {
                'timestamp': depth.time,
                'datetime': depth.datetime,
                'bid_prices': bid_prices,
                'bid_volumes': bid_volumes,
                'ask_prices': ask_prices,
                'ask_volumes': ask_volumes,
                'best_bid': bid_prices[0] if bid_prices else 0,
                'best_ask': ask_prices[0] if ask_prices else 0,
                'spread': (ask_prices[0] - bid_prices[0]) if (bid_prices and ask_prices) else 0
            }
            
            self._data.append(depth_dict)
            return True
    
    def get_latest_depth(self) -> Optional[Dict]:
        """Get the latest depth"""
        with self._lock:
            if not self._data:
                return None
            return dict(self._data[-1])
    
    def get_depth_history(self, count: int = 100) -> List[Dict]:
        """Get recent depth history"""
        with self._lock:
            if not self._data:
                return []
            
            start_idx = max(0, len(self._data) - count)
            return [dict(d) for d in list(self._data)[start_idx:]]


class DataManager:
    """Manages multiple memory stores for different symbols and intervals"""
    
    def __init__(self, csv_dir: Optional[str] = None):
        self.csv_dir = csv_dir
        self.kline_stores: Dict[str, KlineMemoryStore] = {}
        self.depth_stores: Dict[str, DepthMemoryStore] = {}
        self._lock = threading.RLock()
    
    def get_kline_store(self, symbol: str, interval: str) -> KlineMemoryStore:
        """Get or create kline store for symbol-interval pair"""
        key = f"{symbol}@{interval}"
        
        with self._lock:
            if key not in self.kline_stores:
                self.kline_stores[key] = KlineMemoryStore(symbol, interval, csv_dir=self.csv_dir)
            
            return self.kline_stores[key]
    
    def get_depth_store(self, symbol: str) -> 'DepthMemoryStore':
        """Get or create depth store for a symbol"""
        with self._lock:
            if symbol not in self.depth_stores:
                self.depth_stores[symbol] = DepthMemoryStore(symbol)
            return self.depth_stores[symbol]
    
    def add_realtime_kline(self, kline: Kline):
        """Add real-time kline to appropriate store"""
        store = self.get_kline_store(kline.symbol, kline.interval)
        store.add_realtime_kline(kline)
    
    def add_realtime_depth(self, depth: Depth):
        """Add real-time depth to appropriate store"""
        store = self.get_depth_store(depth.symbol)
        store.add_depth(depth)
    
    def prepare_strategy_data(self, symbol: str, interval: str, start_time: int, current_time: int) -> pd.DataFrame:
        """Prepare complete dataset for strategy (historical + real-time)"""
        print(f"🎯 Preparing strategy data for {symbol} {interval}")
        
        store = self.get_kline_store(symbol, interval)
        
        # Ensure we have data coverage
        store.ensure_data_coverage(start_time, current_time)
        
        # Get complete dataset
        df = store.get_dataframe(start_time, current_time)
        
        print(f"✅ Prepared {len(df)} klines for strategy")
        return df
    
    def get_status(self) -> Dict:
        """Get status of all data stores"""
        status = {}
        
        with self._lock:
            # Kline stores
            kline_status = {}
            for key, store in self.kline_stores.items():
                min_time, max_time = store.get_time_range()
                kline_status[key] = {
                    'count': store.get_kline_count(),
                    'time_range': {
                        'min': min_time,
                        'max': max_time
                    }
                }
            
            # Depth stores
            depth_status = {}
            for symbol, store in self.depth_stores.items():
                latest = store.get_latest_depth()
                depth_status[symbol] = {
                    'has_data': latest is not None,
                    'latest_time': latest['timestamp'] if latest else None
                }
            
            status = {
                'klines': kline_status,
                'depths': depth_status
            }
            
            return status