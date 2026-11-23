#!/usr/bin/env python3
"""Craft, place, break and press the four `otherstone`/`otherrock` redstone blocks.

Called by `tools/e2e.sh` (phase U) with the open game's control socket and the
crafting table's position. The harness opens a real table as a screen whose 3x3
grid is slots 100..108 (row-major) and whose result is 200; the player's own
slots keep their inventory numbers.

The slice's rule (from the jar's `data/occultism/recipe/crafting/`,
`.../loot_table/blocks/` and `OccultismBlocks.java` at
`release/v1.21.1-1.225.0`): the **redstone pair** of the building family is
`otherstone_button`, `otherrock_button`, `otherstone_pressure_plate` and
`otherrock_pressure_plate`.

What is proven, in this order:

1. the jar's reverse smelt pair (`smelting/otherstone`, `smelting/otherrock`):
   one cobble block + one coal in a real furnace makes exactly one stone block
   after 200 ticks and consumes the input, while the same input without fuel
   stays put and cooks nothing — the tier's loop closes;
2. each of the four grids, filled the way its recipe declares, reads its result
   in the result slot, is taken (the exact count lands in the inventory) and
   consumes every ingredient: one stone makes one button (shapeless), two stones
   in one row make one pressure plate (shaped `oo`);
3. the wrong-ingredient differentials: one cobble block in the button grid and
   two cobble blocks in the plate grid make **nothing**, and one stone in the
   plate grid makes the *button* (the shapeless one-stone recipe) rather than the
   plate — so a grid's result is its recipe, not "any stone" and not "any row";
4. each of the four blocks is placed by `setblock`, its default state is read
   back (the button's `face=wall,facing=north,powered=false` on a stone support,
   the plate's `powered=false`), a bare hand drops **exactly one** of it (the
   templates are `requires_tool: false`, as upstream's `strength(0.5F)` without
   `requiresCorrectToolForDrops`), a wooden pickaxe drops exactly one as well,
   and a bare-hand break of `occultism:otherstone` still drops nothing — the
   differential that proves the hand leg can fail;
5. the **press**: a plate under the player reads `powered=true` and reads
   `powered=false` after stepping off (its trigger is a physics scan, so the
   assertion is the block state, not a block entity);
6. the **click**: a button right-clicked from three blocks away reads
   `powered=true`, is still pressed ten ticks later and reads `powered=false`
   after the template's press duration.

`PLATE_SELFTEST=1` and `BUTTON_SELFTEST=1` each invert their own leg (the
assertion becomes "not powered"), so a working package must fail those runs.

Exit 1 with the observed state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200
BUTTON_CELL = [GRID[4]]  # the centre: a shapeless one-item recipe matches anywhere
PLATE_CELLS = [GRID[3], GRID[4]]  # the middle row's first two cells: `oo`

# name, ingredient, grid cells, result count
REDSTONE = [
    ("otherstone_button", "occultism:otherstone", BUTTON_CELL, 1),
    ("otherstone_pressure_plate", "occultism:otherstone", PLATE_CELLS, 1),
    ("otherrock_button", "occultism:otherrock", BUTTON_CELL, 1),
    ("otherrock_pressure_plate", "occultism:otherrock", PLATE_CELLS, 1),
]

# The four blocks, in the order the phase places them.
BLOCKS = [
    "otherstone_button",
    "otherstone_pressure_plate",
    "otherrock_button",
    "otherrock_pressure_plate",
]

# The row the four blocks go on (the cobble row: every block there has been
# broken by phase T by the time this runs). A button needs a wall: its support
# goes one block north, and the default state (`face=wall,facing=north`) then
# attaches to it. The press/click stations sit two rows further north, and the
# furnace pair on the row between. `break_at` stands two blocks south.
BLOCK_Z = 46
SUPPORT_Z = 45
FURNACE_Z = 44
PRESS_Z = 48
TABLE_X = 222
TABLE_Z = 38
FURNACE_X = 214

# The default state each template declares, as `ctl block` reports it.
DEFAULT_STATE = {
    "otherstone_button": "[face=wall,facing=north,powered=false]",
    "otherrock_button": "[face=wall,facing=north,powered=false]",
    "otherstone_pressure_plate": "[powered=false]",
    "otherrock_pressure_plate": "[powered=false]",
}


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    plate_selftest = os.environ.get("PLATE_SELFTEST") == "1"
    button_selftest = os.environ.get("BUTTON_SELFTEST") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"REDSTONE FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"REDSTONE FAIL: no menu is open: {screen}")
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
            f"REDSTONE FAIL: missing ingredient {item} in the open screen "
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
            raise SystemExit(f"REDSTONE FAIL: no inventory slot for {item} ({inventory()})")
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
            raise SystemExit("REDSTONE FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"REDSTONE FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"REDSTONE FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"REDSTONE FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("REDSTONE FAIL: the redstone phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    def set_block(bx: int, by: int, bz: int, state: str) -> None:
        run("command", f"execute in minecraft:the_end run setblock {bx} {by} {bz} {state}")
        wait(4)

    def state_at(bx: int, by: int, bz: int) -> dict:
        return run("block", str(bx), str(by), str(bz))

    # ---- the reverse smelt pair, and the no-fuel differential -------------
    print("== the reverse smelt pair: the cobble block back into the stone ==")

    def set_furnace(fx: int, fz: int) -> None:
        # Air first: `/setblock` of the *same* state keeps the block entity, so a
        # furnace left at this position would answer with its own leftovers.
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:air")
        wait(2)
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:furnace")
        wait(4)

    def container(fx: int, fz: int) -> list:
        return run("container", str(fx), str(y), str(fz)).get("slots") or []

    def smelt(cobble: str, fx: int, with_fuel: bool) -> list:
        set_furnace(fx, FURNACE_Z)
        run("container", str(fx), str(y), str(FURNACE_Z), f"occultism:{cobble}", "1")
        if with_fuel:
            run("container", str(fx), str(y), str(FURNACE_Z), "minecraft:coal", "1")
        wait(240)
        return container(fx, FURNACE_Z)

    for i, (stone, cobble) in enumerate(
        (("otherstone", "othercobblestone"), ("otherrock", "othercobblerock"))
    ):
        fx = FURNACE_X + i * 4
        got = smelt(cobble, fx, with_fuel=True)
        print(f"  furnace with coal: {json.dumps(got)}")
        output = got[2] if len(got) > 2 else None
        if not output or output.get("item") != f"occultism:{stone}":
            raise SystemExit(
                f"REDSTONE FAIL: the furnace made {output} from {cobble}, "
                f"expected one occultism:{stone}"
            )
        if output.get("count") != 1:
            raise SystemExit(f"REDSTONE FAIL: the furnace made {output.get('count')} {stone} blocks")
        if got[0] is not None:
            raise SystemExit(f"REDSTONE FAIL: the furnace kept its input: {got[0]}")
        print(f"  {cobble} + coal -> occultism:{stone} (the input is gone)")

    # The differential: no fuel, no cooking. The input must still be there.
    fx = FURNACE_X + 8
    got = smelt("othercobblestone", fx, with_fuel=False)
    print(f"  furnace without fuel: {json.dumps(got)}")
    if len(got) > 2 and got[2] is not None:
        raise SystemExit(f"REDSTONE FAIL: a fuel-less furnace cooked {got[2]}")
    if not got or got[0] is None:
        raise SystemExit(f"REDSTONE FAIL: the fuel-less furnace lost its input: {got}")
    print("  no fuel: the output stayed empty and the input stayed put")
    for i in range(3):
        set_block(FURNACE_X + i * 4, y, FURNACE_Z, "minecraft:air")

    # ---- the four grids, one fresh table each -----------------------------
    print("== the four redstone grids ==")
    for name, ingredient, cells, count in REDSTONE:
        close_table()
        fresh_inventory()
        run("give", ingredient, str(len(cells)))
        wait(3)
        reset_table()
        place(ingredient, cells)
        expect_result(f"occultism:{name}", count, f"the {name} grid")
        take_result(f"occultism:{name}", f"the {name}")
        expect_consumed({ingredient: 0}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"REDSTONE FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the wrong-ingredient differentials -------------------------------
    print("== the wrong ingredients: nothing in the same grids ==")
    for label, item, cells, expect in (
        # The button grid takes the smooth stone, not the cobble tier.
        (
            "one othercobblestone in the button grid",
            "occultism:othercobblestone",
            BUTTON_CELL,
            None,
        ),
        (
            "one othercobblerock in the button grid",
            "occultism:othercobblerock",
            BUTTON_CELL,
            None,
        ),
        (
            "two othercobblestone in the plate grid",
            "occultism:othercobblestone",
            PLATE_CELLS,
            None,
        ),
        # The plate's shape needs both cells: one stone alone is the *button*
        # recipe (shapeless, one stone), which is why this grid must be two.
        (
            "one otherstone in the plate grid",
            "occultism:otherstone",
            [GRID[3]],
            "occultism:otherstone_button",
        ),
    ):
        close_table()
        fresh_inventory()
        run("give", item, str(len(cells)))
        wait(3)
        reset_table()
        place(item, cells)
        if expect is None:
            expect_result(None, 0, label)
        else:
            expect_result(expect, 1, f"{label} (the button recipe, not the plate)")
    close_table()

    # ---- the four blocks: place, read back, hand 1, pickaxe 1 -------------
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
                raise SystemExit(f"REDSTONE FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"REDSTONE FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

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
        """Place the block (with a stone support for a button) and read it back."""
        # A break's reply counts the drops at the block's position, so a leftover
        # entity from the previous leg (the hand break's own drop, two blocks away
        # from a player who never walked there) merges into the next reply and reads
        # 2 (measured on this phase's second run, 2026-11-23). Clear the world's
        # item entities first; this is a harness fix, the table rolls once.
        run("command", "kill @e[type=minecraft:item]")
        wait(2)
        if block.endswith("_button"):
            set_block(bx, y, SUPPORT_Z, "minecraft:stone")
        set_block(bx, y, bz, block)
        got = state_at(bx, y, bz)["state"]
        expected = block + DEFAULT_STATE[block.split(":")[1]]
        if got != expected:
            raise SystemExit(f"REDSTONE FAIL: {block} reads '{got}', expected '{expected}'")
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
            raise SystemExit(f"REDSTONE FAIL: the hand break left {reply.get('block')} at x={bx}")
        if drops(reply).get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"REDSTONE FAIL: a hand break of {block} dropped {drops(reply)}, "
                "expected exactly one (the template is requires_tool: false)"
            )
        if block.endswith("_button"):
            set_block(bx, y, SUPPORT_Z, "minecraft:air")

    def pickaxe_break(block: str, bx: int, bz: int) -> None:
        lay(block, bx, bz)
        hold("minecraft:wooden_pickaxe")
        reply = break_at(bx, y, bz)
        print(
            f"  {block} with a wooden pickaxe: "
            f"{json.dumps({k: reply.get(k) for k in ('block_before', 'block', 'drops')})}"
        )
        if reply.get("block") != "minecraft:air":
            raise SystemExit(f"REDSTONE FAIL: the pickaxe break left {reply.get('block')} at x={bx}")
        if drops(reply).get(block) != 1 or drops_total(reply) != 1:
            raise SystemExit(
                f"REDSTONE FAIL: a wooden pickaxe dropped {drops(reply)} for {block}, "
                "expected exactly one of it"
            )
        if block.endswith("_button"):
            set_block(bx, y, SUPPORT_Z, "minecraft:air")

    print("== the four blocks: default state, hand 1, wooden pickaxe 1 ==")
    for i, name in enumerate(BLOCKS):
        block = f"occultism:{name}"
        bx = 214 + i * 2
        hand_break(block, bx, BLOCK_Z)
        pickaxe_break(block, bx, BLOCK_Z)

    # ---- the differential that makes the hand leg fail-capable -----------
    # `otherstone` is `requires_tool: true` (0.2.4's pair): a bare hand breaks it
    # and rolls NOTHING. If the hand break of the four above dropped one only
    # because "hand breaks drop", this assertion could not differ.
    diff_x = 222
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    set_block(diff_x, y, BLOCK_Z, "occultism:otherstone")
    got = state_at(diff_x, y, BLOCK_Z)["state"]
    if got != "occultism:otherstone":
        raise SystemExit(f"REDSTONE FAIL: otherstone did not place: '{got}'")
    run("command", "clear @s")
    wait(4)
    reply = break_at(diff_x, y, BLOCK_Z)
    print(f"  differential, otherstone by hand: {json.dumps(drops(reply))}")
    if drops_total(reply) != 0:
        raise SystemExit(
            f"REDSTONE FAIL: a hand break of otherstone dropped {drops(reply)}; "
            "the differential that makes this family's hand-1 result meaningful is broken"
        )

    # ---- the press: a plate under the player reads powered=true -----------
    def press(plate: str, bx: int) -> None:
        block = f"occultism:{plate}"
        got = lay(block, bx, PRESS_Z)
        print(f"  press station: placed {got} at x={bx}")
        run("command", f"execute in minecraft:the_end run tp @s {bx}.5 101 {PRESS_Z}.5")
        wait(6)
        on = state_at(bx, y, PRESS_Z)["state"]
        print(f"  standing on {block}: {on}")
        if plate_selftest:
            print("  (plate selftest: asserting the plate is NOT powered under the player)")
            if "powered=true" in on:
                raise SystemExit(
                    f"REDSTONE FAIL: plate selftest: {block} reads '{on}' under the player "
                    "(this run must fail)"
                )
        else:
            if "powered=true" not in on:
                raise SystemExit(
                    f"REDSTONE FAIL: {block} reads '{on}' under the player, "
                    "expected powered=true"
                )
        run("command", f"execute in minecraft:the_end run tp @s {bx}.5 101 {PRESS_Z + 3}.5")
        wait(30)
        off = state_at(bx, y, PRESS_Z)["state"]
        print(f"  stepped off (30 ticks later): {off}")
        if "powered=false" not in off:
            raise SystemExit(
                f"REDSTONE FAIL: {block} reads '{off}' after the player stepped off, "
                "expected powered=false"
            )
        set_block(bx, y, PRESS_Z, "minecraft:air")

    for i, plate in enumerate(("otherstone_pressure_plate", "otherrock_pressure_plate")):
        press(plate, 214 + i * 4)

    # ---- the click: a button reads powered=true, then false ---------------
    def click(button: str, bx: int) -> None:
        block = f"occultism:{button}"
        got = lay(block, bx, PRESS_Z)
        print(f"  click station: placed {got} at x={bx}")
        run("command", f"execute in minecraft:the_end run tp @s {bx}.5 101 {PRESS_Z + 3}.5")
        wait(5)
        run("look-at-block", str(bx), str(y), str(PRESS_Z), "up")
        wait(2)
        run("interact", str(bx), str(y), str(PRESS_Z))
        wait(2)
        on = state_at(bx, y, PRESS_Z)["state"]
        print(f"  after the click: {on}")
        if button_selftest:
            print("  (button selftest: asserting the button was NOT pressed by the click)")
            if "powered=true" in on:
                raise SystemExit(
                    f"REDSTONE FAIL: button selftest: {block} reads '{on}' after the click "
                    "(this run must fail)"
                )
        else:
            if "powered=true" not in on:
                raise SystemExit(
                    f"REDSTONE FAIL: {block} reads '{on}' after the click, "
                    "expected powered=true"
                )
        wait(10)
        held = state_at(bx, y, PRESS_Z)["state"]
        print(f"  ten ticks later: {held}")
        if "powered=true" not in held:
            raise SystemExit(
                f"REDSTONE FAIL: {block} reads '{held}' ten ticks after the click, "
                "expected it still pressed"
            )
        wait(20)
        off = state_at(bx, y, PRESS_Z)["state"]
        print(f"  twenty ticks after that: {off}")
        if "powered=false" not in off:
            raise SystemExit(
                f"REDSTONE FAIL: {block} reads '{off}' after its press duration, "
                "expected powered=false (the template's ticksToStayPressed is 20)"
            )
        set_block(bx, y, PRESS_Z, "minecraft:air")

    for i, button in enumerate(("otherstone_button", "otherrock_button")):
        click(button, 218 + i * 4)

    run("command", "clear @s")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
