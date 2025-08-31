#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
安全的策略示例 - 只订阅行情，不进行实际交易
适合新手学习和测试
"""

from pyalgo import *
import time

class SafeDemo:
    """安全的演示策略 - 只观察行情，不下单"""
    
    def __init__(self, sub: DepthSubscription):
        self.sub = sub
        self.start_time = time.time()
        self.tick_count = 0
        
        # 设置回调函数
        self.sub.on_data = self.on_depth
        
        print("🔍 安全模式启动 - 只观察行情，不会下单")
        print(f"📊 订阅交易对: {self.sub.symbol}")
        print("=" * 50)
    
    def on_depth(self, depth: Depth):
        """处理行情数据"""
        self.tick_count += 1
        
        # 每10次tick打印一次信息
        if self.tick_count % 10 == 0:
            bid_price = self.sub.bid_prc(0)  # 最优买价
            ask_price = self.sub.ask_prc(0)  # 最优卖价
            spread = ask_price - bid_price   # 买卖价差
            
            print(f"⏰ {self.sub.datetime}")
            print(f"💰 {self.sub.symbol} - 买价: {bid_price:.4f}, 卖价: {ask_price:.4f}")
            print(f"📈 价差: {spread:.4f} ({spread/bid_price*100:.3f}%)")
            print(f"📊 已接收 {self.tick_count} 次行情更新")
            print("-" * 30)

def main():
    """主函数"""
    try:
        # 连接到本地交易服务器
        engine = Engine(0.001)  # 最小价格精度
        session = engine.make_session(
            addr="ws://localhost:8111", 
            session_id=1, 
            name="safe_demo", 
            trading=False  # 安全模式：不允许交易
        )
        
        # 创建深度订阅 (选择一个流动性好的交易对)
        depth_sub = session.subscribe("btcusdt", "depth")
        
        # 启动安全策略
        strategy = SafeDemo(depth_sub)
        
        print("🚀 开始运行...")
        print("💡 提示: 按 Ctrl+C 停止程序")
        
        # 运行引擎
        engine.run()
        
    except KeyboardInterrupt:
        print("\n👋 程序已停止")
    except Exception as e:
        print(f"❌ 错误: {e}")

if __name__ == "__main__":
    main()