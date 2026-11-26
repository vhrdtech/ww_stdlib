"""Asks the sensor node what it is, names it, and finds it on the bench, through the standard device contract
traits.

Run the simulated device first (`cargo run -p node --example node_sim`), then
`python node.py`, or `python node.py ws://ADDRESS:PORT/ww` for another address.
"""

import sys

import wire_weaver as ww


def main(url):
    dev = ww.connect(websocket_url=url)

    # --8<-- [start:about]
    about = dev.device
    firmware = about.firmware.FIRMWARE.read()
    board = about.board.BOARD.read()
    v = firmware["version"]
    print("firmware:", firmware["name"], f"{v['major']}.{v['minor']}.{v['patch']}")
    print("board:", board["name"], "rev", board["revision"], "serial", board["serial"])
    # --8<-- [end:about]

    # --8<-- [start:label]
    about.label.name.write("bench 2")
    print("name:", about.label.name.read())
    # --8<-- [end:label]

    # --8<-- [start:identify]
    about.indication.identify(2.0)
    try:
        about.indication.identify(2.0)
    except ww.RemoteError as e:
        print("identify again:", e.common)
    # --8<-- [end:identify]

    dev.disconnect()


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "ws://127.0.0.1:8083/ww")
