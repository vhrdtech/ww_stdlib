# stdlib_py

Python scripts of the standard library examples, each run against its simulated device:

```sh
cargo run -p all_gpio --example all_gpio_sim     # terminal 1
uv run python gpio.py                            # terminal 2, in this directory
```

The crate builds `wire_weaver_py` as the `wire_weaver` module; outside this repository, install the `wire_weaver`
wheel from vhrdtech/ww_tools instead. `just test-py` runs the scripts against the simulated devices and checks their
output against `docs/generated/`.
