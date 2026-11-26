"""Reads the GPS receiver on port 0 of the UART bridge and sends a line through the loopback of port 1, through
the standard ww_uart calls.

Run the simulated device first (`cargo run -p uart --example uart_sim`), then
`python uart.py`, or `python uart.py ws://ADDRESS:PORT/ww` for another address.
"""

import sys

import wire_weaver as ww


def main(url):
    dev = ww.connect(websocket_url=url)

    # --8<-- [start:configure]
    gps = dev.uart[0]
    line = {
        "baud_rate": 9600,
        "data_bits": "Eight",
        "parity": "None",
        "stop_bits": "Stop1",
        "mode": "Asynchronous",
        "prevent_back_feed": False,
        "io_level": None,
        "idle_chars": 2,
    }
    print("port 0:", gps.configure(line)["baud_rate"])
    # --8<-- [end:configure]

    # --8<-- [start:rx]
    with gps.rx as rx:
        for _ in range(2):
            chunk = rx.recv(timeout=3)
            print(bytes(chunk["bytes"]).decode().rstrip())
    # --8<-- [end:rx]

    # --8<-- [start:loopback]
    jumper = dev.uart[1]
    with jumper.rx as echo, jumper.tx as tx:
        tx.send(b"hello from Python\n")
        print("port 1:", bytes(echo.recv(timeout=3)["bytes"]).decode().rstrip())
    # --8<-- [end:loopback]

    dev.disconnect()


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "ws://127.0.0.1:8082/ww")
