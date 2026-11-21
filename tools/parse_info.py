#!/usr/bin/env python3
"""Parse `/occultism info`'s line into shell assignments.

`Spirit Fire at [200, 101, 0]: block=occultism:spirit_fire, colour=yellow,
last=occultism:awakened_feather x3, rules=26/27`

The conversion count is gone in 0.2.1: the diagnostic is a bounded ring, not a
per-fire counter (the review finding), so only the newest conversion is reported.

Prints `ok=1` plus `BLOCK=… COLOUR=… LAST=… RULES=… FILES=…`
(or `ok=0`), for the e2e to `eval`.
"""

from __future__ import annotations

import re
import sys

LINE = re.compile(
    r"^Spirit Fire at \[(?P<x>-?\d+), (?P<y>-?\d+), (?P<z>-?\d+)\]: "
    r"block=(?P<block>\S+), colour=(?P<colour>\S+), "
    r"last=(?P<last>.+?), rules=(?P<rules>\d+)/(?P<files>\d+)$"
)


def main() -> None:
    line = sys.argv[1] if len(sys.argv) > 1 else ""
    match = LINE.match(line)
    if not match:
        print("ok=0")
        return
    print("ok=1")
    for field in ("x", "y", "z", "rules", "files"):
        print(f"{field.upper()}={match[field]}")
    print(f"BLOCK={match['block']!r}")
    print(f"COLOUR={match['colour']!r}")
    print(f"LAST={match['last']!r}")


if __name__ == "__main__":
    main()
