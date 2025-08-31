from pyalgo import *


class Demo:
    """"""

    def __init__(self, sub: DepthSubscription):
        self.sub = sub
        # use to send/kill order
        self.smtord = SmartOrder(sub)

        self.fin = False

        # add trading phase if you need
        # otherwise self.sub.phase is always Phase.UNDEF
        # 设置全天24小时都是开盘阶段（加密货币交易）
        self.sub.add_phase(8, 0, 0, Phase.OPEN)    # 0点开始开盘
        # self.sub.add_phase(16, 0, 0, Phase.CLOSE)  # 注释掉收盘阶段

        # 调试：打印阶段设置
        print(f"=== 策略初始化 ===")
        print(f"订阅对象: {self.sub}")
        print(f"阶段设置完成")

        # set callback
        self.sub.on_data = self.on_depth
        self.sub.on_order = self.on_order

    def on_order(self, order: Order):
        print(order)

    def on_depth(self, depth: Depth):
        # 调试：详细的时间信息
        current_time = self.sub.datetime
        print(f"=== 深度回调 ===")
        print(f"时间: {current_time}")
        print(f"时间类型: {type(current_time)}")
        print(f"时区: {getattr(current_time, 'tzinfo', 'None')}")
        print(f"小时: {current_time.hour}, 分钟: {current_time.minute}")
        print(f"时间: {self.sub.datetime}, 标的: {self.sub.symbol}")
        print(f"当前持仓: {self.sub.net}")
        print(f"当前阶段: {self.sub.phase}")
        print(f"阶段类型: {type(self.sub.phase)}")
        print(f"完成标志: {self.fin}")
        print(f"订单状态: {self.smtord.is_active}")

        if self.fin:
            self.smtord.kill()
            return

        match self.sub.phase:
            case Phase.OPEN:
                print("进入开盘阶段")
                if self.sub.net == 0:
                    print("无持仓，准备下单")
                    if not self.fin:
                        if self.smtord.is_active:
                            print("有活跃订单，先撤单")
                            self.smtord.kill()
                        else:
                            print("开始下单...")
                            self.smtord.send(
                                self.sub.bid_prc(5),
                                30,
                                Side.BUY,
                                OrderType.LIMIT,
                                Tif.GTC,
                            )
                            self.fin = True
                            print("下单完成")
                else:
                    print(f"已有持仓: {self.sub.net}")

            case Phase.CLOSE:
                print("进入收盘阶段")
                pass
            case Phase.UNDEF:
                print("⚠️ 警告：当前阶段为 UNDEF，可能的原因：")
                print("  1. 时间解析问题")
                print("  2. 阶段设置未生效")
                print("  3. pyalgo 版本问题")
            case _:
                print(f"未知阶段: {self.sub.phase}")

if __name__ == "__main__":
    eng = Engine(0.001)
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )

    sub = session.subscribe("dogeusdt", "depth")
    demo = Demo(sub)
    eng.run()
