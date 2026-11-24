#!/usr/bin/env python3
"""Craft the two `otherstone_tablet` recipes at a real table (slice 13, phase Y).

Called by `tools/e2e.sh` with the open game's control socket and the crafting
table's position. The harness opens a real table whose 3x3 grid is slots 100..108
(row-major) and whose result is 200; the player's own slots keep their inventory
numbers.

The slice, read from the jar at `release/v1.21.1-1.225.0`:

* `otherstone_tablet.json` and `otherstone_tablet2.json` are both shaped
  `aga`/`gsg`/`aga` grids: `a` = `#c:dusts/otherworld_wood` (the ashes carried
  in 0.2.13), `g` = `#c:ingots/gold` (carried in 0.2.12), `s` =
  `occultism:otherstone` in the first file and `occultism:otherrock` in the
  second. Both make exactly one `occultism:otherstone_tablet`.

What is proven:

1. each grid, filled exactly as its recipe declares (four ashes, four gold,
   one stone), makes exactly one tablet, which is taken into the inventory;
   every ingredient is consumed and the grid is emptied;
2. a three-ashes grid (no gold, no stone) makes nothing;
3. `TABLET_SELFTEST=1` asserts the first grid makes two, so a working package
   must FAIL this run (the run's `EXIT:1` is its fail-capability proof).

The result is read from **slot 200 of the open table screen** — `ctl screen`
also answers the plain inventory menu, so a screen without slot 200 is a
harness error, not a "nothing was crafted" result. The player is in survival,
so the ingredient consumption is vanilla's rule. Exit 1 with the observed
state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200

# (file, stone, the cells its key letters occupy).
GRIDS = [
    ("otherstone_tablet", "occultism:otherstone"),
    ("otherstone_tablet2", "occultism:otherrock"),
]
ASH = "occultism:otherworld_ashes"
GOLD = "minecraft:gold_ingot"
RESULT_ID = "occultism:otherstone_tablet"
# aga / gsg / aga, row-major.
CELLS = {
    ASH: [GRID[0], GRID[2], GRID[6], GRID[8]],
    GOLD: [GRID[1], GRID[3], GRID[5], GRID[7]],
}
STONE_CELL = [GRID[4]]


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    selftest = os.environ.get("TABLET_SELFTEST") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"TABLET FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def screen_slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"TABLET FAIL: no menu is open: {screen}")
        return screen["slots"]

    def require_table() -> None:
        """`ctl screen` also answers the inventory menu: only a table has 200."""
        numbers = {int(slot["slot"]) for slot in screen_slots()}
        missing = [n for n in (*GRID, RESULT) if n not in numbers]
        if missing:
            raise SystemExit(
                f"TABLET FAIL: no crafting table screen (missing slots {missing}); "
                "the result must be read from slot 200, not from the inventory menu"
            )

    def slot_of(number: int) -> dict:
        return next((s for s in screen_slots() if s["slot"] == number), {})

    def item_of(number: int) -> str | None:
        return slot_of(number).get("item") or None

    def count_of(number: int) -> int:
        return int(slot_of(number).get("count") or 0)

    def inventory() -> str:
        return json.dumps(run("inventory")["counts"], sort_keys=True)

    def in_inventory(item: str) -> int:
        return int(run("inventory")["counts"].get(item, 0))

    def find(item: str) -> int:
        for slot in screen_slots():
            if slot.get("item") == item:
                return int(slot["slot"])
        raise SystemExit(
            f"TABLET FAIL: missing ingredient {item} in the open screen "
            f"(inventory: {inventory()})"
        )

    def empty_or_same(item: str) -> int:
        free = None
        for slot in screen_slots():
            number = int(slot["slot"])
            if number > 35:
                continue
            got = slot.get("item")
            if got == item and int(slot.get("count") or 0) < 64:
                return number
            if got is None and free is None:
                free = number
        if free is None:
            raise SystemExit(f"TABLET FAIL: no inventory slot for {item} ({inventory()})")
        return free

    def place(cells_by_item: dict[str, list[int]]) -> None:
        for item, cells in cells_by_item.items():
            source = find(item)
            run("click-slot", str(source))
            wait(1)
            for cell in cells:
                run("click-slot", str(cell), "right")
                wait(1)
            run("click-slot", str(source))
            wait(1)

    def close_table() -> None:
        raw("close")
        wait(2)

    def reset_table() -> None:
        close_table()
        run("setblock", str(x), str(y), str(z), "minecraft:air")
        run("setblock", str(x), str(y), str(z), "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) + 0.5} {y} {float(z) + 3.5}")
        wait(3)
        run("interact", str(x), str(y), str(z))
        wait(5)
        require_table()

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        require_table()
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"TABLET FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot (200) stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"TABLET FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
            )
        print(f"  {what}: the result slot (200) reads {got} x{got_count}")

    def take_result(item: str, what: str) -> None:
        target = empty_or_same(item)
        run("click-slot", str(RESULT))
        wait(1)
        run("click-slot", str(target))
        wait(2)
        print(f"  {what}: taken into slot {target} (inventory {inventory()})")

    def expect_consumed(items: dict[str, int], what: str) -> None:
        for item, left in items.items():
            got = in_inventory(item)
            if got != left:
                raise SystemExit(
                    f"TABLET FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    def expect_grid_empty(what: str) -> None:
        for number in GRID:
            got = item_of(number)
            if got is not None:
                raise SystemExit(f"TABLET FAIL: {what} left {got} in grid slot {number}")
        print(f"  {what}: the grid is empty")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("TABLET FAIL: the tablet phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    # ---- the two jar grids, one fresh table each -----------------------------
    print("== the two otherstone_tablet grids ==")
    for index, (file, stone) in enumerate(GRIDS):
        close_table()
        fresh_inventory()
        cells = {ASH: list(CELLS[ASH]), GOLD: list(CELLS[GOLD]), stone: list(STONE_CELL)}
        for item, positions in cells.items():
            run("give", item, str(len(positions)))
            wait(2)
        reset_table()
        place(cells)
        what = f"the {file} grid"
        if selftest and index == 0:
            print("  (selftest: asserting the first tablet grid makes two)")
            expect_result(RESULT_ID, 2, f"selftest: {what}")
            continue
        expect_result(RESULT_ID, 1, what)
        take_result(RESULT_ID, "the tablet")
        expect_consumed({item: 0 for item in cells}, "the tablet recipe")
        expect_grid_empty("the tablet recipe")
        if in_inventory(RESULT_ID) != 1:
            raise SystemExit(
                f"TABLET FAIL: the crafted tablet is not in the inventory ({inventory()})"
            )

    # ---- the wrong grid: three ashes make nothing ----------------------------
    print("== the wrong grid: three ashes make nothing ==")
    close_table()
    fresh_inventory()
    wrong = {ASH: [GRID[0], GRID[3], GRID[6]]}
    for item, positions in wrong.items():
        run("give", item, str(len(positions)))
        wait(2)
    reset_table()
    place(wrong)
    expect_result(None, 0, "three ashes in the first column")
    close_table()
    return 0


if __name__ == "__main__":
    sys.exit(main())
