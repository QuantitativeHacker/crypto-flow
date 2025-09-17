"""Data query module for historical data retrieval and binance data fetching"""

import requests
import pandas as pd
from typing import List, Dict, Optional, Tuple
from datetime import datetime, timedelta
import time
import csv
import json
from pathlib import Path


class BinanceHistoricalFetcher:
    """Fetches historical data from Binance API"""
    
    def __init__(self):
        self.base_url = "https://api.binance.com/api/v3"
        self.session = requests.Session()
        self.rate_limit_delay = 0.1  # 100ms between requests
    
    def fetch_klines(self, symbol: str, interval: str, start_time: int, end_time: int, limit: int = 1000) -> List[Dict]:
        """Fetch klines from Binance API"""
        url = f"{self.base_url}/klines"
        
        params = {
            'symbol': symbol.upper(),
            'interval': interval,
            'startTime': start_time,
            'endTime': end_time,
            'limit': limit
        }
        
        try:
            time.sleep(self.rate_limit_delay)  # Rate limiting
            response = self.session.get(url, params=params, timeout=10)
            response.raise_for_status()
            
            data = response.json()
            
            # Convert to our format
            klines = []
            for item in data:
                klines.append({
                    'open_time': int(item[0]),
                    'close_time': int(item[6]),
                    'open': float(item[1]),
                    'high': float(item[2]),
                    'low': float(item[3]),
                    'close': float(item[4]),
                    'volume': float(item[5]),
                    'quote_volume': float(item[7]),
                    'trade_count': int(item[8]),
                    'taker_buy_volume': float(item[9]),
                    'taker_buy_quote_volume': float(item[10]),
                    'is_closed': True  # Historical data is always closed
                })
            
            return klines
            
        except Exception as e:
            print(f"Error fetching klines from Binance: {e}")
            return []
    
    def fetch_klines_range(self, symbol: str, interval: str, start_time: int, end_time: int) -> List[Dict]:
        """获取大时间范围内的K线数据，自动处理分页"""
        all_klines = []
        current_start = start_time
        
        # Calculate interval in milliseconds
        interval_ms = self._interval_to_ms(interval)
        max_klines_per_request = 1000
        
        while current_start < end_time:
            # Calculate end time for this batch
            batch_end = min(
                current_start + (max_klines_per_request * interval_ms),
                end_time
            )
            
            print(f"📥 Fetching {symbol} {interval} from {datetime.fromtimestamp(current_start/1000)} to {datetime.fromtimestamp(batch_end/1000)}")
            
            batch_klines = self.fetch_klines(symbol, interval, current_start, batch_end)
            
            if not batch_klines:
                break
            
            all_klines.extend(batch_klines)
            
            # Update start time for next batch
            current_start = batch_klines[-1]['close_time'] + 1
            
            # If we got less than expected, we've reached the end
            if len(batch_klines) < max_klines_per_request:
                break
        
        print(f"✅ Fetched {len(all_klines)} klines for {symbol} {interval}")
        return all_klines
    
    def _interval_to_ms(self, interval: str) -> int:
        """Convert interval string to milliseconds"""
        interval_map = {
            '1m': 60 * 1000,
            '3m': 3 * 60 * 1000,
            '5m': 5 * 60 * 1000,
            '15m': 15 * 60 * 1000,
            '30m': 30 * 60 * 1000,
            '1h': 60 * 60 * 1000,
            '2h': 2 * 60 * 60 * 1000,
            '4h': 4 * 60 * 60 * 1000,
            '6h': 6 * 60 * 60 * 1000,
            '8h': 8 * 60 * 60 * 1000,
            '12h': 12 * 60 * 60 * 1000,
            '1d': 24 * 60 * 60 * 1000,
            '3d': 3 * 24 * 60 * 60 * 1000,
            '1w': 7 * 24 * 60 * 60 * 1000,
            '1M': 30 * 24 * 60 * 60 * 1000,
        }
        return interval_map.get(interval, 60 * 1000)  # Default to 1m


