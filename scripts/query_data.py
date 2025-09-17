#!/usr/bin/env python3
"""Data query utility script"""

import sys
import os
from pathlib import Path

# Add pyalgo to path
sys.path.insert(0, str(Path(__file__).parent.parent / "pyalgo" / "python"))

from pyalgo.data.query import DataQuery
import argparse
import pandas as pd
from datetime import datetime, timedelta
import json


def main():
    parser = argparse.ArgumentParser(description='Query crypto data')
    parser.add_argument('--symbol', required=True, help='Trading symbol (e.g., btcusdt)')
    parser.add_argument('--interval', default='1m', help='Kline interval (e.g., 1m, 5m, 1h)')
    parser.add_argument('--start', help='Start time (YYYY-MM-DD HH:MM:SS or timestamp)')
    parser.add_argument('--end', help='End time (YYYY-MM-DD HH:MM:SS or timestamp)')
    parser.add_argument('--days', type=int, default=1, help='Number of days back from now')
    parser.add_argument('--output', help='Output file path (CSV format)')
    parser.add_argument('--format', choices=['csv', 'json', 'summary'], default='summary',
                       help='Output format')
    parser.add_argument('--csv-dir', default='data/csv', help='CSV data directory')
    parser.add_argument('--no-fetch', action='store_true', 
                       help='Disable auto-fetch from Binance')
    
    args = parser.parse_args()
    
    # Parse time arguments
    if args.start and args.end:
        start_time = parse_time(args.start)
        end_time = parse_time(args.end)
    else:
        # Use days parameter
        end_time = int(datetime.now().timestamp() * 1000)
        start_time = int((datetime.now() - timedelta(days=args.days)).timestamp() * 1000)
    
    print(f"🔍 Querying data for {args.symbol} {args.interval}")
    print(f"📅 Time range: {datetime.fromtimestamp(start_time/1000)} to {datetime.fromtimestamp(end_time/1000)}")
    
    # Create query instance (CSV-only)
    query = DataQuery(args.csv_dir)
    
    try:
        # Get klines
        klines = query.get_klines(
            symbol=args.symbol,
            interval=args.interval,
            start_time=start_time,
            end_time=end_time,
            auto_fetch=not args.no_fetch
        )
        
        if not klines:
            print("❌ No data found")
            return
        
        # Convert to DataFrame
        df = pd.DataFrame(klines)
        df['datetime'] = pd.to_datetime(df['open_time'], unit='ms')
        
        # Output results
        if args.format == 'summary':
            print_summary(df, args.symbol, args.interval)
        elif args.format == 'csv':
            output_csv(df, args.output or f"{args.symbol}_{args.interval}_data.csv")
        elif args.format == 'json':
            output_json(klines, args.output or f"{args.symbol}_{args.interval}_data.json")
        
    except Exception as e:
        print(f"❌ Error: {e}")
    finally:
        query.close()


def parse_time(time_str: str) -> int:
    """Parse time string to timestamp in milliseconds"""
    try:
        # Try parsing as timestamp first
        if time_str.isdigit():
            return int(time_str)
        
        # Try parsing as datetime string
        dt = pd.to_datetime(time_str)
        return int(dt.timestamp() * 1000)
    except:
        raise ValueError(f"Invalid time format: {time_str}")


def print_summary(df: pd.DataFrame, symbol: str, interval: str):
    """Print data summary"""
    print(f"\n📊 Data Summary for {symbol} {interval}")
    print(f"📈 Records: {len(df)}")
    print(f"📅 Time range: {df['datetime'].min()} to {df['datetime'].max()}")
    print(f"💰 Price range: {df['low'].min():.8f} - {df['high'].max():.8f}")
    print(f"📊 Volume range: {df['volume'].min():.2f} - {df['volume'].max():.2f}")
    
    # Recent data
    print(f"\n🕐 Latest 5 records:")
    recent = df.tail(5)[['datetime', 'open', 'high', 'low', 'close', 'volume']]
    print(recent.to_string(index=False))


def output_csv(df: pd.DataFrame, filename: str):
    """Output data to CSV file"""
    df.to_csv(filename, index=False)
    print(f"💾 Data saved to {filename}")


def output_json(data: list, filename: str):
    """Output data to JSON file"""
    with open(filename, 'w') as f:
        json.dump(data, f, indent=2)
    print(f"💾 Data saved to {filename}")


if __name__ == "__main__":
    main()