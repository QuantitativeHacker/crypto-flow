#!/usr/bin/env python3
"""Demo strategy using the new data service"""

import sys
from pathlib import Path

# Add pyalgo to path
sys.path.insert(0, str(Path(__file__).parent.parent / "pyalgo" / "python"))

from pyalgo.data.strategy_interface import StrategyDataProvider
from datetime import datetime, timedelta
import pandas as pd
import numpy as np
import time


class DataServiceDemo:
    """Demo strategy showcasing data service capabilities"""
    
    def __init__(self, symbol: str = "btcusdt", interval: str = "1m"):
        self.symbol = symbol
        self.interval = interval
        
        print(f"🎯 Initializing demo strategy for {symbol} {interval}")
        
        # Initialize data provider
        self.data_provider = StrategyDataProvider(symbol, interval)
        
        # Strategy state
        self.running = False
        
    def demonstrate_historical_data(self):
        """Demonstrate historical data access"""
        print("\n📊 === Historical Data Demo ===")
        
        # Get recent data
        df = self.data_provider.get_data(count=100)
        print(f"📈 Retrieved {len(df)} recent klines")
        
        if not df.empty:
            print(f"📅 Time range: {df.index[0]} to {df.index[-1]}")
            print(f"💰 Price range: {df['low'].min():.6f} - {df['high'].max():.6f}")
            print(f"📊 Volume range: {df['volume'].min():.2f} - {df['volume'].max():.2f}")
            
            # Calculate some basic statistics
            returns = df['close'].pct_change().dropna()
            print(f"📈 Average return: {returns.mean():.6f}")
            print(f"📊 Return volatility: {returns.std():.6f}")
            
            # Show latest data
            print(f"\n🕐 Latest kline:")
            latest = df.iloc[-1]
            print(f"  Time: {latest.name}")
            print(f"  OHLC: {latest['open']:.6f} / {latest['high']:.6f} / {latest['low']:.6f} / {latest['close']:.6f}")
            print(f"  Volume: {latest['volume']:.2f}")
    
    def demonstrate_time_range_query(self):
        """Demonstrate time range queries"""
        print("\n🕐 === Time Range Query Demo ===")
        
        # Query last 6 hours
        end_time = datetime.now()
        start_time = end_time - timedelta(hours=6)
        
        print(f"📅 Querying data from {start_time} to {end_time}")
        
        df = self.data_provider.get_data_range(start_time, end_time)
        print(f"📊 Retrieved {len(df)} klines for 6-hour period")
        
        if not df.empty:
            # Calculate hourly statistics
            hourly = df.resample('1H').agg({
                'open': 'first',
                'high': 'max', 
                'low': 'min',
                'close': 'last',
                'volume': 'sum'
            }).dropna()
            
            print(f"\n📊 Hourly summary:")
            for idx, row in hourly.iterrows():
                change = ((row['close'] - row['open']) / row['open']) * 100
                print(f"  {idx.strftime('%H:%M')}: {row['close']:.6f} ({change:+.2f}%) Vol: {row['volume']:.0f}")
    
    def demonstrate_real_time_data(self):
        """Demonstrate real-time data access"""
        print("\n🔴 === Real-time Data Demo ===")
        print("📡 Monitoring real-time data for 30 seconds...")
        
        start_time = time.time()
        last_price = None
        
        while time.time() - start_time < 30:
            # Get latest price
            current_price = self.data_provider.get_latest_price()
            
            if current_price and current_price != last_price:
                # Price changed
                change_str = ""
                if last_price:
                    change = ((current_price - last_price) / last_price) * 100
                    change_str = f" ({change:+.4f}%)"
                
                print(f"💰 {datetime.now().strftime('%H:%M:%S')} - Price: {current_price:.6f}{change_str}")
                last_price = current_price
            
            # Get latest depth
            depth = self.data_provider.get_latest_depth()
            if depth:
                spread = depth['spread']
                spread_pct = (spread / depth['best_bid']) * 100 if depth['best_bid'] > 0 else 0
                print(f"📈 {datetime.now().strftime('%H:%M:%S')} - Bid: {depth['best_bid']:.6f} Ask: {depth['best_ask']:.6f} Spread: {spread:.6f} ({spread_pct:.4f}%)")
            
            time.sleep(2)  # Check every 2 seconds
    
    def demonstrate_technical_analysis(self):
        """Demonstrate technical analysis with the data"""
        print("\n📊 === Technical Analysis Demo ===")
        
        # Get more data for analysis
        df = self.data_provider.get_data(count=200)
        
        if len(df) < 50:
            print("❌ Not enough data for technical analysis")
            return
        
        # Calculate moving averages
        df['ma_20'] = df['close'].rolling(20).mean()
        df['ma_50'] = df['close'].rolling(50).mean()
        
        # Calculate RSI
        delta = df['close'].diff()
        gain = (delta.where(delta > 0, 0)).rolling(14).mean()
        loss = (-delta.where(delta < 0, 0)).rolling(14).mean()
        rs = gain / loss
        df['rsi'] = 100 - (100 / (1 + rs))
        
        # Calculate Bollinger Bands
        df['bb_middle'] = df['close'].rolling(20).mean()
        bb_std = df['close'].rolling(20).std()
        df['bb_upper'] = df['bb_middle'] + (bb_std * 2)
        df['bb_lower'] = df['bb_middle'] - (bb_std * 2)
        
        # Get latest values
        latest = df.iloc[-1]
        
        print(f"📊 Technical Indicators (Latest):")
        print(f"  Price: {latest['close']:.6f}")
        print(f"  MA20: {latest['ma_20']:.6f}")
        print(f"  MA50: {latest['ma_50']:.6f}")
        print(f"  RSI: {latest['rsi']:.2f}")
        print(f"  BB Upper: {latest['bb_upper']:.6f}")
        print(f"  BB Lower: {latest['bb_lower']:.6f}")
        
        # Generate simple signals
        signals = []
        
        if latest['close'] > latest['ma_20'] > latest['ma_50']:
            signals.append("🟢 Bullish trend (Price > MA20 > MA50)")
        elif latest['close'] < latest['ma_20'] < latest['ma_50']:
            signals.append("🔴 Bearish trend (Price < MA20 < MA50)")
        
        if latest['rsi'] > 70:
            signals.append("⚠️ Overbought (RSI > 70)")
        elif latest['rsi'] < 30:
            signals.append("⚠️ Oversold (RSI < 30)")
        
        if latest['close'] > latest['bb_upper']:
            signals.append("📈 Above Bollinger Upper Band")
        elif latest['close'] < latest['bb_lower']:
            signals.append("📉 Below Bollinger Lower Band")
        
        print(f"\n🎯 Signals:")
        for signal in signals:
            print(f"  {signal}")
        
        if not signals:
            print("  📊 No clear signals")
    
    def run_demo(self):
        """Run the complete demo"""
        print(f"🚀 Starting Data Service Demo for {self.symbol} {self.interval}")
        print(f"📅 Demo started at: {datetime.now()}")
        
        try:
            # Demonstrate different capabilities
            self.demonstrate_historical_data()
            self.demonstrate_time_range_query()
            self.demonstrate_technical_analysis()
            self.demonstrate_real_time_data()
            
            print("\n✅ Demo completed successfully!")
            
        except KeyboardInterrupt:
            print("\n⌨️ Demo interrupted by user")
        except Exception as e:
            print(f"\n❌ Demo error: {e}")
        finally:
            self.stop()
    
    def stop(self):
        """Stop the demo"""
        print("\n🛑 Stopping demo...")
        self.data_provider.close()
        print("✅ Demo stopped")


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='Data Service Demo Strategy')
    parser.add_argument('--symbol', default='btcusdt', help='Trading symbol')
    parser.add_argument('--interval', default='1m', help='Kline interval')
    
    args = parser.parse_args()
    
    demo = DataServiceDemo(args.symbol, args.interval)
    demo.run_demo()


if __name__ == "__main__":
    main()