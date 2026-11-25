# node_api

API of the sensor node example: a device that tells about itself. Its `device` subtree holds the standard library's
device contract (firmware, board, unique id, user label, a text log and the indication), and the root one method of
the node's own.

- Device: `examples/node_device` (simulated on a PC by `examples/node`).
- Host: `examples/node` (the generated client, `node_about`), `ww info` and `examples/stdlib_py` (`node.py`).

The crate is `no_std`; its `std` feature turns on the owned types for host code.