class DataQuery:
    """Handles data queries with automatic historical data fetching (CSV backend)"""
    
    def __init__(self, csv_dir: Optional[str] = None):
        # CSV-only backend
        base = Path(csv_dir) if csv_dir else Path("data/csv")
        self.base_csv_dir = base
        self.kline_dir = self.base_csv_dir / "klines"
        self.depth_dir = self.base_csv_dir / "depths"
        self.kline_dir.mkdir(parents=True, exist_ok=True)
        self.depth_dir.mkdir(parents=True, exist_ok=True)
        self.binance_fetcher = BinanceHistoricalFetcher()
    
    # Helper paths
    def _kline_csv_path(self, symbol: str, interval: str) -> Path:
        return self.kline_dir / f"{symbol}_{interval}.csv"
    
    def _depth_csv_path(self, symbol: str) -> Path:
        return self.depth_dir / f"{symbol}_depths.csv"
    
    def _append_klines_to_csv(self, symbol: str, interval: str, klines: List[Dict]):
        if not klines:
            return
        file_path = self._kline_csv_path(symbol, interval)
        file_exists = file_path.exists()
        fieldnames = [
            'symbol','interval','open_time','close_time','open','high','low','close',
            'volume','quote_volume','trade_count','taker_buy_volume','taker_buy_quote_volume','is_closed','datetime'
        ]
        with file_path.open('a', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            if not file_exists:
                writer.writeheader()
            for k in klines:
                row = {
                    'symbol': symbol,
                    'interval': interval,
                    'open_time': k['open_time'],
                    'close_time': k['close_time'],
                    'open': k['open'],
                    'high': k['high'],
                    'low': k['low'],
                    'close': k['close'],
                    'volume': k['volume'],
                    'quote_volume': k['quote_volume'],
                    'trade_count': k['trade_count'],
                    'taker_buy_volume': k['taker_buy_volume'],
                    'taker_buy_quote_volume': k['taker_buy_quote_volume'],
                    'is_closed': bool(k.get('is_closed', True)),
                    'datetime': datetime.utcfromtimestamp(k['open_time']/1000).strftime('%Y-%m-%d %H:%M:%S')
                }
                writer.writerow(row)
    
    def _read_klines_from_csv(self, symbol: str, interval: str, start_time: int, end_time: int) -> List[Dict]:
        path = self._kline_csv_path(symbol, interval)
        if not path.exists():
            return []
        try:
            df = pd.read_csv(path)
            if 'open_time' not in df.columns:
                return []
            # filter and sort
            df = df[(df['open_time'] >= start_time) & (df['open_time'] <= end_time)]
            if df.empty:
                return []
            df = df.sort_values('open_time')
            # ensure column types
            for c in ['open','high','low','close','volume','quote_volume','taker_buy_volume','taker_buy_quote_volume']:
                if c in df.columns:
                    df[c] = pd.to_numeric(df[c], errors='coerce')
            for c in ['open_time','close_time','trade_count']:
                if c in df.columns:
                    df[c] = pd.to_numeric(df[c], errors='coerce', downcast='integer')
            if 'is_closed' in df.columns:
                # Robust cast for boolean values from CSV
                df['is_closed'] = df['is_closed'].apply(lambda v: True if str(v).lower() in ('true', '1') else (False if str(v).lower() in ('false', '0') else bool(v)))
            # drop duplicates by open_time (keep the last occurrence)
            df = df.drop_duplicates(subset=['open_time'], keep='last')
            cols = ['open_time','close_time','open','high','low','close','volume','quote_volume','trade_count','taker_buy_volume','taker_buy_quote_volume','is_closed']
            df = df[cols]
            return df.to_dict(orient='records')
        except Exception as e:
            print(f"Error reading klines CSV: {e}")
            return []
    
    def get_klines(self, symbol: str, interval: str, start_time: int, end_time: int, auto_fetch: bool = True) -> List[Dict]:
        """Get klines from CSV with optional Binance backfill to CSV"""
        # Read from CSV first
        csv_klines = self._read_klines_from_csv(symbol, interval, start_time, end_time)
        
        if not auto_fetch:
            return csv_klines
        
        # Find missing ranges based on CSV data
        missing_ranges = self._find_missing_kline_ranges(symbol, interval, start_time, end_time, csv_klines)
        
        if missing_ranges:
            print(f"🔍 Found {len(missing_ranges)} missing data ranges for {symbol} {interval}")
            for missing_start, missing_end in missing_ranges:
                print(f"📥 Fetching missing data: {datetime.fromtimestamp(missing_start/1000)} to {datetime.fromtimestamp(missing_end/1000)}")
                fetched_klines = self.binance_fetcher.fetch_klines_range(symbol, interval, missing_start, missing_end)
                if fetched_klines:
                    # Append fetched to CSV
                    self._append_klines_to_csv(symbol, interval, fetched_klines)
        
        # Re-read merged range from CSV
        merged = self._read_klines_from_csv(symbol, interval, start_time, end_time)
        return merged
    
    def _find_missing_kline_ranges(self, symbol: str, interval: str, start_time: int, end_time: int, existing_klines: List[Dict]) -> List[Tuple[int, int]]:
        """Find missing time ranges in kline data"""
        if not existing_klines:
            return [(start_time, end_time)]
        
        # Sort existing klines by time
        existing_klines.sort(key=lambda x: x['open_time'])
        
        missing_ranges = []
        interval_ms = self.binance_fetcher._interval_to_ms(interval)
        
        # Check gap before first kline
        first_kline_time = existing_klines[0]['open_time']
        if start_time < first_kline_time:
            missing_ranges.append((start_time, first_kline_time - 1))
        
        # Check gaps between klines
        for i in range(len(existing_klines) - 1):
            current_end = existing_klines[i]['close_time']
            next_start = existing_klines[i + 1]['open_time']
            
            # If there's a gap larger than one interval
            if next_start - current_end > interval_ms:
                missing_ranges.append((current_end + 1, next_start - 1))
        
        # Check gap after last kline
        last_kline_time = existing_klines[-1]['close_time']
        if end_time > last_kline_time:
            missing_ranges.append((last_kline_time + 1, end_time))
        
        return missing_ranges
    
    def get_depths(self, symbol: str, start_time: int, end_time: int) -> List[Dict]:
        """Get depth data from CSV (no auto-fetch)"""
        path = self._depth_csv_path(symbol)
        if not path.exists():
            return []
        try:
            df = pd.read_csv(path)
            if 'timestamp' not in df.columns:
                return []
            df = df[(df['timestamp'] >= start_time) & (df['timestamp'] <= end_time)]
            if df.empty:
                return []
            df = df.sort_values('timestamp')
            # Decode arrays
            for col in ['bid_prices','bid_volumes','ask_prices','ask_volumes']:
                if col in df.columns:
                    df[col] = df[col].apply(lambda x: json.loads(x) if isinstance(x, str) else x)
            cols = ['timestamp','bid_prices','bid_volumes','ask_prices','ask_volumes']
            df = df[cols]
            return df.to_dict(orient='records')
        except Exception as e:
            print(f"Error reading depths CSV: {e}")
            return []
    
    def get_latest_kline_time(self, symbol: str, interval: str) -> Optional[int]:
        """Get the latest kline timestamp from CSV"""
        path = self._kline_csv_path(symbol, interval)
        if not path.exists():
            return None
        try:
            df = pd.read_csv(path, usecols=['close_time'])
            if df.empty:
                return None
            return int(pd.to_numeric(df['close_time'], errors='coerce').dropna().max())
        except Exception:
            return None
    
    def get_klines_as_dataframe(self, symbol: str, interval: str, start_time: int, end_time: int, auto_fetch: bool = True) -> pd.DataFrame:
        """Get klines as pandas DataFrame"""
        klines = self.get_klines(symbol, interval, start_time, end_time, auto_fetch)
        
        if not klines:
            return pd.DataFrame()
        
        df = pd.DataFrame(klines)
        df['datetime'] = pd.to_datetime(df['open_time'], unit='ms')
        df.set_index('datetime', inplace=True)
        
        return df
    
    def close(self):
        """No-op for CSV backend"""
        return