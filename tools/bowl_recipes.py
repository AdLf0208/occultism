#!/usr/bin/env python3
"""Craft, place, fill and break the six sacrificial bowls (slice 11, phase W).

Called by `tools/e2e.sh` with the open game's control socket and the crafting
table's position. The harness opens a real table whose 3x3 grid is slots 100..108
(row-major) and whose result is 200; the player's own slots keep their inventory
numbers.

The slice's rule, read from the jar at `release/v1.21.1-1.225.0`:

* **`SacrificialBowlBlock.useItemOn`**: a sneaking click changes nothing; an empty
  bowl takes the held stack through `insertItem(0, held, false)` — the block
  entity's `ItemStackHandler(1)` has `getSlotLimit → 1`, so one item goes in and
  the rest stays in the hand; a full bowl with an empty hand gives its stack back
  with `setItemInHand(hand, extractItem(0, 64, false))`; a full bowl with a full
  hand gives it to the inventory (`ItemHandlerHelper.giveItemToPlayer`). Upstream
  answers `SUCCESS` for every click on one of its bowls.
* **`SacrificialBowlBlockEntity.onContentsChanged`**: with a spirit fire **below**,
  a **non-golden** bowl replaces what it holds with the spirit-fire recipe's
  result (the jar excludes `otherflower`); the golden pair's block entity
  (`GoldenSacrificialBowlBlockEntity`) skips that check and only starts rituals.
* **`StorageUtil.dropInventoryItems`** on removal: what the bowl holds is spilled
  when it is broken (the engine's `Containers.dropContents` for a declared
  container), beside the bowl's own loot table.
* Every bowl is `Properties.of().strength(1.5, 30).noOcclusion()` and **none**
  calls `requiresCorrectToolForDrops`: `requires_tool` is false, so a bare hand
  drops exactly one (the `minecraft:stone` template would otherwise gate it).

What is proven, in this order:

1. each of the six grids, filled the way its recipe declares, makes exactly one
   bowl with every ingredient consumed (and the wrong shapes make nothing);
2. each of the six blocks is placed and read back;
3. the storage round trip on a plain bowl and a golden one: a component-carrying
   stack (a damaged diamond sword) goes in, is read back **through the block
   entity** (`/occultism info` and `/data get block`), and comes back to an empty
   hand with its component intact (`/data get entity @s SelectedItem`);
4. a stack of five puts exactly one in the bowl and leaves four in the hand (the
   jar's slot limit);
5. breaking a bowl with an item in it drops the bowl **and** the item;
6. the conversion: a feather in a plain bowl over a spirit fire becomes an
   awakened feather; the same in a golden bowl stays a feather;
7. a sneaking click changes nothing;
8. each bowl drops exactly one to a bare hand and to a wooden pickaxe, with
   `otherstone`'s bare-hand break (nothing) as the differential.

`BOWL_NEGATIVE=1` asserts the first grid makes nothing and `BOWL_SELFTEST=1`
asserts the returned stack is *not* the one that went in; both runs must FAIL.

Exit 1 with the observed state on the first mismatch.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

GRID = list(range(100, 109))
RESULT = 200
# The bowl's ring: the middle row and the two corners of the top row (`o o`/`ooo`).
RING = [GRID[0], GRID[2], GRID[3], GRID[4], GRID[5]]
# The golden ring: eight gold around the bowl in the centre.
GOLD_RING = [GRID[i] for i in (0, 1, 2, 3, 5, 6, 7, 8)]
CENTRE = [GRID[4]]

# name (result id), recipe file, {ingredient: cells}, result count
GRIDS = [
    (
        "sacrificial_bowl",
        "sacrificial_bowl",
        {"occultism:otherstone": RING},
        1,
    ),
    (
        "dark_sacrificial_bowl",
        "dark_sacrificial_bowl",
        {"occultism:otherrock": RING},
        1,
    ),
    (
        "golden_sacrificial_bowl",
        "golden_sacrificial_bowl",
        {
            "minecraft:gold_ingot": GOLD_RING,
            "occultism:sacrificial_bowl": CENTRE,
        },
        1,
    ),
    (
        "dark_golden_sacrificial_bowl",
        "dark_golden_sacrificial_bowl",
        {
            "minecraft:gold_ingot": GOLD_RING,
            "occultism:dark_sacrificial_bowl": CENTRE,
        },
        1,
    ),
    (
        "copper_sacrificial_bowl",
        "copper_sacrificial_bowl",
        {
            "occultism:sacrificial_bowl": [GRID[0]],
            "minecraft:copper_ingot": [GRID[1]],
        },
        1,
    ),
    (
        "dark_copper_sacrificial_bowl",
        "dark_copper_sacrificial_bowl",
        {
            "occultism:dark_sacrificial_bowl": [GRID[0]],
            "minecraft:copper_ingot": [GRID[1]],
        },
        1,
    ),
]

BOWLS = [grid[0] for grid in GRIDS]

# The row the bowls are placed on, one block apart; the table is the phase-P one.
BLOCK_Z = 44
TABLE_X = 222
TABLE_Z = 38

# The stack the storage round trip moves: a sword with a component on it, so a
# write that dropped the component would be visible.
CARRIED = "minecraft:diamond_sword"
CARRIED_WITH_DAMAGE = "minecraft:diamond_sword[minecraft:damage=5]"


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
    negative = os.environ.get("BOWL_NEGATIVE") == "1"
    selftest = os.environ.get("BOWL_SELFTEST") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"BOWL FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 4) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"BOWL FAIL: no menu is open: {screen}")
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
            f"BOWL FAIL: missing ingredient {item} in the open screen "
            f"(inventory: {inventory()})"
        )

    def empty_or_same(item: str) -> int:
        """A slot the crafted stack can go into: a same-item stack with room first
        (so two crafts merge, as a player's would), then an empty one."""
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
            raise SystemExit(f"BOWL FAIL: no inventory slot for {item} ({inventory()})")
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
            raise SystemExit("BOWL FAIL: the crafting table did not open")

    def fresh_inventory() -> None:
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(3)

    def expect_result(want: str | None, count: int, what: str) -> None:
        got, got_count = item_of(RESULT), count_of(RESULT)
        if want is None:
            if got is not None:
                raise SystemExit(f"BOWL FAIL: {what} made {got} x{got_count}, expected nothing")
            print(f"  {what}: the result slot stayed empty")
            return
        if got != want or got_count != count:
            raise SystemExit(
                f"BOWL FAIL: {what} made {got} x{got_count}, expected {want} x{count}"
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
                    f"BOWL FAIL: {what} left {got} {item}, expected {left} "
                    f"(inventory {inventory()})"
                )
        print(f"  {what}: every ingredient was consumed ({inventory()})")

    run("command", "gamemode survival @s")
    for _ in range(40):
        if str(run("status")["server"].get("game_mode")) == "0":
            break
        wait(1)
    else:
        raise SystemExit("BOWL FAIL: the bowls phase needs a survival player")
    print(f"  survival, table at {x},{y},{z}")

    def set_block(bx: int, by: int, bz: int, state: str) -> None:
        run("command", f"execute in minecraft:the_end run setblock {bx} {by} {bz} {state}")
        wait(4)

    def place_bowl(bx: int, by: int, bz: int, name: str) -> None:
        """Put a **fresh** bowl down: `/setblock` of the block already standing
        there is a no-op and keeps its block entity, so the air write is what makes
        the next leg start from an empty bowl (measured 2026-11-23: the dirty bowl
        of the slot-limit leg handed its dirt to the player and took no apple)."""
        set_block(bx, by, bz, "minecraft:air")
        run("command", "kill @e[type=minecraft:item]")
        wait(2)
        set_block(bx, by, bz, f"occultism:{name}")

    def state_at(bx: int, by: int, bz: int) -> str:
        return run("block", str(bx), str(by), str(bz)).get("block") or ""

    def near(bx: int, by: int, bz: int, polls: int = 30) -> None:
        """Wait until the client's player has arrived beside the block: a click
        the client still sends from its old position is refused as out of reach
        (`interact` answers `{"error": "block … is N blocks away"}`), which a
        swallowed reply turns into a silently missing click (measured 2026-11-23
        on this driver's first run)."""
        for _ in range(polls):
            position = run("state")["player"]["position"]
            # Horizontally beside it, at the block's own height or one above/below
            # (a player stands on the floor beside a block placed a level up).
            if (
                abs(position[0] - bx) < 1.0
                and abs(position[2] - bz) < 1.0
                and abs(position[1] - by) < 3.0
            ):
                return
            wait(2)
        raise SystemExit(
            f"BOWL FAIL: the client never arrived beside {bx},{by},{bz} "
            f"(player at {run('state')['player']['position']})"
        )

    def stand(bx: int, by: int, bz: int) -> None:
        """Walk the player beside the block **server-side** and wait for the client:
        `ctl tp` sets the client's position alone, and the server then answers
        `moved wrongly!` and snaps the player back (measured in the full run,
        2026-11-23), which leaves the click out of reach."""
        run("command", f"execute in minecraft:the_end run tp @s {bx}.5 {by} {bz}.5")
        wait(3)
        near(bx, by, bz)

    def click(
        bx: int, by: int, bz: int, face: str = "up", stand_y: int | None = None
    ) -> dict:
        """Stand beside the block and right-click it, asserting the click landed.
        `stand_y` is where the player stands when the clicked block is a level up
        (a bowl on a spirit fire) — the floor, not the bowl's own height."""
        stand(bx, by if stand_y is None else stand_y, bz + 2)
        run("look-at-block", str(bx), str(by), str(bz), face)
        wait(2)
        reply = run("interact", str(bx), str(by), str(bz), face)
        wait(4)
        if "error" in reply:
            raise SystemExit(
                f"BOWL FAIL: the right click at {bx},{by},{bz} was refused: {reply}"
            )
        return reply

    def hand_counts() -> dict[str, int]:
        return run("inventory")["counts"]

    def select_hotbar(item: str, count: int = 1) -> None:
        contract, out, err = raw(
            "command",
            f"item replace entity @s hotbar.0 with {item} {count}",
        )
        if contract != 0:
            raise SystemExit(f"BOWL FAIL: cannot put {item} in the hand: {out}{err}")
        wait(2)
        run("select", "1")
        wait(2)

    marker = [0]

    def marked(command: str, pattern: str) -> str:
        """Run a command as the owner (level 4) through chat, after a unique marker
        message, and return the reply matching `pattern`.

        `ctl messages` is **newest first** and keeps earlier replies, so "the last
        line matching" hands back a stale answer: the readout of the *previous*
        click (measured 2026-11-23, where it read the sword back after the take-out
        click had already emptied the bowl). The marker tells the two apart: the
        reply is above (newer than) the marker the command was sent after."""
        marker[0] += 1
        token = f"bowl marker {marker[0]}"
        run("chat", token)
        run("chat", f"/{command}")
        seen = []
        for _ in range(12):
            wait(3)
            chat = run("messages", "40").get("chat", [])
            seen = chat
            at = next((index for index, line in enumerate(chat) if token in line), None)
            if at is None:
                continue
            above = [line for line in chat[:at] if pattern in line]
            if above:
                return above[-1]
        raise SystemExit(
            f"BOWL FAIL: /{command} gave no {pattern!r} reply after its marker "
            f"(messages {json.dumps(seen)})"
        )

    def info(bx: int, by: int, bz: int) -> str:
        return marked(f"occultism info {bx} {by} {bz}", "stored=")

    # ---- the six grids, one fresh table each ------------------------------
    print("== the six bowl grids ==")
    for name, file, cells_by_item, count in GRIDS:
        close_table()
        fresh_inventory()
        for item, cells in cells_by_item.items():
            run("give", item, str(len(cells)))
            wait(2)
        reset_table()
        place(cells_by_item)
        what = f"the {file} grid"
        if negative and name == "sacrificial_bowl":
            print("  (negative: asserting the sacrificial_bowl grid makes nothing)")
            expect_result(None, 0, f"negative: {what}")
            continue
        expect_result(f"occultism:{name}", count, f"{what} -> {name}")
        take_result(f"occultism:{name}", f"the {name}")
        expect_consumed({item: 0 for item in cells_by_item}, f"the {name} recipe")
        if in_inventory(f"occultism:{name}") != count:
            raise SystemExit(
                f"BOWL FAIL: the crafted {name} is not in the inventory "
                f"{count} times ({inventory()})"
            )

    # ---- the wrong shapes: nothing ----------------------------------------
    print("== the wrong grids: nothing ==")
    for label, cells_by_item in (
        # The bowl needs five stones: the four corners alone are no bowl.
        (
            "four otherstone in the ring's corners",
            {"occultism:otherstone": [GRID[0], GRID[2], GRID[6], GRID[8]]},
        ),
        # The golden bowl needs eight ingots around the bowl.
        (
            "seven gold and one bowl",
            {
                "minecraft:gold_ingot": GOLD_RING[:-1],
                "occultism:sacrificial_bowl": CENTRE,
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

    # ---- the six blocks: placed and read back ------------------------------
    print("== the six bowls: placed and read back ==")
    run("command", "clear @s")
    run("command", "kill @e[type=minecraft:item]")
    wait(3)
    for index, name in enumerate(BOWLS):
        bx = 226 + index * 2
        set_block(bx, y, BLOCK_Z, f"occultism:{name}")
        got = state_at(bx, y, BLOCK_Z)
        if got != f"occultism:{name}":
            raise SystemExit(
                f"BOWL FAIL: occultism:{name} reads '{got}' at x={bx}"
            )
        print(f"  placed {got} at x={bx}")

    # ---- the storage round trip, with a component on the stack -------------
    print("== the storage round trip: a damaged sword in and back ==")
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    for name, bx in (("sacrificial_bowl", 226), ("golden_sacrificial_bowl", 234)):
        run("command", "clear @s")
        wait(2)
        place_bowl(bx, y, BLOCK_Z, name)
        select_hotbar(CARRIED_WITH_DAMAGE, 1)
        before = marked("data get entity @s SelectedItem", CARRIED)
        print(f"  {name}: the held stack before the click: {before.strip()}")
        click(bx, y, BLOCK_Z)
        line = info(bx, y, BLOCK_Z)
        print(f"  {name}: the bowl's block entity reads -> {line.strip()}")
        if f"stored={CARRIED} x1" not in line:
            raise SystemExit(
                f"BOWL FAIL: {name} does not hold {CARRIED} after the click: {line}"
            )
        stored_nbt = marked(f"data get block {bx} {y} {BLOCK_Z}", "Items:")
        print(f"  {name}: the block entity's NBT carries the component: {stored_nbt.strip()}")
        if CARRIED in hand_counts():
            raise SystemExit(
                f"BOWL FAIL: the sword is still in the hand after it went into {name} "
                f"({inventory()})"
            )
        # The empty-hand click returns exactly that stack.
        click(bx, y, BLOCK_Z)
        line = info(bx, y, BLOCK_Z)
        print(f"  {name}: after the take-out click -> {line.strip()}")
        if "stored=empty" not in line:
            raise SystemExit(f"BOWL FAIL: {name} still holds something: {line}")
        returned = marked("data get entity @s SelectedItem", CARRIED)
        print(f"  {name}: the returned stack reads {returned.strip()}")
        if selftest:
            raise SystemExit(
                f"BOWL FAIL: bowl selftest: {CARRIED} returned to the hand "
                "(this run must fail; the take-out click must give it back)"
            )
        if "damage" not in returned:
            raise SystemExit(
                f"BOWL FAIL: {name} returned the stack without its component: {returned}"
            )
        if hand_counts().get(CARRIED) != 1:
            raise SystemExit(
                f"BOWL FAIL: {name} did not return exactly one {CARRIED} "
                f"({inventory()})"
            )

    # ---- the jar's slot limit: one goes in, the rest stays in the hand -----
    print("== the slot limit: five in the hand, one in the bowl ==")
    run("command", "clear @s")
    wait(2)
    place_bowl(226, y, BLOCK_Z, "sacrificial_bowl")
    run("give", "minecraft:dirt", "5")
    wait(2)
    select_hotbar("minecraft:dirt", 5)
    click(226, y, BLOCK_Z)
    line = info(226, y, BLOCK_Z)
    print(f"  the bowl holds -> {line.strip()}")
    if "stored=minecraft:dirt x1" not in line:
        raise SystemExit(f"BOWL FAIL: the bowl did not take exactly one dirt: {line}")
    if hand_counts().get("minecraft:dirt") != 4:
        raise SystemExit(
            f"BOWL FAIL: the hand kept {hand_counts().get('minecraft:dirt')} dirt, "
            f"expected 4 ({inventory()})"
        )
    print("  the hand kept four dirt: the jar's slot limit of one")

    # ---- breaking a full bowl drops the bowl and what it held --------------
    print("== breaking a full bowl: the bowl and its stack ==")
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    place_bowl(226, y, BLOCK_Z, "sacrificial_bowl")
    run("command", "clear @s")
    wait(2)
    run("give", "minecraft:apple", "1")
    wait(2)
    select_hotbar("minecraft:apple", 1)
    click(226, y, BLOCK_Z)
    line = info(226, y, BLOCK_Z)
    if "stored=minecraft:apple x1" not in line:
        raise SystemExit(f"BOWL FAIL: the bowl did not take the apple: {line}")
    run("command", "clear @s")
    wait(2)
    stand(226, y, BLOCK_Z + 2)
    run("look-at-block", "226", str(y), str(BLOCK_Z), "up")
    wait(2)
    reply = run("break", "226", str(y), str(BLOCK_Z))
    print(
        "  the break: "
        f"{json.dumps({key: reply.get(key) for key in ('block_before', 'block', 'drops')})}"
    )
    drops = reply.get("drops") or {}
    # The reply's `drops` is what the break produced — the loot table's bowl **and**
    # the container's spill (the engine's `Containers.dropContents` path), which is
    # the assertion. The item entities are not read: the apple can be picked up
    # before the next `ctl entities` (the 0.2.4 phase-L finding).
    if drops.get("occultism:sacrificial_bowl") != 1:
        raise SystemExit(
            f"BOWL FAIL: the break dropped {drops}, expected one sacrificial_bowl"
        )
    if drops.get("minecraft:apple") != 1:
        raise SystemExit(
            f"BOWL FAIL: breaking the full bowl did not spill its apple: {drops}"
        )
    print(f"  the bowl and its held stack both dropped: {json.dumps(drops)}")

    # ---- the conversion over spirit fire, and the golden differential ------
    print("== the conversion: a feather over spirit fire, plain vs golden ==")
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    for name, expect in (
        ("sacrificial_bowl", "occultism:awakened_feather"),
        ("golden_sacrificial_bowl", "minecraft:feather"),
    ):
        run("command", "clear @s")
        wait(2)
        place_bowl(230, y + 1, BLOCK_Z, name)
        set_block(230, y, BLOCK_Z, "occultism:spirit_fire")
        run("give", "minecraft:feather", "1")
        wait(2)
        select_hotbar("minecraft:feather", 1)
        click(230, y + 1, BLOCK_Z, stand_y=y)
        line = info(230, y + 1, BLOCK_Z)
        print(f"  {name} over spirit fire -> {line.strip()}")
        if f"stored={expect} x1" not in line:
            raise SystemExit(
                f"BOWL FAIL: {name} over spirit fire stored {line!r}, "
                f"expected {expect} x1"
            )
    set_block(230, y + 1, BLOCK_Z, "minecraft:air")
    set_block(230, y, BLOCK_Z, "minecraft:air")

    # ---- a sneaking click changes nothing ----------------------------------
    print("== a sneaking click changes nothing ==")
    run("command", "clear @s")
    wait(2)
    place_bowl(226, y, BLOCK_Z, "sacrificial_bowl")
    run("give", "minecraft:apple", "1")
    wait(2)
    select_hotbar("minecraft:apple", 1)
    run("key", "shift", "down")
    wait(2)
    click(226, y, BLOCK_Z)
    wait(3)
    run("key", "shift", "up")
    wait(2)
    line = info(226, y, BLOCK_Z)
    print(f"  after the sneaking click -> {line.strip()}")
    if "stored=empty" not in line:
        raise SystemExit(f"BOWL FAIL: a sneaking click filled the bowl: {line}")
    if hand_counts().get("minecraft:apple") != 1:
        raise SystemExit(
            f"BOWL FAIL: a sneaking click consumed the apple ({inventory()})"
        )

    # ---- the tool pair: hand 1, pickaxe 1, otherstone's hand 0 --------------
    def break_at(bx: int, by: int, bz: int) -> dict:
        stand(bx, by, bz + 2)
        run("look-at-block", str(bx), str(by), str(bz), "up")
        wait(2)
        code, out, err = raw("break", str(bx), str(by), str(bz))
        if code != 0:
            print(f"    break at {bx},{by},{bz} failed ({out or err}); re-aiming", file=sys.stderr)
            run("look-at-block", str(bx), str(by), str(bz), "up")
            wait(3)
            code, out, err = raw("break", str(bx), str(by), str(bz))
            if code != 0:
                raise SystemExit(f"BOWL FAIL: cannot break {bx},{by},{bz}: {out}{err}")
        try:
            return json.loads(out)
        except json.JSONDecodeError:
            raise SystemExit(f"BOWL FAIL: ctl break at {bx},{by},{bz}: {out}{err}")

    def drops(reply: dict) -> dict[str, int]:
        return reply.get("drops") or {}

    print("== the six bowls: hand 1, wooden pickaxe 1 ==")
    for index, name in enumerate(BOWLS):
        bx = 226 + index * 2
        block = f"occultism:{name}"
        for label, held in (("by hand", None), ("with a wooden pickaxe", "minecraft:wooden_pickaxe")):
            # A leftover entity from the previous leg merges into the next break's
            # reply, so the world's items are cleared first.
            run("command", "kill @e[type=minecraft:item]")
            wait(2)
            set_block(bx, y, BLOCK_Z, block)
            if held is None:
                run("command", "clear @s")
            else:
                select_hotbar(held, 1)
            wait(4)
            reply = break_at(bx, y, BLOCK_Z)
            print(
                f"  {block} {label}: "
                f"{json.dumps({key: reply.get(key) for key in ('block_before', 'block', 'drops')})}"
            )
            if reply.get("block") != "minecraft:air":
                raise SystemExit(
                    f"BOWL FAIL: the {label} break left {reply.get('block')} at x={bx}"
                )
            if drops(reply).get(block) != 1:
                raise SystemExit(
                    f"BOWL FAIL: the {label} break of {block} dropped {drops(reply)}, "
                    "expected exactly one (requires_tool is false)"
                )

    # ---- the differential that makes the hand leg fail-capable ------------
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    set_block(226, y, BLOCK_Z, "occultism:otherstone")
    run("command", "clear @s")
    wait(4)
    reply = break_at(226, y, BLOCK_Z)
    print(f"  differential, otherstone by hand: {json.dumps(drops(reply))}")
    if drops(reply):
        raise SystemExit(
            f"BOWL FAIL: a hand break of otherstone dropped {drops(reply)}; "
            "the differential that makes the bowls' hand-1 result meaningful is broken"
        )

    run("command", "clear @s")
    run("command", "kill @e[type=minecraft:item]")
    wait(2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
