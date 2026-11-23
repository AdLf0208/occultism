#!/usr/bin/env python3
"""Craft, place and break the frames and pedestals of the building family.

Called by `tools/e2e.sh` (phase V) with the open game's control socket and the
crafting table's position. The harness opens a real table as a screen whose 3x3
grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers.

The slice's rule (from the jar's `data/occultism/recipe/crafting/`,
`.../loot_table/blocks/`, `OccultismItems.java:407-410` and
`OccultismBlocks.java:535-541` at `release/v1.21.1-1.225.0`):

* **`otherstone_frame` and `otherrock_frame` are items**, not blocks. The jar has
  an item model, an item name and a grid for each, and **no** blockstate, **no**
  block model and **no** loot table. The brief called the frame "a wall-mounted
  item frame with its own block entity"; the artefact says otherwise, and the
  artefact wins. What is proven here is the item: it crafts, it is given, and a
  `setblock` of its id leaves air (it is not a block);
* **`otherstone_pedestal` and `otherstone_pedestal_silver`** are plain
  `NonPathfindableBlock`s (no block entity, no stored stack, no use): a full copy
  of `OTHERSTONE` with `noOcclusion()`, a four-box post shape and
  `isPathfindable → false`. The otherrock pedestal keeps the jar's old id (the
  `TODO: change id in next major version` beside `OTHERROCK_PEDESTAL`), so its
  recipe **file** is `otherrock_pedestal.json` and its **result** is
  `occultism:otherstone_pedestal_silver`.

What is proven, in this order:

1. each of the four grids, filled the way its recipe declares, reads its result
   in the result slot, is taken (the exact count lands in the inventory) and
   consumes every ingredient: eight stones in a ring (the centre empty) make one
   frame, five slabs + one stone make one pedestal;
2. the differentials: nine stones in the ring **and** the centre make nothing
   (the frame's pattern has a hole), and four slabs + one stone make nothing (the
   pedestal needs five);
3. the two frames are **items**: `give` puts one in the inventory and a
   `setblock` of the id leaves air. `PEDESTAL_SELFTEST=1` inverts that leg (it
   demands the frame place a block), so a working package must fail that run;
4. each pedestal is placed by `setblock`, its state is read back, a bare hand
   drops **nothing** and a wooden pickaxe drops exactly one — the template is
   `minecraft:stone` with `requiresCorrectToolForDrops` (the pin's
   `gameplay-facts.json` reads `requires_tool: true` for `minecraft:stone`), and
   a bare-hand break of `occultism:otherstone` also drops nothing, the
   differential that keeps the hand leg fail-capable.

`PEDESTAL_NEGATIVE=1` asserts the frame grid makes nothing, so a working package
must fail that run too (the counterfactual for the crafting leg).

Exit 1 with the observed state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200
# The frame's ring: every cell but the centre.
RING = [GRID[i] for i in (0, 1, 2, 3, 5, 6, 7, 8)]
# The pedestal's pattern `s s` / ` o ` / `sss`: slabs on the top corners and the
# whole bottom row, one stone in the centre.
PEDESTAL_SLABS = [GRID[i] for i in (0, 2, 6, 7, 8)]
PEDESTAL_STONE = [GRID[4]]

# name (result id), recipe file, {ingredient: cells}, result count
GRIDS = [
    ("otherstone_frame", "otherstone_frame", {"occultism:otherstone": RING}, 1),
    ("otherrock_frame", "otherrock_frame", {"occultism:otherrock": RING}, 1),
    (
        "otherstone_pedestal",
        "otherstone_pedestal",
        {
            "occultism:otherstone_slab": PEDESTAL_SLABS,
            "occultism:otherstone": PEDESTAL_STONE,
        },
        1,
    ),
    # The jar's file/result split: `otherrock_pedestal.json` makes the
    # `otherstone_pedestal_silver` block.
    (
        "otherstone_pedestal_silver",
        "otherrock_pedestal",
        {
            "occultism:otherrock_slab": PEDESTAL_SLABS,
            "occultism:otherrock": PEDESTAL_STONE,
        },
        1,
    ),
]

# The two pedestals, in the order the phase places them.
PEDESTALS = ["otherstone_pedestal", "otherstone_pedestal_silver"]

# The row the pedestals go on (`z = 50`: the last row before the arena's edge, so
# the breaker stands on the floor at `z = 52`). The differential sits beside them.
BLOCK_Z = 50
TABLE_X = 222
TABLE_Z = 38


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    negative = os.environ.get("PEDESTAL_NEGATIVE") == "1"
    selftest = os.environ.get("PEDESTAL_SELFTEST") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"PEDESTAL FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"PEDESTAL FAIL: no menu is open: {screen}")
        return screen["slots"]

    def slot_of(number: int) -> dict:
        return next((s for s in slots() if s["slot"] == number), {})

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
            f"PEDESTAL FAIL: missing ingredient {item} in the open screen "
            f"(inventory: {inventory()})"
        )

    def empty_or_same(item: str) -> int:
        """A slot the crafted stack can be put into: a same-item stack with room
        first (so two crafts merge, as a player's would), then an empty one."""
        free = None
        for slot in slots():
            number = int(slot["slot"])
            if number > 35:
                continue
            got = slot.get("item")
            if got == item and int(slot.get("count") or 0) < 64:
                return number
            if got is None and free is None:
                free = number
        if free is None:
            raise SystemExit(f"PEDESTAL FAIL: no inventory slot for {item} ({inventory()})")
        return free

    def place(cells_by_item: dict[str, list[int]]) -> None:
        """Fill the grid one item per cell, one ingredient stack at a time."""
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
        if not slots():
            raise SystemExit("PEDESTAL FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"PEDESTAL FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"PEDESTAL FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
            )
        print(f"  {what}: the result slot reads {got} x{got_count}")

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
                    f"PEDESTAL FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("PEDESTAL FAIL: the frames-and-pedestals phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    def set_block(bx: int, by: int, bz: int, state: str) -> None:
        run("command", f"execute in minecraft:the_end run setblock {bx} {by} {bz} {state}")
        wait(4)

    def state_at(bx: int, by: int, bz: int) -> dict:
        return run("block", str(bx), str(by), str(bz))

    # ---- the four grids, one fresh table each -----------------------------
    print("== the four frame and pedestal grids ==")
    for name, _file, cells_by_item, count in GRIDS:
        close_table()
        fresh_inventory()
        for item, cells in cells_by_item.items():
            run("give", item, str(len(cells)))
            wait(2)
        reset_table()
        place(cells_by_item)
        what = f"the {_file} grid"
        if negative and name == "otherstone_frame":
            print("  (negative: asserting the otherstone frame grid makes nothing)")
            expect_result(None, 0, f"negative: {what}")
            continue
        expect_result(f"occultism:{name}", count, f"{what} -> {name}")
        take_result(f"occultism:{name}", f"the {name}")
        expect_consumed({item: 0 for item in cells_by_item}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"PEDESTAL FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the wrong shapes: nothing ----------------------------------------
    print("== the wrong grids: nothing ==")
    for label, cells_by_item in (
        # The frame's ring has a hole: a ninth stone in the centre is no frame.
        ("nine otherstone in the ring and the centre", {"occultism:otherstone": GRID}),
        # The pedestal needs five slabs: four and the stone are no pedestal.
        (
            "four otherstone_slab and one otherstone",
            {
                "occultism:otherstone_slab": [GRID[i] for i in (0, 2, 6, 7)],
                "occultism:otherstone": PEDESTAL_STONE,
            },
        ),
    ):
        close_table()
        fresh_inventory()
        for item, cells in cells_by_item.items():
            run("give", item, str(len(cells)))
            wait(2)
        reset_table()
        place(cells_by_item)
        expect_result(None, 0, label)
    close_table()

    # ---- the two frames are items, not blocks -----------------------------
    print("== the two frames: items, not blocks ==")
    run("command", "clear @s")
    run("command", "kill @e[type=minecraft:item]")
    wait(3)
    for frame in ("otherstone_frame", "otherrock_frame"):
        item = f"occultism:{frame}"
        run("give", item, "1")
        wait(3)
        if in_inventory(item) != 1:
            raise SystemExit(
                f"PEDESTAL FAIL: the {item} item did not register ({inventory()})"
            )
        print(f"  {item}: one in the inventory")
        set_block(214, y, BLOCK_Z, "minecraft:air")
        # The id is an item id: `/setblock` must refuse it and leave air.
        rc, out, err = raw(
            "command",
            f"execute in minecraft:the_end run setblock 214 {y} {BLOCK_Z} {item}",
        )
        wait(4)
        got = state_at(214, y, BLOCK_Z).get("state")
        if selftest:
            print("  (frame selftest: asserting the frame id places a block)")
            if got == "minecraft:air":
                raise SystemExit(
                    f"PEDESTAL FAIL: frame selftest: {item} left air at 214,{y},{BLOCK_Z} "
                    "(this run must fail; the frame is an item, not a block)"
                )
            print(f"  frame selftest: {item} placed {got} (a working package must not)")
            continue
        if got != "minecraft:air":
            raise SystemExit(
                f"PEDESTAL FAIL: /setblock {item} left {got}: the frame is an item, "
                f"not a block (reply {out.strip() or err.strip()})"
            )
        print(f"  /setblock {item}: the block stayed air (it is an item)")

    # ---- the two pedestals: state, hand 0, wooden pickaxe 1 ---------------
    def break_at(bx: int, by: int, bz: int) -> dict:
        run("command", f"execute in minecraft:the_end run tp @s {bx}.5 101 {bz + 2}.5")
        wait(4)
        run("look-at-block", str(bx), str(by), str(bz), "up")
        wait(2)
        rc, out, err = raw("break", str(bx), str(by), str(bz))
        if rc != 0:
            print(f"    break at {bx},{by},{bz} failed ({out or err}); re-aiming", file=sys.stderr)
            run("look-at-block", str(bx), str(by), str(bz), "up")
            wait(3)
            rc, out, err = raw("break", str(bx), str(by), str(bz))
            if rc != 0:
                raise SystemExit(f"PEDESTAL FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"PEDESTAL FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

    def drops(reply: dict) -> dict[str, int]:
        return reply.get("drops") or {}

    def drops_total(reply: dict) -> int:
        return sum(drops(reply).values())

    def hold(item: str) -> None:
        run("command", f"item replace entity @s hotbar.0 with {item} 1")
        wait(4)
        run("select", "1")
        wait(2)

    def lay(block: str, bx: int, bz: int) -> str:
        # A break's reply counts the drops at the block's position, so a leftover
        # entity from the previous leg merges into the next reply (measured on
        # phase U's second run, 2026-11-23). Clear the world's items first.
        run("command", "kill @e[type=minecraft:item]")
        wait(2)
        set_block(bx, y, bz, block)
        got = state_at(bx, y, bz)["state"]
        if got != block:
            raise SystemExit(f"PEDESTAL FAIL: {block} reads '{got}', expected '{block}'")
        return got

    def hand_break(block: str, bx: int, bz: int) -> None:
        got = lay(block, bx, bz)
        print(f"  placed {got} at x={bx}")
        run("command", "clear @s")
        wait(4)
        reply = break_at(bx, y, bz)
        print(
            f"  {block} by hand: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"PEDESTAL FAIL: the hand break left {reply.get('block')} at x={bx}")
        if drops_total(reply) != 0:
            raise SystemExit(
                f"PEDESTAL FAIL: a hand break of {block} dropped {drops(reply)}, "
                "expected nothing (the template is requires_tool: true)"
            )

    def pickaxe_break(block: str, bx: int, bz: int) -> None:
        lay(block, bx, bz)
        hold("minecraft:wooden_pickaxe")
        reply = break_at(bx, y, bz)
        print(
            f"  {block} with a wooden pickaxe: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"PEDESTAL FAIL: the pickaxe break left {reply.get('block')} at x={bx}")
        if drops(reply).get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"PEDESTAL FAIL: a wooden pickaxe dropped {drops(reply)} for {block}, "
                "expected exactly one of it"
            )

    print("== the two pedestals: state, hand 0, wooden pickaxe 1 ==")
    for i, name in enumerate(PEDESTALS):
        block = f"occultism:{name}"
        bx = 214 + i * 2
        hand_break(block, bx, BLOCK_Z)
        pickaxe_break(block, bx, BLOCK_Z)

    # ---- the differential that makes the hand leg fail-capable -----------
    # `otherstone` is the same `requires_tool: true` stone template: a bare hand
    # breaks it and rolls NOTHING. If the hand breaks above dropped nothing only
    # because "hand breaks roll nothing", this assertion could not differ.
    diff_x = 222
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    set_block(diff_x, y, BLOCK_Z, "occultism:otherstone")
    got = state_at(diff_x, y, BLOCK_Z)["state"]
    if got != "occultism:otherstone":
        raise SystemExit(f"PEDESTAL FAIL: otherstone did not place: '{got}'")
    run("command", "clear @s")
    wait(4)
    reply = break_at(diff_x, y, BLOCK_Z)
    print(f"  differential, otherstone by hand: {json.dumps(drops(reply))}")
    if drops_total(reply) != 0:
        raise SystemExit(
            f"PEDESTAL FAIL: a hand break of otherstone dropped {drops(reply)}; "
            "the differential that makes this family's hand-0 result meaningful is broken"
        )

    run("command", "clear @s")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
