"""uart.py against the simulated UART bridge: what it prints is what the book shows, in
docs/generated/uart/uart_py.txt.
"""

import sys

from conftest import HERE, check_output

sys.path.insert(0, str(HERE))
import uart  # noqa: E402


def test_uart_prints_what_the_book_shows(uart_url):
    check_output("uart/uart_py.txt", "python uart.py", lambda: uart.main(uart_url))
