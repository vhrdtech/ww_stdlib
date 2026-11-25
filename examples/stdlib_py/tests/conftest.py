"""Fixtures of the standard library examples' scripts: each simulated device on a free port, and the check of a
script's output against what the book shows under docs/generated/.
"""

import contextlib
import io
import os
import subprocess
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parents[1]
ROOT = HERE.parents[1]
GENERATED = ROOT / "docs" / "generated"


def simulated(package, example):
    """Runs `cargo run -p PACKAGE --example EXAMPLE -- --port 0` and yields the URL its first line names."""
    sim = subprocess.Popen(
        ["cargo", "run", "-q", "-p", package, "--example", example, "--", "--port", "0"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        text=True,
    )
    try:
        line = sim.stdout.readline()
        prefix = f"{package} at "
        assert line.startswith(prefix), line
        yield line.removeprefix(prefix).strip()
    finally:
        sim.terminate()
        sim.wait(timeout=10)


@pytest.fixture
def gpio_url():
    """A simulated GPIO expander, stopped after the test."""
    yield from simulated("all_gpio", "all_gpio_sim")


@pytest.fixture
def uart_url():
    """A simulated UART bridge, stopped after the test."""
    yield from simulated("uart", "uart_sim")


def check_output(name, command, run):
    """Runs `run()`, and checks what it printed, after the `$ command` line, against docs/generated/NAME.
    WW_BOOK_OUTPUTS=write (`just book-outputs`) writes the file instead."""
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        run()
    text = f"$ {command}\n" + out.getvalue()
    output = GENERATED / name
    if os.environ.get("WW_BOOK_OUTPUTS") == "write":
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text)
    assert output.read_text() == text, f"{output} is stale: run `just book-outputs`"
