#!/usr/bin/env python3
"""Craft, place and break the eight `othercobblestone`/`othercobblerock` blocks.

Called by `tools/e2e.sh` (phase T) with the open game's control socket and the
crafting table's position. The harness opens a real table as a screen whose 3x3
grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers.

The slice's rule (from the jar's `data/occultism/recipe/crafting/`,
`.../loot_table/blocks/` and `OccultismBlocks.java`): the **cobblestone tier** is
the pair `othercobblestone`/`othercobblerock` plus each one's slab, stairs and
wall. There is no grid for the two plain blocks — upstream has none: a player
gets them by breaking `otherstone`/`otherrock` **without silk touch** (the raw
stones' own table, which 0.2.4 shipped simplified and this slice carries from the
jar), and smelts them back into the stone.

What is proven, in this order:

1. each of the six grids (slab/stair/wall of each cobble block), filled the way
   its recipe declares, reads its result in the result slot, is taken (the exact
   count lands in the inventory) and consumes every ingredient;
2. the differentials: four cobble blocks in the 2x2 make nothing (this tier has
   no 2x2 recipe) and four dirt in the same cells make nothing, so a grid's
   result is its recipe and not "any 2x2";
3. the **obtainability** of the two plain blocks: `otherstone`/`otherrock` broken
   with a wooden pickaxe drop exactly one cobble block and none of themselves,
   and the same block broken with a **silk-touch** pickaxe drops exactly one of
   itself and no cobble — the pair that fails if the 0.2.4 table (always itself)
   or a verbatim 1.21.1 condition shape (nothing at all) is carried;
4. each of the eight blocks is placed by `setblock`, its default state is read
   back (the template's states, not a plain block's), a bare hand breaks it and
   rolls **nothing** (`requires_tool`, as upstream's `requiresCorrectToolForDrops`),
   a wooden pickaxe breaks it and drops **exactly one** of itself;
5. two cobble walls side by side connect (`west=low` on the second), which is the
   `minecraft:walls` tag doing its work; a double cobble slab drops two, the jar
   table's `type=double` branch; and `othercobblerock_slab` — the table the jar
   does **not** ship — drops one.

`COBBLE_NEGATIVE=1` asserts the cobble slab grid makes **nothing**, so a working
package must fail that run: the phase's counterfactual.

Exit 1 with the observed state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200
SLAB_ROW = GRID[3:6]  # the middle row
STAIR_CELLS = [GRID[0], GRID[3], GRID[4], GRID[6], GRID[7], GRID[8]]
WALL_CELLS = GRID[0:6]  # two full rows

# name, ingredient, grid cells, result count
COBBLE: list[tuple[str, str, list[int], int]] = []
for _stone in ("othercobblestone", "othercobblerock"):
    COBBLE += [
        (f"{_stone}_slab", f"occultism:{_stone}", SLAB_ROW, 6),
        (f"{_stone}_stairs", f"occultism:{_stone}", STAIR_CELLS, 4),
        (f"{_stone}_wall", f"occultism:{_stone}", WALL_CELLS, 6),
    ]

# The eight blocks, in the order the phase places them. The slab is first, at
# x=214, so the brief's own `setblock 214 101 46 occultism:othercobblestone_slab`
# is the first assertion of the row.
BLOCKS = [
    "othercobblestone_slab",
    "othercobblestone_stairs",
    "othercobblestone_wall",
    "othercobblestone",
    "othercobblerock_slab",
    "othercobblerock_stairs",
    "othercobblerock_wall",
    "othercobblerock",
]

# The default state each template declares, as `ctl block` reports it.
DEFAULT_STATE = {
    "plain": "",
    "slab": "[type=bottom,waterlogged=false]",
    "stairs": "[facing=north,half=bottom,shape=straight,waterlogged=false]",
    "wall": "[east=none,north=none,south=none,up=true,waterlogged=false,west=none]",
}

# The row the eight blocks go on (the polished row: every block there has been
# broken by phase S by the time this runs), the wall pair / special legs' row
# just north of it, and the raw stones' row beyond that. `break_at` stands two
# blocks south, so nothing is placed past z=50 (the arena floor ends at z=52).
BLOCK_Z = 46
SPECIAL_Z = 48
RAW_Z = 50
TABLE_X = 222
TABLE_Z = 38


def shape(name: str) -> str:
    if name.endswith("_slab"):
        return "slab"
    if name.endswith("_stairs"):
        return "stairs"
    if name.endswith("_wall"):
        return "wall"
    return "plain"


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    negative = os.environ.get("COBBLE_NEGATIVE") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"COBBLE FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"COBBLE FAIL: no menu is open: {screen}")
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
            f"COBBLE FAIL: missing ingredient {item} in the open screen "
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
            raise SystemExit(f"COBBLE FAIL: no inventory slot for {item} ({inventory()})")
        return free

    def place(item: str, cells: list[int]) -> None:
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
            raise SystemExit("COBBLE FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"COBBLE FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"COBBLE FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"COBBLE FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("COBBLE FAIL: the cobble phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    # ---- the six grids, one fresh table each ------------------------------
    for name, ingredient, cells, count in COBBLE:
        close_table()
        fresh_inventory()
        run("give", ingredient, str(len(cells)))
        wait(3)
        reset_table()
        place(ingredient, cells)
        if negative and name == "othercobblestone_slab":
            print("  (negative: asserting the cobble slab grid makes nothing)")
            expect_result(None, 0, "negative: the cobble slab grid")
            return 1
        expect_result(f"occultism:{name}", count, f"the {name} grid")
        take_result(f"occultism:{name}", f"the {name}")
        expect_consumed({ingredient: 0}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"COBBLE FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the differentials: a 2x2 of the cobble makes nothing ------------
    for label, item, cells in (
        # This tier has no 2x2 recipe (the bricks tier's 2x2 is the smooth
        # stone), and four dirt in the same cells make nothing: a grid's result
        # is its recipe, not "any 2x2".
        ("four othercobblestone in the 2x2", "occultism:othercobblestone",
         [GRID[0], GRID[1], GRID[3], GRID[4]]),
        ("four dirt in the 2x2", "minecraft:dirt", [GRID[0], GRID[1], GRID[3], GRID[4]]),
    ):
        close_table()
        fresh_inventory()
        run("give", item, str(len(cells)))
        wait(3)
        reset_table()
        place(item, cells)
        expect_result(None, 0, label)
    close_table()

    # ---- the eight blocks: place, read back, hand 0, pickaxe 1 ------------
    def state_at(bx: int, by: int, bz: int) -> dict:
        return run("block", str(bx), str(by), str(bz))

    def set_block(bx: int, by: int, bz: int, state: str) -> None:
        run("command", f"execute in minecraft:the_end run setblock {bx} {by} {bz} {state}")
        wait(4)

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
                raise SystemExit(f"COBBLE FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"COBBLE FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

    def drops(reply: dict) -> dict[str, int]:
        return reply.get("drops") or {}

    def drops_total(reply: dict) -> int:
        return sum(drops(reply).values())

    def hold(item: str) -> None:
        run("command", f"item replace entity @s hotbar.0 with {item} 1")
        wait(4)
        run("select", "1")
        wait(2)

    def silk_hold() -> None:
        run("command", "clear @s")
        run("command", "xp add @s 30 levels")
        hold("minecraft:diamond_pickaxe")
        run("command", "enchant @s minecraft:silk_touch")
        wait(4)

    def hand_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"COBBLE FAIL: {block} did not place at {bx},{by},{bz}: '{got}'")
        expected = block + DEFAULT_STATE[shape(block.split(":")[1])]
        if got != expected:
            raise SystemExit(f"COBBLE FAIL: {block} reads '{got}', expected '{expected}'")
        print(f"  placed {got} at x={bx}")
        run("command", "clear @s")
        wait(4)
        reply = break_at(bx, by, bz)
        print(
            f"  {block} by hand: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"COBBLE FAIL: the hand break left {reply.get('block')} at x={bx}")
        if drops_total(reply) != 0:
            raise SystemExit(
                f"COBBLE FAIL: a hand break of {block} dropped {drops(reply)} "
                "(requires_tool must gate the table)"
            )

    def pickaxe_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"COBBLE FAIL: {block} did not place again: '{got}'")
        hold("minecraft:wooden_pickaxe")
        reply = break_at(bx, by, bz)
        print(
            f"  {block} with a wooden pickaxe: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"COBBLE FAIL: the pickaxe break left {reply.get('block')} at x={bx}")
        got_drops = drops(reply)
        if got_drops.get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"COBBLE FAIL: a wooden pickaxe dropped {got_drops} for {block}, "
                "expected exactly one of it"
            )

    print("== the eight blocks: default state, hand 0, wooden pickaxe 1 ==")
    for i, name in enumerate(BLOCKS):
        block = f"occultism:{name}"
        bx = 214 + i * 2
        hand_break(block, bx, y, BLOCK_Z)
        pickaxe_break(block, bx, y, BLOCK_Z)

    # ---- two cobble walls side by side connect (`minecraft:walls`) --------
    wall_a, wall_b = 214, 215
    set_block(wall_a, y, SPECIAL_Z, "occultism:othercobblestone_wall")
    set_block(wall_b, y, SPECIAL_Z, "occultism:othercobblestone_wall")
    state_a = state_at(wall_a, y, SPECIAL_Z)["state"]
    state_b = state_at(wall_b, y, SPECIAL_Z)["state"]
    print(f"  wall pair: {state_a} / {state_b}")
    if "west=low" not in state_b:
        raise SystemExit(
            f"COBBLE FAIL: the second cobble wall does not connect to the first "
            f"('{state_b}'): the wall is not in minecraft:walls"
        )
    set_block(wall_a, y, SPECIAL_Z, "minecraft:air")
    set_block(wall_b, y, SPECIAL_Z, "minecraft:air")
    wait(4)

    # ---- the double cobble slab: the jar table's `type=double` branch -----
    double_x = 218
    set_block(double_x, y, SPECIAL_Z, "occultism:othercobblestone_slab[type=double]")
    got = state_at(double_x, y, SPECIAL_Z)["state"]
    if got != "occultism:othercobblestone_slab[type=double,waterlogged=false]":
        raise SystemExit(f"COBBLE FAIL: the double cobble slab reads '{got}'")
    hold("minecraft:wooden_pickaxe")
    reply = break_at(double_x, y, SPECIAL_Z)
    measured = drops(reply).get("occultism:othercobblestone_slab", 0)
    print(f"  double cobble slab: the pickaxe break dropped {measured} (the jar table's double branch)")
    if reply.get("block") != "minecraft:air":
        raise SystemExit(f"COBBLE FAIL: the double cobble slab did not break: {reply.get('block')}")
    if measured != 2 or drops_total(reply) != 2:
        raise SystemExit(
            f"COBBLE FAIL: the double cobble slab dropped {drops(reply)}, expected exactly 2 "
            "(the table's match_block type=double branch)"
        )

    # ---- the otherrock slab, whose table the jar does not ship -----------
    # `othercobblerock_slab.json` is the fourth absent table; without it the
    # block would drop nothing at all. Its own pickaxe break is the proof.
    rock_x = 220
    set_block(rock_x, y, SPECIAL_Z, "occultism:othercobblerock_slab")
    run("command", "clear @s")
    wait(4)
    hold("minecraft:wooden_pickaxe")
    reply = break_at(rock_x, y, SPECIAL_Z)
    got_drops = drops(reply)
    print(f"  othercobblerock_slab (the jar's missing table): {json.dumps(got_drops)}")
    if got_drops.get("occultism:othercobblerock_slab") != 1 or drops_total(reply) != 1:
        raise SystemExit(
            f"COBBLE FAIL: othercobblerock_slab dropped {got_drops}, expected exactly one "
            "(the jar ships no table for it; the port does)"
        )

    # ---- the plain pair's obtainability: the raw stones' own tables ------
    print("== the plain pair: the raw stones drop cobble without silk touch, themselves with it ==")
    for i, (stone, cobble_name) in enumerate(
        (("otherstone", "othercobblestone"), ("otherrock", "othercobblerock"))
    ):
        # A break reply's `.drops` is keyed by the full id.
        cobble = f"occultism:{cobble_name}"
        bx = 230 + i * 2
        set_block(bx, y, RAW_Z, f"occultism:{stone}")
        got = state_at(bx, y, RAW_Z)["state"]
        if got != f"occultism:{stone}":
            raise SystemExit(f"COBBLE FAIL: {stone} did not place at {bx},{y},{RAW_Z}: '{got}'")
        hold("minecraft:wooden_pickaxe")
        reply = break_at(bx, y, RAW_Z)
        print(
            f"  {stone} with a wooden pickaxe: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"COBBLE FAIL: the wooden pickaxe left {reply.get('block')}")
        if drops(reply).get(cobble) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"COBBLE FAIL: {stone} without silk touch dropped {drops(reply)}, "
                f"expected exactly one {cobble} (the jar's alternatives table)"
            )
        if drops(reply).get(f"occultism:{stone}"):
            raise SystemExit(
                f"COBBLE FAIL: {stone} dropped itself without silk touch: {drops(reply)}"
            )
        set_block(bx, y, RAW_Z, f"occultism:{stone}")
        silk_hold()
        reply = break_at(bx, y, RAW_Z)
        print(
            f"  {stone} with a silk-touch pickaxe: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"COBBLE FAIL: the silk-touch break left {reply.get('block')}")
        if drops(reply).get(f"occultism:{stone}") != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"COBBLE FAIL: {stone} under silk touch dropped {drops(reply)}, "
                "expected exactly the stone itself"
            )
        if drops(reply).get(cobble):
            raise SystemExit(
                f"COBBLE FAIL: {stone} under silk touch also dropped {cobble}: {drops(reply)}"
            )

    run("command", "clear @s")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
