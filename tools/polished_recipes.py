#!/usr/bin/env python3
"""Craft, smelt and exercise the eight polished-tier `otherstone`/`otherrock` blocks.

Called by `tools/e2e.sh` (phase S) with the open game's control socket and the
crafting table's position. The harness opens a real table as a screen whose 3x3
grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers.

The slice's rule (from the jar's `data/occultism/recipe/crafting/` and
`.../recipe/smelting/`): a recipe whose result and every ingredient the package
already provides. These eight qualify, because their ingredients are
`otherstone`/`otherrock` (carried since 0.2.0) and their results were not
registered until 0.2.8:

* `polished_otherstone`, `polished_otherrock`  **smelting** the raw stone,
                                               200 ticks, 0.5 xp -> 1
* `polished_otherstone_slab`   `ooo` (middle row) of the polished block -> 6
* `polished_otherstone_stairs` `#  `/`## `/`###`                       -> 4
* `polished_otherstone_wall`   `ooo`/`ooo`                             -> 6
* `polished_otherrock_*`       the same three, of the polished otherrock

What is proven, in this order:

1. each of the six grids, filled the way its recipe declares, reads its result in
   the result slot, is taken (the exact count lands in the inventory) and
   consumes every ingredient;
2. the **smelting** pair in a real furnace **first**, because that is how a player
   obtains the polish at all: one raw stone + one coal -> exactly one polished
   block after 200 ticks, with the input gone, plus the **no-fuel** differential
   (the same stone never cooks and stays in the input slot). Smelting first also
   makes the red against a package without the tier name the missing result;
3. then the grids, with the differentials: four **polished** blocks in the 2x2
   make nothing (this tier has no 2x2 recipe; the bricks tier's 2x2 is the raw
   stone) and four dirt in the same cells make nothing, so a grid's result is its
   recipe and not "any 2x2";
4. each of the eight blocks is placed by `setblock`, its default state is read
   back (the template's states, not a plain block's), a bare hand breaks it and
   rolls **nothing** (`requires_tool`, as upstream's `requiresCorrectToolForDrops`),
   a wooden pickaxe breaks it and drops **exactly one** of itself;
5. two polished walls side by side connect (`west=low` on the second), which is
   the `minecraft:walls` tag doing its work; and a double polished slab drops
   two, the jar table's `type=double` branch (`polished_otherrock_slab`'s table is
   the port's fix for the jar's third omission).

`POLISHED_NEGATIVE=1` asserts the polished slab grid makes **nothing**, so a
working package must fail that run: the phase's counterfactual.

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
POLISHED: list[tuple[str, str, list[int], int]] = []
for _stone in ("otherstone", "otherrock"):
    _p = f"polished_{_stone}"
    POLISHED += [
        (f"{_p}_slab", f"occultism:{_p}", SLAB_ROW, 6),
        (f"{_p}_stairs", f"occultism:{_p}", STAIR_CELLS, 4),
        (f"{_p}_wall", f"occultism:{_p}", WALL_CELLS, 6),
    ]

# The eight blocks, in the order the phase places them.
BLOCKS = [
    "polished_otherstone",
    "polished_otherstone_slab",
    "polished_otherstone_stairs",
    "polished_otherstone_wall",
    "polished_otherrock",
    "polished_otherrock_slab",
    "polished_otherrock_stairs",
    "polished_otherrock_wall",
]

# The default state each template declares, as `ctl block` reports it.
DEFAULT_STATE = {
    "plain": "",
    "slab": "[type=bottom,waterlogged=false]",
    "stairs": "[facing=north,half=bottom,shape=straight,waterlogged=false]",
    "wall": "[east=none,north=none,south=none,up=true,waterlogged=false,west=none]",
}

# The brief's own coordinates: the polished row reuses the bricks row (every
# block there has been broken by phase R by the time this runs) and the furnace
# row is the bricks phase's, re-placed fresh so no leftover contents count.
BLOCK_Z = 46
FURNACE_X = 230
FURNACE_Z = 42


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
    negative = os.environ.get("POLISHED_NEGATIVE") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"POLISHED FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"POLISHED FAIL: no menu is open: {screen}")
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
            f"POLISHED FAIL: missing ingredient {item} in the open screen "
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
            raise SystemExit(f"POLISHED FAIL: no inventory slot for {item} ({inventory()})")
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
            raise SystemExit("POLISHED FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"POLISHED FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"POLISHED FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"POLISHED FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("POLISHED FAIL: the polished phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    # ---- the smelting pair, and the no-fuel differential ------------------
    print("== the polished pair in a real furnace, and the same stone with no fuel ==")

    def set_furnace(fx: int, fz: int) -> None:
        # Air first: `/setblock` of the *same* block state keeps the existing
        # block entity, so phase R's furnace at this position would answer with
        # its own leftovers (measured in the first full run, 2026-11-22).
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:air")
        wait(2)
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:furnace")
        wait(4)

    def container(fx: int, fz: int) -> list:
        return run("container", str(fx), str(y), str(fz)).get("slots") or []

    def smelt(stone: str, fx: int, with_fuel: bool) -> list:
        set_furnace(fx, FURNACE_Z)
        run("container", str(fx), str(y), str(FURNACE_Z), f"occultism:{stone}", "1")
        if with_fuel:
            run("container", str(fx), str(y), str(FURNACE_Z), "minecraft:coal", "1")
        wait(240)
        return container(fx, FURNACE_Z)

    for i, stone in enumerate(("otherstone", "otherrock")):
        fx = FURNACE_X + i * 4
        got = smelt(stone, fx, with_fuel=True)
        print(f"  furnace with coal: {json.dumps(got)}")
        output = got[2] if len(got) > 2 else None
        if not output or output.get("item") != f"occultism:polished_{stone}":
            raise SystemExit(
                f"POLISHED FAIL: the furnace made {output} from {stone}, "
                f"expected one occultism:polished_{stone}"
            )
        if output.get("count") != 1:
            raise SystemExit(f"POLISHED FAIL: the furnace made {output.get('count')} polished blocks")
        if got[0] is not None:
            raise SystemExit(f"POLISHED FAIL: the furnace kept its input: {got[0]}")
        print(f"  {stone} + coal -> {output['item']} (the input is gone)")

    # The differential: no fuel, no cooking. The input must still be there.
    fx = FURNACE_X + 8
    got = smelt("otherstone", fx, with_fuel=False)
    print(f"  furnace without fuel: {json.dumps(got)}")
    if len(got) > 2 and got[2] is not None:
        raise SystemExit(f"POLISHED FAIL: a fuel-less furnace cooked {got[2]}")
    if not got or got[0] is None:
        raise SystemExit(f"POLISHED FAIL: the fuel-less furnace lost its input: {got}")
    print("  no fuel: the output stayed empty and the input stayed put")

    # ---- the six grids, one fresh table each ------------------------------
    for name, ingredient, cells, count in POLISHED:
        close_table()
        fresh_inventory()
        run("give", ingredient, str(len(cells)))
        wait(3)
        reset_table()
        place(ingredient, cells)
        if negative and name == "polished_otherstone_slab":
            print("  (negative: asserting the polished slab grid makes nothing)")
            expect_result(None, 0, "negative: the polished slab grid")
            return 1
        expect_result(f"occultism:{name}", count, f"the {name} grid")
        take_result(f"occultism:{name}", f"the {name}")
        expect_consumed({ingredient: 0}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"POLISHED FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the differentials: the slab needs the polished block -------------
    for label, item, cells in (
        # A 2x2 of polished blocks makes nothing in this tier (the bricks tier's
        # 2x2 is the *raw* stone), and four dirt in the same cells make nothing:
        # a grid's result is its recipe, not "any 2x2".
        ("four polished otherstone in the 2x2", "occultism:polished_otherstone",
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
                raise SystemExit(f"POLISHED FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"POLISHED FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

    def drops(reply: dict) -> dict[str, int]:
        return reply.get("drops") or {}

    def drops_total(reply: dict) -> int:
        return sum(drops(reply).values())

    def hand_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"POLISHED FAIL: {block} did not place at {bx},{by},{bz}: '{got}'")
        expected = block + DEFAULT_STATE[shape(block.split(":")[1])]
        if got != expected:
            raise SystemExit(f"POLISHED FAIL: {block} reads '{got}', expected '{expected}'")
        print(f"  placed {got} at x={bx}")
        run("command", "clear @s")
        wait(4)
        reply = break_at(bx, by, bz)
        print(
            f"  {block} by hand: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"POLISHED FAIL: the hand break left {reply.get('block')} at x={bx}")
        if drops_total(reply) != 0:
            raise SystemExit(
                f"POLISHED FAIL: a hand break of {block} dropped {drops(reply)} "
                "(requires_tool must gate the table)"
            )

    def pickaxe_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"POLISHED FAIL: {block} did not place again: '{got}'")
        run("command", "item replace entity @s hotbar.0 with minecraft:wooden_pickaxe 1")
        wait(4)
        run("select", "1")
        wait(2)
        reply = break_at(bx, by, bz)
        print(
            f"  {block} with a wooden pickaxe: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"POLISHED FAIL: the pickaxe break left {reply.get('block')} at x={bx}")
        got_drops = drops(reply)
        if got_drops.get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"POLISHED FAIL: a wooden pickaxe dropped {got_drops} for {block}, "
                "expected exactly one of it"
            )

    print("== the eight blocks: default state, hand 0, wooden pickaxe 1 ==")
    for i, name in enumerate(BLOCKS):
        block = f"occultism:{name}"
        bx = 214 + i * 2
        hand_break(block, bx, y, BLOCK_Z)
        pickaxe_break(block, bx, y, BLOCK_Z)

    # ---- two polished walls side by side connect (`minecraft:walls`) ------
    wall_a, wall_b = 214, 215
    wall_z = BLOCK_Z + 4
    set_block(wall_a, y, wall_z, "occultism:polished_otherstone_wall")
    set_block(wall_b, y, wall_z, "occultism:polished_otherstone_wall")
    state_a = state_at(wall_a, y, wall_z)["state"]
    state_b = state_at(wall_b, y, wall_z)["state"]
    print(f"  wall pair: {state_a} / {state_b}")
    if "west=low" not in state_b:
        raise SystemExit(
            f"POLISHED FAIL: the second polished wall does not connect to the first "
            f"('{state_b}'): the wall is not in minecraft:walls"
        )
    set_block(wall_a, y, wall_z, "minecraft:air")
    set_block(wall_b, y, wall_z, "minecraft:air")
    wait(4)

    # ---- the double polished slab: the jar table's `type=double` branch ---
    double_x = 224
    set_block(double_x, y, wall_z, "occultism:polished_otherstone_slab[type=double]")
    got = state_at(double_x, y, wall_z)["state"]
    if got != "occultism:polished_otherstone_slab[type=double,waterlogged=false]":
        raise SystemExit(f"POLISHED FAIL: the double polished slab reads '{got}'")
    run("command", "item replace entity @s hotbar.0 with minecraft:wooden_pickaxe 1")
    wait(4)
    run("select", "1")
    wait(2)
    reply = break_at(double_x, y, wall_z)
    measured = drops(reply).get("occultism:polished_otherstone_slab", 0)
    print(f"  double polished slab: the pickaxe break dropped {measured} (the jar table's double branch)")
    if reply.get("block") != "minecraft:air":
        raise SystemExit(f"POLISHED FAIL: the double polished slab did not break: {reply.get('block')}")
    if measured != 2 or drops_total(reply) != 2:
        raise SystemExit(
            f"POLISHED FAIL: the double polished slab dropped {drops(reply)}, expected exactly 2 "
            "(the table's match_block type=double branch)"
        )

    # ---- the otherrock slab, whose table the jar does not ship -----------
    # `polished_otherrock_slab.json` is the third absent table; without it the
    # block would drop nothing at all. Its own pickaxe break is the proof.
    rock_x = 226
    set_block(rock_x, y, wall_z, "occultism:polished_otherrock_slab")
    run("command", "clear @s")
    wait(4)
    run("command", "item replace entity @s hotbar.0 with minecraft:wooden_pickaxe 1")
    wait(4)
    run("select", "1")
    wait(2)
    reply = break_at(rock_x, y, wall_z)
    got_drops = drops(reply)
    print(f"  polished_otherrock_slab (the jar's missing table): {json.dumps(got_drops)}")
    if got_drops.get("occultism:polished_otherrock_slab") != 1 or drops_total(reply) != 1:
        raise SystemExit(
            f"POLISHED FAIL: polished_otherrock_slab dropped {got_drops}, expected exactly one "
            "(the jar ships no table for it; the port does)"
        )

    run("command", "clear @s")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
