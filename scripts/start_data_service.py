#!/usr/bin/env python3
"""Data service startup script"""

import sys
import os
from pathlib import Path

# Add pyalgo to path
sys.path.insert(0, str(Path(__file__).parent.parent / "pyalgo" / "python"))

from pyalgo.data.realtime_service import RealtimeDataService
import argparse
import signal
import time
from datetime import datetime


def main():
    parser = argparse.ArgumentParser(description='Crypto Data Collection Service')
    parser.add_argument('--config', default='config/data_service.json', 
                       help='Configuration file path')
    parser.add_argument('--status', action='store_true', 
                       help='Show service status and exit')
    parser.add_argument('--test', action='store_true',
                       help='Run in test mode (shorter duration)')
    
    args = parser.parse_args()
    
    print("🚀 Crypto Data Service")
    print(f"📅 Started at: {datetime.now()}")
    print(f"📋 Config: {args.config}")
    
    # Create service instance
    service = RealtimeDataService(args.config)
    
    if args.status:
        status = service.get_status()
        print("\n📊 Service Status:")
        for key, value in status.items():
            print(f"  {key}: {value}")
        return
    
    # Setup signal handlers for graceful shutdown
    def signal_handler(signum, frame):
        print(f"\n📡 Received signal {signum}, shutting down...")
        service.stop()
        sys.exit(0)
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    try:
        if args.test:
            print("🧪 Running in test mode (30 seconds)")
            service.start()
            time.sleep(30)
            service.stop()
        else:
            print("🔄 Running in production mode (press Ctrl+C to stop)")
            service.start()
    
    except KeyboardInterrupt:
        print("\n⌨️ Keyboard interrupt received")
        service.stop()
    except Exception as e:
        print(f"❌ Error: {e}")
        service.stop()
        sys.exit(1)


if __name__ == "__main__":
    main()