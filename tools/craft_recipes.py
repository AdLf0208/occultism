#!/usr/bin/env python3
"""Craft the two carried crafting recipes in a real crafting table.

Called by `tools/e2e.sh` (phase P) with the open game's control socket and the
table's position. The harness opens a crafting table as a screen whose 3x3 grid
is slots 100..108 (row-major) and whose result is 200; the player's own slots
keep their inventory numbers (hotbar 0..8).

The slice's rule: a crafting recipe is carried only when its **result and every
ingredient** the package (or vanilla) already provides. Two jar recipes meet it:

* `demons_dream_essence_from_fruit_or_seed` - the jar's shaped `ppp/ppp/ppp`
  whose key is the alternative `#c:crops/datura` / `#c:seeds/datura`, making one
  `occultism:demons_dream_essence`. It is the anchor: the fruit and its seed
  landed in 0.2.3, so this is the first version in which the essence is
  obtainable at all.
* `book_of_binding_empty` - the jar's shapeless awakened feather + purified ink
  + taboo book.

What is proven, in this order:

1. nine datura fruit craft exactly one essence and the fruit is consumed;
2. nine datura seeds craft exactly one too (the alternative's other side);
3. a mixed grid (five fruit, four seeds) still crafts one - the alternative is
   per cell, not per recipe;
4. a differential: nine dirt make nothing, and eight fruit with one dirt in the
   last cell make nothing either, so the result is the recipe and not "any 3x3";
5. the shapeless book recipe makes one book and consumes its three items.

`CRAFT_NEGATIVE=1` inverts assertion 1 (a grid of fruit must make **nothing**),
so the run must fail: that is this phase's counterfactual.

Exit 1 with the observed state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200
ESSENCE = "occultism:demons_dream_essence"
FRUIT = "occultism:datura"
SEEDS = "occultism:datura_seeds"
BOOK = "occultism:book_of_binding_empty"


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]
    negative = os.environ.get("CRAFT_NEGATIVE") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"CRAFT FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"CRAFT FAIL: no menu is open: {screen}")
        return screen["slots"]

    def slot_of(number: int) -> dict:
        found = [s for s in slots() if s["slot"] == number]
        return found[0] if found else {}

    def item_of(number: int) -> str | None:
        return slot_of(number).get("item") or None

    def count_of(number: int) -> int:
        return int(slot_of(number).get("count") or 0)

    def in_inventory(item: str) -> int:
        return int(run("inventory")["counts"].get(item, 0))

    def inventory() -> str:
        return json.dumps(run("inventory")["counts"], sort_keys=True)

    def find(item: str) -> int:
        for slot in slots():
            if slot.get("item") == item:
                return int(slot["slot"])
        raise SystemExit(
            f"CRAFT FAIL: missing ingredient {item} in the open screen "
            f"(inventory: {inventory()})"
        )

    def place(item: str, targets: list[int]) -> None:
        """Pick the item up, put one in each target, put the rest back."""
        source = find(item)
        run("click-slot", str(source))
        wait(1)
        for target in targets:
            run("click-slot", str(target), "right")
            wait(1)
        run("click-slot", str(source))
        wait(1)

    def close_table() -> None:
        try:
            run("close")
        except SystemExit:
            pass
        wait(2)

    def reset_table() -> None:
        """A fresh table with an empty grid, and the player back in reach."""
        close_table()
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) + 0.5} {y} {float(z) + 3.5}")
        wait(3)
        run("interact", x, y, z)
        wait(5)
        if not slots():
            raise SystemExit("CRAFT FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def give(item: str, count: int) -> None:
        run("give", item, str(count))

    def expect_result(want: str | None, count: int, what: str) -> None:
        got = item_of(RESULT)
        got_count = count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"CRAFT FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"CRAFT FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
            )
        print(f"  {what}: the result slot reads {got} x{got_count}")

    def take_result(what: str) -> None:
        run("click-slot", str(RESULT))
        wait(1)
        run("click-slot", "0")
        wait(2)
        print(f"  {what}: taken into the hotbar (inventory {inventory()})")

    def expect_consumed(items: dict[str, int], what: str) -> None:
        for item, left in items.items():
            got = in_inventory(item)
            if got != left:
                raise SystemExit(
                    f"CRAFT FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    # Survival: the creative inventory's own grid is endless and would hide a
    # consumption bug; the crafting table consumes in every mode, but survival is
    # the mode a player actually crafts in.
    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("CRAFT FAIL: the crafting phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    # 1. Nine datura fruit -> one essence (the anchor).
    close_table()
    fresh_inventory()
    give(FRUIT, 9)
    wait(3)
    reset_table()
    place(FRUIT, GRID)
    if negative:
        print("  (negative: asserting a 3x3 of fruit makes nothing)")
        expect_result(None, 0, "negative: the 3x3 fruit grid")
        return 1
    expect_result(ESSENCE, 1, "the 3x3 fruit grid")
    take_result("the essence")
    expect_consumed({FRUIT: 0}, "the fruit grid")
    if in_inventory(ESSENCE) != 1:
        raise SystemExit(
            f"CRAFT FAIL: the fruit-grid essence is not in the inventory ({inventory()})"
        )

    # 2. Nine datura seeds -> one essence (the alternative's other side).
    close_table()
    fresh_inventory()
    give(SEEDS, 9)
    wait(3)
    reset_table()
    place(SEEDS, GRID)
    expect_result(ESSENCE, 1, "the 3x3 seed grid")
    take_result("the seed-grid essence")
    expect_consumed({SEEDS: 0}, "the seed grid")
    if in_inventory(ESSENCE) != 1:
        raise SystemExit(
            f"CRAFT FAIL: the seed-grid essence is not in the inventory ({inventory()})"
        )

    # 3. A mixed grid: the alternative is per cell.
    close_table()
    fresh_inventory()
    give(FRUIT, 5)
    give(SEEDS, 4)
    wait(3)
    reset_table()
    place(FRUIT, GRID[:5])
    place(SEEDS, GRID[5:])
    expect_result(ESSENCE, 1, "the mixed 5 fruit + 4 seed grid")
    take_result("the mixed-grid essence")
    expect_consumed({FRUIT: 0, SEEDS: 0}, "the mixed grid")
    if in_inventory(ESSENCE) != 1:
        raise SystemExit(
            f"CRAFT FAIL: the mixed-grid essence is not in the inventory ({inventory()})"
        )

    # 4. The differentials: dirt is not an ingredient, in any cell.
    close_table()
    fresh_inventory()
    give("minecraft:dirt", 9)
    wait(3)
    reset_table()
    place("minecraft:dirt", GRID)
    expect_result(None, 0, "nine dirt")

    close_table()
    fresh_inventory()
    give(FRUIT, 8)
    give("minecraft:dirt", 1)
    wait(3)
    reset_table()
    place(FRUIT, GRID[:8])
    place("minecraft:dirt", GRID[8:])
    expect_result(None, 0, "eight fruit with one dirt in the last cell")

    # 5. The shapeless book: three items, any three cells.
    close_table()
    fresh_inventory()
    give("occultism:awakened_feather", 1)
    give("occultism:purified_ink", 1)
    give("occultism:taboo_book", 1)
    wait(3)
    reset_table()
    place("occultism:awakened_feather", GRID[0:1])
    place("occultism:purified_ink", GRID[1:2])
    place("occultism:taboo_book", GRID[2:3])
    expect_result(BOOK, 1, "the shapeless book")
    take_result("the book")
    expect_consumed(
        {
            "occultism:awakened_feather": 0,
            "occultism:purified_ink": 0,
            "occultism:taboo_book": 0,
        },
        "the book recipe",
    )
    if in_inventory(BOOK) != 1:
        raise SystemExit(f"CRAFT FAIL: the crafted book is not in the inventory ({inventory()})")
    close_table()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
