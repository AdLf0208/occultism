#!/usr/bin/env python3
"""Craft and exercise the twelve bricks-tier `otherstone`/`otherrock` blocks.

Called by `tools/e2e.sh` (phase R) with the open game's control socket and the
crafting table's position. The harness opens a real table as a screen whose 3x3
grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers.

The slice's rule (from the jar's `data/occultism/recipe/crafting/` and
`.../recipe/smelting/`): a recipe whose result and every ingredient the package
already provides. These twelve qualify, because their ingredients are
`otherstone`/`otherrock` (carried since 0.2.0) and their results were not
registered until 0.2.7:

* `otherstone_bricks`      `oo`/`oo`                    -> 4
* `otherstone_bricks_slab`  `ooo` (middle row)           -> 6
* `otherstone_bricks_stairs` `#  `/`## `/`###`          -> 4
* `otherstone_bricks_wall` `ooo`/`ooo`                  -> 6
* `otherrock_bricks*`       the same four,
* `chiseled_*`              two of the brick **slab**, stacked (upstream's own
                            recipe - not a bricks 2x2)  -> 1
* `cracked_*`               **smelting** the bricks, 200 ticks, 0.3 xp -> 1

What is proven, in this order:

1. each of the ten grids, filled the way its recipe declares, reads its result in
   the result slot, is taken (the exact count lands in the inventory) and
   consumes every ingredient;
2. a **chain in one opened table**: twelve carried stones -> three brick crafts
   (4 each) -> six of those bricks become four brick stairs -> three of the
   remaining bricks become six brick slabs -> two of those slabs become one
   chiseled block. Every step consumes the previous step's output, which is what
   makes these recipes a family rather than twelve independent grids;
3. the **smelting** pair in a real furnace: bricks + coal -> cracked bricks after
   200 ticks, and a furnace with the same bricks but **no fuel** never cooks (the
   differential);
4. differentials: four dirt in the same 2x2 make nothing, and three stones (one
   cell short) make nothing, so a grid's result is its recipe and not "any 2x2";
5. each of the twelve blocks is placed by `setblock`, its default state is read
   back (the template's states, not a plain block's), a bare hand breaks it and
   rolls **nothing** (`requires_tool`, as upstream's `requiresCorrectToolForDrops`),
   a wooden pickaxe breaks it and drops **exactly one** of itself;
6. two brick walls side by side connect (`west=low` on the second), which is the
   `minecraft:walls` tag doing its work; and a double brick slab drops two, the
   jar table's `type=double` branch.

`BRICKS_NEGATIVE=1` asserts the 2x2 bricks grid makes **nothing**, so a working
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
BRICKS_2X2 = [GRID[0], GRID[1], GRID[3], GRID[4]]  # top-left 2x2
SLAB_ROW = GRID[3:6]  # the middle row
STAIR_CELLS = [GRID[0], GRID[3], GRID[4], GRID[6], GRID[7], GRID[8]]
WALL_CELLS = GRID[0:6]  # two full rows
CHISELED = [GRID[0], GRID[3]]  # one column, two cells

# name, ingredient, grid cells, result count
BRICKS: list[tuple[str, str, list[int], int]] = []
for _stone in ("otherstone", "otherrock"):
    _b = f"{_stone}_bricks"
    BRICKS += [
        (_b, f"occultism:{_stone}", BRICKS_2X2, 4),
        (f"{_b}_slab", f"occultism:{_b}", SLAB_ROW, 6),
        (f"{_b}_stairs", f"occultism:{_b}", STAIR_CELLS, 4),
        (f"{_b}_wall", f"occultism:{_b}", WALL_CELLS, 6),
        (f"chiseled_{_b}", f"occultism:{_b}_slab", CHISELED, 1),
    ]

# The twelve blocks, in the order the phase places them.
BLOCKS = [
    "otherstone_bricks",
    "otherstone_bricks_slab",
    "otherstone_bricks_stairs",
    "otherstone_bricks_wall",
    "chiseled_otherstone_bricks",
    "cracked_otherstone_bricks",
    "otherrock_bricks",
    "otherrock_bricks_slab",
    "otherrock_bricks_stairs",
    "otherrock_bricks_wall",
    "chiseled_otherrock_bricks",
    "cracked_otherrock_bricks",
]

# The default state each template declares, as `ctl block` reports it.
DEFAULT_STATE = {
    "plain": "",
    "slab": "[type=bottom,waterlogged=false]",
    "stairs": "[facing=north,half=bottom,shape=straight,waterlogged=false]",
    "wall": "[east=none,north=none,south=none,up=true,waterlogged=false,west=none]",
}

BLOCK_Z = 46  # the bricks row, north of the six building blocks (z=33)
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
    negative = os.environ.get("BRICKS_NEGATIVE") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"BRICKS FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"BRICKS FAIL: no menu is open: {screen}")
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
            f"BRICKS FAIL: missing ingredient {item} in the open screen "
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
            raise SystemExit(f"BRICKS FAIL: no inventory slot for {item} ({inventory()})")
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
            raise SystemExit("BRICKS FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"BRICKS FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"BRICKS FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"BRICKS FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    # ---- the ten grids, one fresh table each ------------------------------
    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("BRICKS FAIL: the bricks phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    for name, ingredient, cells, count in BRICKS:
        close_table()
        fresh_inventory()
        run("give", ingredient, str(len(cells)))
        wait(3)
        reset_table()
        place(ingredient, cells)
        if negative and name == "otherstone_bricks":
            print("  (negative: asserting the 2x2 of otherstone makes nothing)")
            expect_result(None, 0, "negative: the bricks grid")
            return 1
        expect_result(f"occultism:{name}", count, f"the {name} grid")
        take_result(f"occultism:{name}", f"the {name}")
        expect_consumed({ingredient: 0}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"BRICKS FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the differentials: not every 2x2 is a brick grid -----------------
    for label, item, cells in (
        ("four dirt in the 2x2", "minecraft:dirt", BRICKS_2X2),
        ("three otherstone, one cell short of the 2x2", "occultism:otherstone", BRICKS_2X2[:3]),
    ):
        close_table()
        fresh_inventory()
        run("give", item, str(len(cells)))
        wait(3)
        reset_table()
        place(item, cells)
        expect_result(None, 0, label)
    close_table()

    # ---- the chain: stones -> bricks -> stairs -> slabs -> chiseled --------
    print("== the chain in one opened table: 12 otherstone -> bricks -> stairs/slabs -> chiseled ==")
    close_table()
    fresh_inventory()
    run("give", "occultism:otherstone", "12")
    wait(3)
    reset_table()
    for step in range(3):
        place("occultism:otherstone", BRICKS_2X2)
        expect_result("occultism:otherstone_bricks", 4, f"chain step 1.{step}: the 2x2 of stones")
        take_result("occultism:otherstone_bricks", f"chain step 1.{step}: four bricks")
    expect_consumed({"occultism:otherstone": 0}, "chain step 1: the twelve stones")
    if in_inventory("occultism:otherstone_bricks") != 12:
        raise SystemExit(
            f"BRICKS FAIL: three brick crafts left "
            f"{in_inventory('occultism:otherstone_bricks')} bricks, expected 12 "
            f"({inventory()})"
        )

    place("occultism:otherstone_bricks", STAIR_CELLS)
    expect_result("occultism:otherstone_bricks_stairs", 4, "chain step 2: six bricks")
    take_result("occultism:otherstone_bricks_stairs", "chain step 2: four stairs")
    expect_consumed({"occultism:otherstone_bricks": 6}, "chain step 2: the bricks it used")

    place("occultism:otherstone_bricks", SLAB_ROW)
    expect_result("occultism:otherstone_bricks_slab", 6, "chain step 3: three bricks")
    take_result("occultism:otherstone_bricks_slab", "chain step 3: six slabs")
    expect_consumed({"occultism:otherstone_bricks": 3}, "chain step 3: the bricks it used")

    place("occultism:otherstone_bricks_slab", CHISELED)
    expect_result("occultism:chiseled_otherstone_bricks", 1, "chain step 4: two brick slabs")
    take_result("occultism:chiseled_otherstone_bricks", "chain step 4: one chiseled block")
    expect_consumed(
        {"occultism:otherstone_bricks_slab": 4}, "chain step 4: the slabs it used"
    )
    close_table()

    # ---- the smelting pair, and the no-fuel differential ------------------
    print("== the cracked pair in a real furnace, and the same bricks with no fuel ==")

    def set_furnace(fx: int, fz: int) -> None:
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:furnace")
        wait(4)

    def container(fx: int, fz: int) -> list:
        return run("container", str(fx), str(y), str(fz)).get("slots") or []

    def smelt(stone: str, fx: int, with_fuel: bool) -> list:
        set_furnace(fx, FURNACE_Z)
        run("container", str(fx), str(y), str(FURNACE_Z), f"occultism:{stone}_bricks", "1")
        if with_fuel:
            run("container", str(fx), str(y), str(FURNACE_Z), "minecraft:coal", "1")
        wait(240)
        return container(fx, FURNACE_Z)

    for i, stone in enumerate(("otherstone", "otherrock")):
        fx = FURNACE_X + i * 4
        got = smelt(stone, fx, with_fuel=True)
        print(f"  furnace with coal: {json.dumps(got)}")
        output = got[2] if len(got) > 2 else None
        if not output or output.get("item") != f"occultism:cracked_{stone}_bricks":
            raise SystemExit(
                f"BRICKS FAIL: the furnace made {output} from {stone}_bricks, "
                f"expected one occultism:cracked_{stone}_bricks"
            )
        if output.get("count") != 1:
            raise SystemExit(f"BRICKS FAIL: the furnace made {output.get('count')} cracked bricks")
        if got[0] is not None:
            raise SystemExit(f"BRICKS FAIL: the furnace kept its input: {got[0]}")
        print(f"  {stone}_bricks + coal -> {output['item']} (the input is gone)")

    # The differential: no fuel, no cooking. The input must still be there.
    fx = FURNACE_X + 8
    got = smelt("otherstone", fx, with_fuel=False)
    print(f"  furnace without fuel: {json.dumps(got)}")
    if len(got) > 2 and got[2] is not None:
        raise SystemExit(f"BRICKS FAIL: a fuel-less furnace cooked {got[2]}")
    if not got or got[0] is None:
        raise SystemExit(f"BRICKS FAIL: the fuel-less furnace lost its input: {got}")
    print("  no fuel: the output stayed empty and the input stayed put")

    # ---- the twelve blocks: place, read back, hand 0, pickaxe 1 -----------
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
                raise SystemExit(f"BRICKS FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"BRICKS FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

    def drops(reply: dict) -> dict[str, int]:
        return reply.get("drops") or {}

    def drops_total(reply: dict) -> int:
        return sum(drops(reply).values())

    def hand_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"BRICKS FAIL: {block} did not place at {bx},{by},{bz}: '{got}'")
        expected = block + DEFAULT_STATE[shape(block.split(":")[1])]
        if got != expected:
            raise SystemExit(f"BRICKS FAIL: {block} reads '{got}', expected '{expected}'")
        print(f"  placed {got} at x={bx}")
        run("command", "clear @s")
        wait(4)
        reply = break_at(bx, by, bz)
        print(
            f"  {block} by hand: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"BRICKS FAIL: the hand break left {reply.get('block')} at x={bx}")
        if drops_total(reply) != 0:
            raise SystemExit(
                f"BRICKS FAIL: a hand break of {block} dropped {drops(reply)} "
                "(requires_tool must gate the table)"
            )

    def pickaxe_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"BRICKS FAIL: {block} did not place again: '{got}'")
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
            raise SystemExit(f"BRICKS FAIL: the pickaxe break left {reply.get('block')} at x={bx}")
        got_drops = drops(reply)
        if got_drops.get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"BRICKS FAIL: a wooden pickaxe dropped {got_drops} for {block}, "
                "expected exactly one of it"
            )

    print("== the twelve blocks: default state, hand 0, wooden pickaxe 1 ==")
    for i, name in enumerate(BLOCKS):
        block = f"occultism:{name}"
        bx = 214 + i * 2
        hand_break(block, bx, y, BLOCK_Z)
        pickaxe_break(block, bx, y, BLOCK_Z)

    # ---- two brick walls side by side connect (the `minecraft:walls` tag) --
    wall_a, wall_b = 214, 215
    wall_z = BLOCK_Z + 4
    set_block(wall_a, y, wall_z, "occultism:otherstone_bricks_wall")
    set_block(wall_b, y, wall_z, "occultism:otherstone_bricks_wall")
    state_a = state_at(wall_a, y, wall_z)["state"]
    state_b = state_at(wall_b, y, wall_z)["state"]
    print(f"  wall pair: {state_a} / {state_b}")
    if "west=low" not in state_b:
        raise SystemExit(
            f"BRICKS FAIL: the second brick wall does not connect to the first ('{state_b}'): "
            "the wall is not in minecraft:walls"
        )
    set_block(wall_a, y, wall_z, "minecraft:air")
    set_block(wall_b, y, wall_z, "minecraft:air")
    wait(4)

    # ---- the double brick slab: the jar table's `type=double` branch -------
    double_x = 224
    set_block(double_x, y, BLOCK_Z + 4, "occultism:otherstone_bricks_slab[type=double]")
    got = state_at(double_x, y, BLOCK_Z + 4)["state"]
    if got != "occultism:otherstone_bricks_slab[type=double,waterlogged=false]":
        raise SystemExit(f"BRICKS FAIL: the double brick slab reads '{got}'")
    run("command", "item replace entity @s hotbar.0 with minecraft:wooden_pickaxe 1")
    wait(4)
    run("select", "1")
    wait(2)
    reply = break_at(double_x, y, BLOCK_Z + 4)
    measured = drops(reply).get("occultism:otherstone_bricks_slab", 0)
    print(f"  double brick slab: the pickaxe break dropped {measured} (the jar table's double branch)")
    if reply.get("block") != "minecraft:air":
        raise SystemExit(f"BRICKS FAIL: the double brick slab did not break: {reply.get('block')}")
    if measured != 2 or drops_total(reply) != 2:
        raise SystemExit(
            f"BRICKS FAIL: the double brick slab dropped {drops(reply)}, expected exactly 2 "
            "(the table's match_block type=double branch)"
        )

    run("command", "clear @s")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
