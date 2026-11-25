"""node.py against the simulated sensor node: what it prints is what the book shows, in
docs/generated/node/node_py.txt.
"""

import sys

from conftest import HERE, check_output

sys.path.insert(0, str(HERE))
import node  # noqa: E402


def test_node_prints_what_the_book_shows(node_url):
    check_output("node/node_py.txt", "python node.py", lambda: node.main(node_url))
