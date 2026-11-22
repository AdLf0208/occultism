#!/usr/bin/env python3
"""Craft and exercise the six `otherstone`/`otherrock` building blocks.

Called by `tools/e2e.sh` (phase Q) with the open game's control socket and the
crafting table's position. The harness opens a real table as a screen whose 3x3
grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers.

The slice's rule (from the jar's `data/occultism/recipe/crafting/`): a shaped
recipe whose result and every ingredient the package already provides. These six
qualify — their ingredient is `occultism:otherstone` / `occultism:otherrock`,
both carried since 0.2.0, while the results were not registered until 0.2.6:

* `otherstone_slab`   `ooo`          in the middle row -> 6
* `otherstone_stairs` `#  `/`## `/`###`              -> 4
* `otherstone_wall`   `ooo`/`ooo`                     -> 6
* `otherrock_slab`, `otherrock_stairs`, `otherrock_wall` the same with otherrock

What is proven, in this order:

1. each recipe, filled the way it declares, reads its result in the result slot,
   is taken (the exact count lands in the inventory) and consumes every
   ingredient;
2. a differential: three dirt in the middle row make nothing, so the result is
   the recipe and not "any row";
3. each of the six blocks is placed by `setblock`, its default state is read back
   (the template's states, not a plain block's), a bare hand breaks it and rolls
   **nothing** (`requires_tool`, as upstream's `requiresCorrectToolForDrops`), a
   wooden pickaxe breaks it and drops **exactly one** of itself;
4. two walls side by side connect (`west=low` on the second), which is the
   `minecraft:walls` tag doing its work; without the tag they would stay apart;
5. the jar table's `type=double` branch: a double slab drops two, not one.

`BUILDING_NEGATIVE=1` asserts the slab grid makes **nothing**, so a working
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

# name, ingredient, ingredient count, grid cells, result count
BUILDING: list[tuple[str, str, int, list[int], int]] = [
    ("otherstone_slab", "occultism:otherstone", 3, SLAB_ROW, 6),
    ("otherstone_stairs", "occultism:otherstone", 6, STAIR_CELLS, 4),
    ("otherstone_wall", "occultism:otherstone", 6, WALL_CELLS, 6),
    ("otherrock_slab", "occultism:otherrock", 3, SLAB_ROW, 6),
    ("otherrock_stairs", "occultism:otherrock", 6, STAIR_CELLS, 4),
    ("otherrock_wall", "occultism:otherrock", 6, WALL_CELLS, 6),
]

# The default state each template declares, as `ctl block` reports it.
DEFAULT_STATE = {
    "slab": "[type=bottom,waterlogged=false]",
    "stairs": "[facing=north,half=bottom,shape=straight,waterlogged=false]",
    "wall": "[east=none,north=none,south=none,up=true,waterlogged=false,west=none]",
}

FLOOR_Z = 33  # the building row, two blocks north of the player's stance


def shape(name: str) -> str:
    if name.endswith("_slab"):
        return "slab"
    if name.endswith("_stairs"):
        return "stairs"
    return "wall"


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    negative = os.environ.get("BUILDING_NEGATIVE") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"BUILDING FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"BUILDING FAIL: no menu is open: {screen}")
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
            f"BUILDING FAIL: missing ingredient {item} in the open screen "
            f"(inventory: {inventory()})"
        )

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
        rc, _, _ = raw("close")
        if rc != 0:
            pass
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
            raise SystemExit("BUILDING FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"BUILDING FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"BUILDING FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"BUILDING FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    # ---- the recipes -------------------------------------------------------
    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("BUILDING FAIL: the building phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    for name, ingredient, needed, cells, count in BUILDING:
        close_table()
        fresh_inventory()
        run("give", ingredient, str(needed))
        wait(3)
        reset_table()
        place(ingredient, cells)
        if negative and name == "otherstone_slab":
            print("  (negative: asserting the 3-stone middle row makes nothing)")
            expect_result(None, 0, "negative: the slab grid")
            return 1
        expect_result(f"occultism:{name}", count, f"the {ingredient.split(':')[1]} {shape(name)} grid")
        take_result(f"the {name}")
        expect_consumed({ingredient: 0}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"BUILDING FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the differential: three dirt in the same row make nothing ---------
    close_table()
    fresh_inventory()
    run("give", "minecraft:dirt", "3")
    wait(3)
    reset_table()
    place("minecraft:dirt", SLAB_ROW)
    expect_result(None, 0, "three dirt in the middle row")
    close_table()

    # ---- the six blocks: place, read back, hand 0, pickaxe 1 --------------
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
                raise SystemExit(f"BUILDING FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"BUILDING FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

    def drops(reply: dict) -> dict[str, int]:
        return reply.get("drops") or {}

    def drops_total(reply: dict) -> int:
        return sum(drops(reply).values())

    def hand_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"BUILDING FAIL: {block} did not place at {bx},{by},{bz}: '{got}'")
        expected = block + DEFAULT_STATE[shape(block.split(":")[1])]
        if got != expected:
            raise SystemExit(f"BUILDING FAIL: {block} reads '{got}', expected '{expected}'")
        print(f"  placed {got} at x={bx}")
        run("command", "clear @s")
        wait(4)
        reply = break_at(bx, by, bz)
        print(f"  {block} by hand: {json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}")
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"BUILDING FAIL: the hand break left {reply.get('block')} at x={bx}")
        if drops_total(reply) != 0:
            raise SystemExit(
                f"BUILDING FAIL: a hand break of {block} dropped {drops(reply)} "
                "(requires_tool must gate the table)"
            )

    def pickaxe_break(block: str, bx: int, by: int, bz: int) -> None:
        set_block(bx, by, bz, block)
        got = state_at(bx, by, bz)["state"]
        if not got.startswith(block):
            raise SystemExit(f"BUILDING FAIL: {block} did not place again: '{got}'")
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
            raise SystemExit(f"BUILDING FAIL: the pickaxe break left {reply.get('block')} at x={bx}")
        got_drops = drops(reply)
        if got_drops.get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"BUILDING FAIL: a wooden pickaxe dropped {got_drops} for {block}, "
                "expected exactly one of it"
            )

    print("== the six blocks: default state, hand 0, wooden pickaxe 1 ==")
    for i, (name, _, _, _, _) in enumerate(BUILDING):
        block = f"occultism:{name}"
        bx = 226 + i * 2
        hand_break(block, bx, y, FLOOR_Z)
        pickaxe_break(block, bx, y, FLOOR_Z)

    # ---- two walls side by side connect (the `minecraft:walls` tag) --------
    wall_a, wall_b = 226, 227
    wall_z = FLOOR_Z + 3
    set_block(wall_a, y, wall_z, "occultism:otherstone_wall")
    set_block(wall_b, y, wall_z, "occultism:otherstone_wall")
    state_a = state_at(wall_a, y, wall_z)["state"]
    state_b = state_at(wall_b, y, wall_z)["state"]
    print(f"  wall pair: {state_a} / {state_b}")
    if "west=low" not in state_b:
        raise SystemExit(
            f"BUILDING FAIL: the second wall does not connect to the first ('{state_b}'): "
            "the wall is not in minecraft:walls"
        )
    set_block(wall_a, y, wall_z, "minecraft:air")
    set_block(wall_b, y, wall_z, "minecraft:air")
    wait(4)

    # ---- the double slab: the jar table's `type=double` branch ------------
    double_x = 236
    set_block(double_x, y, FLOOR_Z + 3, "occultism:otherstone_slab[type=double]")
    got = state_at(double_x, y, FLOOR_Z + 3)["state"]
    if got != "occultism:otherstone_slab[type=double,waterlogged=false]":
        raise SystemExit(f"BUILDING FAIL: the double slab reads '{got}'")
    run("command", "item replace entity @s hotbar.0 with minecraft:wooden_pickaxe 1")
    wait(4)
    run("select", "1")
    wait(2)
    reply = break_at(double_x, y, FLOOR_Z + 3)
    measured = drops(reply).get("occultism:otherstone_slab", 0)
    print(f"  double slab: the pickaxe break dropped {measured} (the jar table's double branch)")
    if reply.get("block") != "minecraft:air":
        raise SystemExit(f"BUILDING FAIL: the double slab did not break: {reply.get('block')}")
    if measured != 2 or drops_total(reply) != 2:
        raise SystemExit(
            f"BUILDING FAIL: the double slab dropped {drops(reply)}, expected exactly 2 "
            "(the table's match_block type=double branch)"
        )

    run("command", "clear @s")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
