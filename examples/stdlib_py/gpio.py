"""Blinks the LED of the GPIO expander and reads its button, through the standard ww_gpio calls.

Run the simulated device first (`cargo run -p all_gpio --example all_gpio_sim`), then
`python gpio.py`, or `python gpio.py ws://ADDRESS:PORT/ww` for another address.
"""

import sys

import wire_weaver as ww


def main(url):
    # --8<-- [start:connect]
    dev = ww.connect(websocket_url=url)
    pa = dev.port[0]
    print("port 0:", pa.name())
    # --8<-- [end:connect]

    # --8<-- [start:led]
    led = pa.pin[5]
    led.set_mode("PushPullOutput", "Low")
    for _ in range(3):
        led.toggle()
        print("PA5:", led.output_level())
    # --8<-- [end:led]

    # --8<-- [start:refused]
    try:
        led.set_mode("Analog", None)
    except ww.RemoteError as e:
        print("refused:", e.common)
    print("PA5 mode:", led.mode())
    # --8<-- [end:refused]

    dev.disconnect()


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "ws://127.0.0.1:8081/ww")
