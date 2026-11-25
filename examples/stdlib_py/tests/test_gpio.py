"""gpio.py against the simulated GPIO expander: what it prints is what the book shows, in
docs/generated/all_gpio/gpio_py.txt.
"""

import sys

from conftest import HERE, check_output

sys.path.insert(0, str(HERE))
import gpio  # noqa: E402


def test_gpio_prints_what_the_book_shows(gpio_url):
    check_output("all_gpio/gpio_py.txt", "python gpio.py", lambda: gpio.main(gpio_url))
