#!/usr/bin/env python3
"""Craft the 17 impure-chalk recipes at a real table (slice 12, phase X).

Called by `tools/e2e.sh` with the open game's control socket and the crafting
table's position. The harness opens a real table whose 3x3 grid is slots 100..108
(row-major) and whose result is 200; the player's own slots keep their inventory
numbers.

The slice, read from the jar at `release/v1.21.1-1.225.0`:

* fifteen `minecraft:crafting_shapeless` grids: one `occultism:chalk_white_impure`
  plus three colour materials (or two, for gray and green), and
* two `minecraft:crafting_shaped` grids for the same `chalk_white_impure`: the
  jar ships them mirrored — `xy`/`xy`/`xy`, where one column is
  `#occultism:dusts/chalk_base` and the other `#c:dusts/otherworld_wood`
  (`chalk_white_impure.json` and `chalk_white_impure2.json`).

`chalk_base` is a **nested** tag (`#c:dusts/otherstone` + `#c:dusts/otherrock`);
the driver fills its cells with **both** members (burnt_otherstone and
burnt_otherrock) in the two white grids, so a nested tag that resolves to
nothing cannot pass. The tag-keyed cells elsewhere are filled with the tag's own
member (iesnium_dust, echo_dust, crushed_ice, …), and `tools/e2e.sh` greps the
game log for `invalid ingredient` after this phase.

What is proven, in this order:

1. each of the 17 grids, filled the way its recipe declares, makes exactly one
   impure chalk with every ingredient consumed and the grid emptied;
2. a wrong-ingredient grid (two demonic meat and one dirt, or two burnt stones
   with no otherworld wood) makes nothing;
3. the two carried smelting recipes the chalk base needs: the carried polished
   pair in a real furnace (200 ticks, 0.15 experience) becomes burnt otherstone /
   burnt otherrock with the input consumed, and a fuel-less furnace cooks
   nothing — the same shape the polished tier proved in phase S.

`CHALK_SELFTEST=1` asserts the first grid makes two, so a working package must
FAIL this run.

Exit 1 with the observed state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200
# The bricks/polished phase's furnace row (free once phase S has finished); the
# third furnace is the no-fuel differential.
FURNACE_X = 230
FURNACE_Z = 42

# (file name, result id, {item: cells}, count). The shaped white grids use both
# chalk-base members across their column; the shapeless ones put the colour
# materials in the first cells.
GRIDS = [
    (
        "chalk_white_impure",
        "chalk_white_impure",
        {
            "occultism:burnt_otherstone": [GRID[0], GRID[3]],
            "occultism:burnt_otherrock": [GRID[6]],
            "occultism:otherworld_ashes": [GRID[1], GRID[4], GRID[7]],
        },
        1,
    ),
    (
        "chalk_white_impure2",
        "chalk_white_impure",
        {
            "occultism:otherworld_ashes": [GRID[0], GRID[3], GRID[6]],
            "occultism:burnt_otherstone": [GRID[1], GRID[4]],
            "occultism:burnt_otherrock": [GRID[7]],
        },
        1,
    ),
    (
        "chalk_pink_impure",
        "chalk_pink_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:demonic_meat": [GRID[1], GRID[2], GRID[3]],
        },
        1,
    ),
    (
        "chalk_brown_impure",
        "chalk_brown_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:cruelty_essence": [GRID[1]],
            "minecraft:cocoa_beans": [GRID[2]],
            "minecraft:brown_mushroom": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_gray_impure",
        "chalk_gray_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:gray_paste": [GRID[1]],
        },
        1,
    ),
    (
        "chalk_cyan_impure",
        "chalk_cyan_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:iesnium_dust": [GRID[1]],
            "occultism:echo_dust": [GRID[2]],
            "minecraft:glow_ink_sac": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_black_impure",
        "chalk_black_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:witherite_dust": [GRID[1], GRID[2], GRID[3]],
        },
        1,
    ),
    (
        "chalk_green_impure",
        "chalk_green_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:nature_paste": [GRID[1]],
        },
        1,
    ),
    (
        "chalk_light_blue_impure",
        "chalk_light_blue_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:crushed_ice": [GRID[1]],
            "occultism:crushed_packed_ice": [GRID[2]],
            "occultism:crushed_blue_ice": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_purple_impure",
        "chalk_purple_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:crushed_end_stone": [GRID[1]],
            "occultism:obsidian_dust": [GRID[2], GRID[3]],
        },
        1,
    ),
    (
        "chalk_light_gray_impure",
        "chalk_light_gray_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:silver_dust": [GRID[1]],
            "occultism:iron_dust": [GRID[2]],
            "occultism:crushed_calcite": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_lime_impure",
        "chalk_lime_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:research_fragment_dust": [GRID[1]],
            "occultism:emerald_dust": [GRID[2]],
            "minecraft:slime_ball": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_orange_impure",
        "chalk_orange_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:cursed_honey": [GRID[1]],
            "minecraft:glow_berries": [GRID[2]],
            "minecraft:blaze_powder": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_yellow_impure",
        "chalk_yellow_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "minecraft:glowstone_dust": [GRID[1]],
            "occultism:gold_dust": [GRID[2], GRID[3]],
        },
        1,
    ),
    (
        "chalk_blue_impure",
        "chalk_blue_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:marid_essence": [GRID[1]],
            "occultism:lapis_dust": [GRID[2]],
            "minecraft:tube_coral": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_magenta_impure",
        "chalk_magenta_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:dragonyst_dust": [GRID[1]],
            "occultism:amethyst_dust": [GRID[2]],
            "minecraft:chorus_fruit": [GRID[3]],
        },
        1,
    ),
    (
        "chalk_red_impure",
        "chalk_red_impure",
        {
            "occultism:chalk_white_impure": [GRID[0]],
            "occultism:afrit_essence": [GRID[1]],
            "minecraft:torchflower": [GRID[2]],
            "minecraft:redstone": [GRID[3]],
        },
        1,
    ),
]


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    selftest = os.environ.get("CHALK_SELFTEST") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"CHALK FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"CHALK FAIL: no menu is open: {screen}")
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
            f"CHALK FAIL: missing ingredient {item} in the open screen "
            f"(inventory: {inventory()})"
        )

    def empty_or_same(item: str) -> int:
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
            raise SystemExit(f"CHALK FAIL: no inventory slot for {item} ({inventory()})")
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
        if not slots():
            raise SystemExit("CHALK FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"CHALK FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"CHALK FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"CHALK FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    def expect_grid_empty(what: str) -> None:
        for number in GRID:
            got = item_of(number)
            if got is not None:
                raise SystemExit(f"CHALK FAIL: {what} left {got} in grid slot {number}")
        print(f"  {what}: the grid is empty")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("CHALK FAIL: the chalks phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    # ---- the 17 grids, one fresh table each --------------------------------
    print("== the 17 impure-chalk grids ==")
    for index, (file, result, cells_by_item, count) in enumerate(GRIDS):
        close_table()
        fresh_inventory()
        for item, cells in cells_by_item.items():
            run("give", item, str(len(cells)))
            wait(2)
        reset_table()
        place(cells_by_item)
        what = f"the {file} grid"
        if selftest and index == 0:
            print("  (selftest: asserting the chalk_white_impure grid makes two)")
            expect_result(f"occultism:{result}", 2, f"selftest: {what} -> {result}")
            continue
        expect_result(f"occultism:{result}", count, f"{what} -> {result}")
        take_result(f"occultism:{result}", f"the {result}")
        expect_consumed({item: 0 for item in cells_by_item}, f"the {result} recipe")
        expect_grid_empty(f"the {result} recipe")
        if in_inventory(f"occultism:{result}") != count:
            raise SystemExit(
                f"CHALK FAIL: the crafted {result} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the wrong grids: nothing -----------------------------------------
    print("== the wrong grids: nothing ==")
    for label, cells_by_item in (
        # One wrong material in an otherwise exact pink grid.
        (
            "the pink grid with dirt for one demonic meat",
            {
                "occultism:chalk_white_impure": [GRID[0]],
                "occultism:demonic_meat": [GRID[1], GRID[2]],
                "minecraft:dirt": [GRID[3]],
            },
        ),
        # The shaped white grid's two chalk-base columns and no otherworld wood.
        (
            "two burnt-stone columns and no otherworld wood",
            {
                "occultism:burnt_otherstone": [GRID[0], GRID[1]],
                "occultism:burnt_otherrock": [GRID[3], GRID[4]],
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

    # ---- the two burnt dusts in a real furnace, and the no-fuel differential
    print("== the burnt pair in a real furnace, and the same stone with no fuel ==")

    def set_furnace(fx: int, fz: int) -> None:
        # Air first: `/setblock` of the same block state keeps the existing block
        # entity, so a furnace left by phase S would answer with its own leftovers.
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:air")
        wait(2)
        run("command", f"execute in minecraft:the_end run setblock {fx} {y} {fz} minecraft:furnace")
        wait(4)

    def container(fx: int, fz: int) -> list:
        return run("container", str(fx), str(y), str(fz)).get("slots") or []

    def smelt(polished: str, fx: int, with_fuel: bool) -> list:
        set_furnace(fx, FURNACE_Z)
        run("container", str(fx), str(y), str(FURNACE_Z), f"occultism:{polished}", "1")
        if with_fuel:
            run("container", str(fx), str(y), str(FURNACE_Z), "minecraft:coal", "1")
        wait(240)
        return container(fx, FURNACE_Z)

    for i, polished in enumerate(("polished_otherstone", "polished_otherrock")):
        fx = FURNACE_X + i * 4
        got = smelt(polished, fx, with_fuel=True)
        print(f"  furnace with coal: {json.dumps(got)}")
        output = got[2] if len(got) > 2 else None
        want = f"occultism:burnt_{polished.removeprefix('polished_')}"
        if not output or output.get("item") != want:
            raise SystemExit(
                f"CHALK FAIL: the furnace made {output} from {polished}, expected one {want}"
            )
        if output.get("count") != 1:
            raise SystemExit(f"CHALK FAIL: the furnace made {output.get('count')} burnt dusts")
        if got[0] is not None:
            raise SystemExit(f"CHALK FAIL: the furnace kept its input: {got[0]}")
        print(f"  {polished} + coal -> {output['item']} (the input is gone)")

    fx = FURNACE_X + 8
    got = smelt("polished_otherstone", fx, with_fuel=False)
    print(f"  furnace without fuel: {json.dumps(got)}")
    if len(got) > 2 and got[2] is not None:
        raise SystemExit(f"CHALK FAIL: a fuel-less furnace cooked {got[2]}")
    if not got or got[0] is None:
        raise SystemExit(f"CHALK FAIL: the fuel-less furnace lost its input: {got}")
    print("  no fuel: the output stayed empty and the input stayed put")
    return 0


if __name__ == "__main__":
    sys.exit(main())
