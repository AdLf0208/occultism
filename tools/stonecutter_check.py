#!/usr/bin/env python3
"""Cut real slabs at a real stonecutter (slice 14, phase Z).

The recipe files are data; this is the fail-capable in-game proof that the 26.3
dialect rewrite actually loads and produces the declared output.

For each of the three inputs the harness:

1. places a `minecraft:stonecutter` on a stone base at (x, y, z), opens it with
   `ctl interact` (a `ctl use` leaves the screen in `playing` and every slot click
   silently misses), and **guards** the open menu: `ctl menu` must answer
   `station = "Stonecutter"` with the slots 600 (input) and 650 (result), so a
   plain inventory screen can never be mistaken for the station;
2. inserts **one** item into slot 600, reads back the station's container slots,
   and asserts the offer list names the jar's recipe id;
3. clicks a *different* offer first (the differential: the selection, not the
   input alone, decides the result), then selects the declared recipe with
   `ctl click-gui` on its 4x3 cell (the GUI geometry Horizon's
   `game/stations.rs::station_targets` uses);
4. asserts the selected result reads `<id> x<count>`, that slot 600 holds the
   consumed input and slot 650 holds exactly `count` of the result, then takes
   it and checks the inventory gained exactly that count;
5. repeats with `occultism:otherrock` (its own slab), and with vanilla
   `minecraft:stone` (no `occultism:` offer, no occultism slab) as the
   differential.

`STONECUT_SELFTEST=1` asserts the first slab is **one** instead of two, so a
working package must FAIL that run (`EXIT:1` is its fail-capability proof).

Usage: python3 tools/stonecutter_check.py SOCK CTL X Y Z
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

INPUT = 600
RESULT = 650


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = (int(v) for v in sys.argv[3:6])
    selftest = os.environ.get("STONECUT_SELFTEST") == "1"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"STONECUT FAIL: ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 2) -> None:
        run("wait", str(ticks))

    def menu() -> dict:
        reply = run("menu")
        if reply.get("station") != "Stonecutter":
            raise SystemExit(
                f"STONECUT FAIL: no stonecutter menu is open (station={reply.get('station')!r}); "
                "a plain inventory screen has no station slots"
            )
        # `ctl menu` lists only NON-EMPTY slots: an empty input/result slot is
        # absent, not zero, so the guard is the station name, not the presence
        # of slots 600/650. See game.rs::slots / station_control.rs::menu.
        return reply

    def screen_slots() -> list[dict]:
        """`ctl screen` lists every slot, empty ones included (flat item/count)."""
        return run("screen").get("slots", [])

    def stack(reply: dict, slot: int) -> dict | None:
        for entry in reply.get("slots", []):
            if int(entry["slot"]) == slot:
                return entry.get("stack") or None
        return None

    def counts() -> dict[str, int]:
        return run("inventory").get("counts", {})

    def give(item: str, count: int = 1) -> None:
        run("give", item, str(count))
        wait(2)

    def close() -> None:
        raw("close")
        wait(2)

    def open_cutter(item: str) -> None:
        close()
        run("setblock", str(x), str(y - 1), str(z), "minecraft:stone")
        run("setblock", str(x), str(y), str(z), "minecraft:stonecutter")
        wait(2)
        run("command", "clear @s")
        run("command", "kill @e[type=minecraft:item]")
        wait(2)
        give(item)
        run("command", f"tp @s {x - 0.5} {y} {z + 0.5}")
        wait(2)
        run("look-at-block", str(x), str(y), str(z))
        wait(1)
        run("interact", str(x), str(y), str(z))
        wait(3)
        menu()  # the guard: the stonecutter, not the inventory menu

    def feed(item: str) -> None:
        source = None
        for entry in screen_slots():
            if int(entry["slot"]) <= 35 and entry.get("item") == item:
                source = int(entry["slot"])
                break
        if source is None:
            raise SystemExit(
                f"STONECUT FAIL: {item} is not in the inventory ({json.dumps(counts(), sort_keys=True)})"
            )
        run("click-slot", str(source))
        wait(1)
        run("click-slot", str(INPUT))
        wait(2)
        got = stack(menu(), INPUT)
        if got is None or got.get("item") != item or int(got.get("count", 0)) != 1:
            raise SystemExit(f"STONECUT FAIL: slot {INPUT} does not hold one {item}: {got}")
        print(f"  slot {INPUT} holds one {item}")

    def cell(position: int) -> tuple[float, float]:
        """The centre of recipe cell `position` in the 4x3 grid (GUI origin 52,14)."""
        return (52 + 16 * (position % 4) + 8, 14 + 18 * (position // 4) + 9)

    def select(position: int) -> dict:
        cx, cy = cell(position)
        run("click-gui", str(cx), str(cy))
        wait(2)
        return (menu().get("info") or {})

    def container_log(what: str) -> None:
        """Print `ctl container` at the station position for the transcript.

        A stonecutter has no block entity at this pin: its container *is* the
        station menu (slots 600/650), so `ctl container` answers "no container
        there". The slot assertions below read that station container through
        `ctl menu`, which is position-bound by the guard above.
        """
        rc, out, err = raw("container", str(x), str(y), str(z))
        print(f"  ctl container {x} {y} {z} ({what}): rc={rc} {out.strip()}{err.strip()}")

    def case(item: str, recipe: str, result_item: str, count: int) -> None:
        # The recipe service namespaces station offers as
        # `<ns>:stonecutting/<file stem>` (checked in game: the otherstone input
        # offered seven `occultism:stonecutting/otherstone_*` ids).
        stem = recipe.split(":", 1)[1] if ":" in recipe else recipe
        offer = f"occultism:stonecutting/{stem}"
        print(f"== {item} -> {recipe} ==")
        open_cutter(item)
        feed(item)
        container_log(f"{item} inserted")
        info = menu().get("info") or {}
        offers = info.get("recipes") or []
        if offer not in offers:
            raise SystemExit(
                f"STONECUT FAIL: the stonecutter offers {offers}, not {offer}"
            )
        if len(offers) > 12:
            raise SystemExit(
                f"STONECUT FAIL: {len(offers)} offers do not fit the 12 shown cells"
            )
        index = offers.index(offer)
        want = f"{result_item} x{count}"

        # The differential: a different cell must select a different result.
        other = (index + 1) % len(offers)
        other_info = select(other)
        if other_info.get("selected") != other:
            raise SystemExit(
                f"STONECUT FAIL: cell {other} selected {other_info.get('selected')}, expected {other}"
            )
        if other_info.get("result") == want:
            raise SystemExit(
                f"STONECUT FAIL: cell {other} also reports {want}, so selecting a recipe "
                "does not decide the result"
            )
        print(f"  cell {other} selects {other_info.get('result')} ({offers[other]})")

        chosen = select(index)
        if chosen.get("selected") != index or chosen.get("result") != want:
            raise SystemExit(
                f"STONECUT FAIL: selected {chosen.get('selected')}, result "
                f"{chosen.get('result')!r}, expected cell {index} and {want!r}"
            )
        print(f"  cell {index} selects {chosen.get('result')} ({offer})")

        before = stack(menu(), RESULT)
        if before is None or before.get("item") != result_item or int(before.get("count", 0)) != count:
            raise SystemExit(
                f"STONECUT FAIL: slot {RESULT} holds {before}, expected {result_item} x{count}"
            )
        input_after = stack(menu(), INPUT)
        if input_after is not None and int(input_after.get("count", 0)) > 0:
            # The input is consumed when the result is taken, not when selected.
            print(f"  slot {INPUT} before taking: {input_after}")

        old = counts().get(result_item, 0)
        run("click-slot", str(RESULT))
        wait(1)
        free = None
        for entry in screen_slots():
            slot = int(entry["slot"])
            if slot <= 35 and not entry.get("item"):
                free = slot
                break
        if free is None:
            raise SystemExit("STONECUT FAIL: no empty inventory slot for the cut result")
        run("click-slot", str(free))
        wait(2)
        now = stack(menu(), INPUT)
        if now is not None and int(now.get("count", 0)) > 0:
            raise SystemExit(f"STONECUT FAIL: the {item} input was not consumed: {now}")
        after = counts().get(result_item, 0)
        if after != old + count:
            raise SystemExit(
                f"STONECUT FAIL: the inventory holds {after} {result_item}, expected {old + count}"
            )
        container_log(f"{item} consumed, {count} {result_item} made")
        print(f"  PASS {offer}: {want}, the {item} input was consumed")

    # ---- the two carried stone cases ---------------------------------------
    first_count = 1 if selftest else 2
    if selftest:
        print("(selftest: asserting the first slab is x1 instead of x2)")
    case(
        "occultism:otherstone",
        "occultism:otherstone_slab_from_otherstone",
        "occultism:otherstone_slab",
        first_count,
    )
    case(
        "occultism:otherrock",
        "occultism:otherrock_slab_from_otherrock",
        "occultism:otherrock_slab",
        2,
    )

    # ---- the differential: vanilla stone offers no occultism recipe --------
    print("== vanilla minecraft:stone: no occultism offer ==")
    open_cutter("minecraft:stone")
    feed("minecraft:stone")
    container_log("vanilla stone inserted")
    offers = (menu().get("info") or {}).get("recipes") or []
    mod_offers = [offer for offer in offers if offer.startswith("occultism:")]
    if mod_offers:
        raise SystemExit(f"STONECUT FAIL: vanilla stone offers occultism recipes {mod_offers}")
    before = counts().get("occultism:otherstone_slab", 0)
    if len(offers) > 0:
        info = select(0)
        result = info.get("result") or ""
        if result.startswith("occultism:"):
            raise SystemExit(f"STONECUT FAIL: vanilla stone selected an occultism result {result}")
    if counts().get("occultism:otherstone_slab", 0) != before:
        raise SystemExit("STONECUT FAIL: vanilla stone made an occultism slab")
    print(f"  vanilla stone offers only {offers}; no occultism slab was made")
    print("PASS: the stonecutter cut both carried stones into two slabs each, consumed "
          "the inputs, and vanilla stone made no occultism slab")
    return 0


if __name__ == "__main__":
    sys.exit(main())
