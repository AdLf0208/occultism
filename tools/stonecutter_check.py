#!/usr/bin/env python3
"""Cut all 32 carried recipes at a real stonecutter (slice 15, phase Z).

The recipe files are data; this is the fail-capable in-game proof that every one
of the 26.3-dialect stonecutting recipes actually loads and produces its declared
output. 0.2.15 proved only the two raw-stone slabs; the parity rule (AGENTS.md
2026-11-20) requires every carried recipe to be exercised.

The 32 ids are **enumerated from the ported files**, not from the brief: every
`data/occultism/recipe/stonecutting/*.json` is read, its bare-string ingredient
and `{"count", "id"}` result parsed, and the list asserted to be exactly 32.
The jar was checked to carry the same 32 stems (`stonecutting-carry-0.2.15.txt`).

For each recipe the harness:

1. places a `minecraft:stonecutter` on a stone base at (x, y, z), opens it with
   `ctl interact`, and **guards** the open menu: `ctl menu` must answer
   `station = "Stonecutter"`; `ctl menu` lists only non-empty slots, so an empty
   station slot 600/650 is absent and the guard is the station name, not slot
   presence. A stonecutter has no block entity here (`ctl container` answers
   "no container there"), so its container is the guarded menu's slots 600/650;
2. inserts **one** of the declared ingredient into slot 600 and asserts the
   offer list names `occultism:stonecutting/<file stem>`;
3. clicks a *different* offer first (the differential: the selection, not the
   input alone, decides the result), then selects the declared recipe with
   `ctl click-gui` on its 4x3 cell (the GUI geometry Horizon's
   `game/stations.rs::station_targets` uses);
4. asserts the selected result reads `<id> x<count>`, that slot 650 holds
   exactly `count` of the result, then takes it and asserts the input was
   consumed and the inventory gained exactly `count`;
5. prints one `PASS <offer>: <result> x<count> ...` line per recipe.

Finally vanilla `minecraft:stone` must offer no `occultism:` recipe and make no
occultism slab — the differential against an input with no carried recipes.

`SELFTEST=stonecut-all` (STONECUT_SELFTEST=stonecut-all) flips the bricks cut's
expected count from one to two, so a working package must FAIL that run
(`EXIT:1` is its fail-capability proof).

Usage: python3 tools/stonecutter_check.py SOCK CTL X Y Z
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

INPUT = 600
RESULT = 650
# A count flip on this cut (one brick per otherstone -> expect two) is the
# fail-capability probe for the whole sweep.
SELFTEST_STEM = "otherstone_bricks_from_otherstone"
DATA_DIR = Path(__file__).resolve().parent.parent / "data" / "occultism" / "recipe" / "stonecutting"


def enumerate_recipes() -> list[dict]:
    """Read every carried stonecutting recipe; the id list is the artefact's."""
    if not DATA_DIR.is_dir():
        raise SystemExit(f"STONECUT FAIL: no recipe directory {DATA_DIR}")
    recipes: list[dict] = []
    for path in sorted(DATA_DIR.glob("*.json")):
        data = json.loads(path.read_text())
        if data.get("type") != "minecraft:stonecutting":
            raise SystemExit(f"STONECUT FAIL: {path.name} is not a stonecutting recipe")
        ingredient = data.get("ingredient")
        result = data.get("result") or {}
        if not isinstance(ingredient, str) or not ingredient.startswith("occultism:"):
            raise SystemExit(f"STONECUT FAIL: {path.name} has no bare-string occultism ingredient: {ingredient}")
        if "id" not in result or "count" not in result:
            raise SystemExit(f"STONECUT FAIL: {path.name} has no {{count,id}} result: {result}")
        recipes.append({
            "stem": path.stem,
            "offer": f"occultism:stonecutting/{path.stem}",
            "ingredient": ingredient,
            "result": result["id"],
            "count": int(result["count"]),
        })
    if len(recipes) != 32:
        raise SystemExit(f"STONECUT FAIL: enumerated {len(recipes)} recipes, expected 32")
    return recipes


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = (int(v) for v in sys.argv[3:6])
    selftest = os.environ.get("STONECUT_SELFTEST", "0") not in ("", "0")
    recipes = enumerate_recipes()
    print(f"enumerated {len(recipes)} stonecutting recipes from {DATA_DIR}")
    if selftest:
        print(f"(selftest: asserting {SELFTEST_STEM} yields x2 instead of x1)")

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

    def case(recipe: dict) -> None:
        item = recipe["ingredient"]
        offer = recipe["offer"]
        result_item = recipe["result"]
        count = recipe["count"]
        # The inverted assertion for this whole sweep: flip the bricks cut.
        expected = count + 1 if (selftest and recipe["stem"] == SELFTEST_STEM) else count
        print(f"== {item} -> {offer} ({len(recipes)} recipes) ==")
        open_cutter(item)
        feed(item)
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
        want = f"{result_item} x{expected}"

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
        print(f"PASS {offer}: {result_item} x{count}, the {item} input was consumed")

    # ---- all 32 carried recipes --------------------------------------------
    for recipe in recipes:
        case(recipe)

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
    print(f"PASS: all {len(recipes)} carried stonecutting recipes cut at a real stonecutter "
          "(output id and count asserted, input consumed), a different selected cell yields a "
          "different result, and vanilla stone offers no occultism recipe")
    return 0


if __name__ == "__main__":
    sys.exit(main())
