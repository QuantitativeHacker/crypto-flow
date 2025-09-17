"""
实时把数据写入CSV文件
这里会利用storage模块来存储数据
"""

from pyalgo import *
from .storage import DataStorage, DataCollector
from typing import List, Dict, Optional
import signal
import sys
from pathlib import Path
import json
from datetime import datetime


class RealtimeDataService:
    
    def __init__(self, config_path: str = "config/data_service.json"):
        self.config = self._load_config(config_path)
        # 初始化数据存储
        self.storage = DataStorage(self.config.get('csv_dir', 'data/csv'))
        # 初始化数据采集器
        self.collector = DataCollector(self.storage)
        
        # pyalgo components
        self.engine = None
        self.session = None
        self.subscriptions = {}
        
        # Service state
        self.active = False
        
        # Setup signal handlers
        signal.signal(signal.SIGTERM, self._signal_handler)
        signal.signal(signal.SIGINT, self._signal_handler)
    
    def _load_config(self, config_path: str) -> Dict:
        """Load service configuration"""
        config_file = Path(config_path)
        
        if config_file.exists():
            try:
                with open(config_file, 'r') as f:
                    config = json.load(f)
                print(f"📋 Loaded config from {config_path}")
                return config
            except Exception as e:
                print(f"⚠️ Error loading config: {e}")
        
        # Default configuration
        default_config = {
            "websocket_addr": "ws://localhost:8111",
            "session_id": 1,
            "session_name": "data_service",
            "trading": False,
            "csv_dir": "data/csv",
            "subscriptions": [
                {"symbol": "btcusdt", "stream": "kline:1m"},
                {"symbol": "ethusdt", "stream": "kline:1m"},
                {"symbol": "btcusdt", "stream": "depth"},
                {"symbol": "ethusdt", "stream": "depth"}
            ],
            "engine_interval": 0.001
        }
        
        # Save default config
        config_file.parent.mkdir(parents=True, exist_ok=True)
        with open(config_file, 'w') as f:
            json.dump(default_config, f, indent=2)
        
        print(f"📋 Created default config at {config_path}")
        return default_config
    
    def start(self):
        """Start the real-time data service"""
        print("🚀 Starting real-time data service...")
        
        try:
            # Initialize pyalgo engine
            self.engine = Engine(self.config.get('engine_interval', 0.001))
            
            # Create session
            self.session = self.engine.make_session(
                addr=self.config['websocket_addr'],
                session_id=self.config['session_id'],
                name=self.config['session_name'],
                trading=self.config['trading']
            )
            
            print(f"🔗 Connected to {self.config['websocket_addr']}")
            
            # Setup subscriptions
            self._setup_subscriptions()
            
            # Start data collection
            self.active = True
            print("✅ Data service started successfully")
            print("📊 Collecting real-time data... (Press Ctrl+C to stop)")
            
            # Run the engine
            self.engine.run()
            
        except Exception as e:
            print(f"❌ Error starting service: {e}")
            self.stop()
    
    def _setup_subscriptions(self):
        """Setup market data subscriptions"""
        subscriptions_config = self.config.get('subscriptions', [])
        
        for sub_config in subscriptions_config:
            symbol = sub_config['symbol']
            stream = sub_config['stream']
            
            try:
                # Subscribe using pyalgo
                subscription = self.session.subscribe(symbol, stream)
                
                # Create wrapper to handle data
                if stream.startswith('kline'):
                    wrapper = KlineSubscriptionWrapper(subscription, self.collector)
                elif stream.startswith('depth') or stream == 'bbo':
                    wrapper = DepthSubscriptionWrapper(subscription, self.collector)
                else:
                    print(f"⚠️ Unsupported stream type: {stream}")
                    continue
                
                self.subscriptions[f"{symbol}@{stream}"] = wrapper
                print(f"📡 Subscribed to {symbol}@{stream}")
                
            except Exception as e:
                print(f"❌ Failed to subscribe to {symbol}@{stream}: {e}")
    
    def stop(self):
        """Stop the data service"""
        print("\n🛑 Stopping data service...")
        
        self.active = False
        
        if self.collector:
            self.collector.stop()
        
        if self.storage:
            self.storage.close()
        
        if self.engine:
            self.engine.stop()
        
        print("✅ Data service stopped")
    
    def _signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        print(f"\n📡 Received signal {signum}")
        self.stop()
        sys.exit(0)
    
    def get_status(self) -> Dict:
        """Get service status"""
        status = {
            'active': self.active,
            'subscriptions': list(self.subscriptions.keys()),
            'storage_stats': self.storage.get_storage_stats() if self.storage else {},
            'session_connected': self.session.is_login if self.session else False
        }
        return status


class KlineSubscriptionWrapper:
    """Wrapper for kline subscriptions to handle data collection"""
    
    def __init__(self, subscription: BarSubscription, collector: DataCollector):
        self.subscription = subscription
        self.collector = collector
        
        # Set callback
        self.subscription.on_data = self._on_kline_data
    
    def _on_kline_data(self, kline: Kline):
        """Handle incoming kline data"""
        try:
            # Pass to collector
            self.collector.on_kline(kline)
            
            # Log every 10th kline to avoid spam
            if hasattr(self, '_kline_count'):
                self._kline_count += 1
            else:
                self._kline_count = 1
            

            status = "🟢CLOSED" if kline.is_closed else "🔵LIVE"
            print(f"📊 {status} Kline: {kline.symbol} {kline.interval} {kline.datetime} Close: {kline.close}")
        
        except Exception as e:
            print(f"❌ Error processing kline: {e}")


class DepthSubscriptionWrapper:
    """Wrapper for depth subscriptions to handle data collection"""
    
    def __init__(self, subscription: DepthSubscription, collector: DataCollector):
        self.subscription = subscription
        self.collector = collector
        
        # Set callback
        self.subscription.on_data = self._on_depth_data
    
    def _on_depth_data(self, depth: Depth):
        """Handle incoming depth data"""
        try:
            # Pass to collector
            self.collector.on_depth(depth)
            
            # Log every 100th depth to avoid spam
            if hasattr(self, '_depth_count'):
                self._depth_count += 1
            else:
                self._depth_count = 1
            
            if self._depth_count % 100 == 0:
                bid_price = depth.bid_prc(0) if depth.bid_level > 0 else 0
                ask_price = depth.ask_prc(0) if depth.ask_level > 0 else 0
                print(f"📈 Depth: {depth.symbol} {depth.datetime} Bid: {bid_price} Ask: {ask_price}")
        
        except Exception as e:
            print(f"❌ Error processing depth: {e}")


def main():
    """Main entry point for the data service"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Real-time crypto data collection service')
    parser.add_argument('--config', default='config/data_service.json', help='Configuration file path')
    parser.add_argument('--status', action='store_true', help='Show service status and exit')
    
    args = parser.parse_args()
    
    service = RealtimeDataService(args.config)
    
    if args.status:
        status = service.get_status()
        print("📊 Service Status:")
        for key, value in status.items():
            print(f"  {key}: {value}")
        return
    
    try:
        service.start()
    except KeyboardInterrupt:
        service.stop()


if __name__ == "__main__":
    main()