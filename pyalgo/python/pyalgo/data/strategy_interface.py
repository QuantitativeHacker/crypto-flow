"""Strategy interface for seamless data access"""

import pandas as pd
from typing import Optional, Dict, List
from datetime import datetime, timedelta
from .memory_store import DataManager
from .query import DataQuery
import threading


class StrategyDataInterface:
    """Interface for strategies to access historical and real-time data seamlessly"""
    
    def __init__(self, csv_dir: Optional[str] = None):
        self.data_manager = DataManager(csv_dir)
        self.data_query = DataQuery(csv_dir)
        self._lock = threading.RLock()
        
        print("🎯 Strategy Data Interface initialized")
    
    def get_klines(self, symbol: str, interval: str, start_time: Optional[datetime] = None, 
                   end_time: Optional[datetime] = None, count: Optional[int] = None) -> pd.DataFrame:
        """Get klines for strategy use
        
        Args:
            symbol: Trading symbol (e.g., 'btcusdt')
            interval: Kline interval (e.g., '1m', '5m', '1h')
            start_time: Start time (if None, uses count parameter)
            end_time: End time (if None, uses current time)
            count: Number of klines to get (if start_time is None)
        
        Returns:
            DataFrame with kline data
        """
        # Convert datetime to timestamp
        if end_time is None:
            end_time = datetime.now()
        end_ts = int(end_time.timestamp() * 1000)
        
        if start_time is None:
            if count is None:
                count = 1000  # Default
            # Calculate start time based on count and interval
            interval_minutes = self._interval_to_minutes(interval)
            start_time = end_time - timedelta(minutes=interval_minutes * count)
        
        start_ts = int(start_time.timestamp() * 1000)
        
        # Get data using memory store (which handles historical + real-time)
        df = self.data_manager.prepare_strategy_data(symbol, interval, start_ts, end_ts)
        
        return df
    
    def get_latest_price(self, symbol: str, interval: str = '1m') -> Optional[float]:
        """Get latest price for a symbol"""
        store = self.data_manager.get_kline_store(symbol, interval)
        latest = store.get_latest_kline()
        
        if latest:
            return latest['close']
        return None
    
    def get_latest_depth(self, symbol: str) -> Optional[Dict]:
        """Get latest depth data for a symbol"""
        store = self.data_manager.get_depth_store(symbol)
        return store.get_latest_depth()
    
    def subscribe_realtime(self, symbol: str, interval: str):
        """Subscribe to real-time data (handled by data service)"""
        # This is handled by the RealtimeDataService
        # Just ensure we have a store ready
        self.data_manager.get_kline_store(symbol, interval)
        print(f"📡 Ready to receive real-time data for {symbol} {interval}")
    
    def preload_data(self, symbol: str, interval: str, days: int = 30):
        """Preload historical data for faster strategy startup"""
        end_time = datetime.now()
        start_time = end_time - timedelta(days=days)
        
        start_ts = int(start_time.timestamp() * 1000)
        end_ts = int(end_time.timestamp() * 1000)
        
        print(f"📥 Preloading {days} days of data for {symbol} {interval}")
        
        store = self.data_manager.get_kline_store(symbol, interval)
        count = store.load_historical_data(start_ts, end_ts)
        
        print(f"✅ Preloaded {count} klines")
        return count
    
    def get_data_status(self) -> Dict:
        """Get status of all data stores"""
        return self.data_manager.get_status()
    
    def _interval_to_minutes(self, interval: str) -> int:
        """Convert interval string to minutes"""
        interval_map = {
            '1m': 1,
            '3m': 3,
            '5m': 5,
            '15m': 15,
            '30m': 30,
            '1h': 60,
            '2h': 120,
            '4h': 240,
            '6h': 360,
            '8h': 480,
            '12h': 720,
            '1d': 1440,
            '3d': 4320,
            '1w': 10080,
        }
        return interval_map.get(interval, 1)
    
    def close(self):
        """Close data connections"""
        self.data_query.close()
        print("🔒 Strategy Data Interface closed")


class StrategyDataProvider:
    """Simplified data provider for strategy use"""
    
    def __init__(self, symbol: str, interval: str, csv_dir: Optional[str] = None):
        self.symbol = symbol
        self.interval = interval
        self.interface = StrategyDataInterface(csv_dir)
        
        # Preload some data
        self.interface.preload_data(symbol, interval, days=7)
        self.interface.subscribe_realtime(symbol, interval)
    
    def get_data(self, count: int = 1000) -> pd.DataFrame:
        """Get recent kline data"""
        return self.interface.get_klines(self.symbol, self.interval, count=count)
    
    def get_data_range(self, start_time: datetime, end_time: datetime) -> pd.DataFrame:
        """Get kline data for specific time range"""
        return self.interface.get_klines(self.symbol, self.interval, start_time, end_time)
    
    def get_latest_price(self) -> Optional[float]:
        """Get latest price"""
        return self.interface.get_latest_price(self.symbol, self.interval)
    
    def get_latest_depth(self) -> Optional[Dict]:
        """Get latest depth"""
        return self.interface.get_latest_depth(self.symbol)
    
    def close(self):
        """Close provider"""
        self.interface.close()


# Example usage in strategy
class ExampleStrategy:
    """Example strategy using the data interface"""
    
    def __init__(self, symbol: str = "btcusdt", interval: str = "1m"):
        self.symbol = symbol
        self.interval = interval
        
        # Initialize data provider
        self.data_provider = StrategyDataProvider(symbol, interval)
        
        print(f"🎯 Strategy initialized for {symbol} {interval}")
    
    def run(self):
        """Run strategy"""
        print("🚀 Strategy running...")
        
        # Get historical data
        df = self.data_provider.get_data(count=100)
        print(f"📊 Got {len(df)} historical klines")
        
        if not df.empty:
            print(f"📈 Latest price: {df['close'].iloc[-1]}")
            print(f"📊 Price range: {df['low'].min()} - {df['high'].max()}")
        
        # Get latest depth
        depth = self.data_provider.get_latest_depth()
        if depth:
            print(f"📈 Best bid: {depth['best_bid']}, Best ask: {depth['best_ask']}")
        
        # Strategy logic would go here...
        
    def stop(self):
        """Stop strategy"""
        self.data_provider.close()
        print("🛑 Strategy stopped")


if __name__ == "__main__":
    # Example usage
    strategy = ExampleStrategy()
    try:
        strategy.run()
    finally:
        strategy.stop()