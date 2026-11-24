#!/usr/bin/env python3
"""Prove every carryable smelting row in a real furnace (phase AB).

Called by `tools/e2e.sh` with the open game's control socket and a position in
the End arena. The jar ships 19 `data/occultism/recipe/smelting/*.json`; ten are
carryable (their ingredient and result both exist in this package or vanilla)
and are packed under `data/occultism/recipe/smelting/`. This driver reads those
files, so the row set, the ingredient, the result, the count, the experience and
the 200-tick time come from the artefact, never from this script's prose.

For each row: place a fresh furnace, put one declared input in slot 0 and one
coal in slot 1, wait 240 ticks, then read the container back. The output slot's
id and count, the consumed input and the declared XP are printed on one line;
the XP is parsed from the recipe JSON because `ctl container` reports slots only
(no stored-XP field at this pin) — the value itself is pinned by the unit test
`the_smelting_family_covers_every_carryable_jar_recipe`. A `#tag` ingredient is
resolved through the package's own tag file to a carried member item.

Differentials: a fuel-less furnace cooks nothing and keeps its input, and a
wrong input (dirt) cooks nothing — both on the two dust → vanilla-ingot rows
this slice adds. `SELFTEST=smelt` flips one output count, so a working package
must FAIL that run.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

PORT = Path(__file__).resolve().parents[1]
SMELT_DIR = PORT / "data/occultism/recipe/smelting"

# The ten carryable rows, with the jar's declared ingredient / result / count /
# experience / cooking time. The nine the jar ships that are NOT here are
# recorded as blocked(<gap>) in notes.md; none is invented here.
EXPECTED: dict[str, tuple[str, str, int, float, int]] = {
    "burnt_otherstone": ("occultism:polished_otherstone", "occultism:burnt_otherstone", 1, 0.15, 200),
    "burnt_otherrock": ("occultism:polished_otherrock", "occultism:burnt_otherrock", 1, 0.15, 200),
    "cracked_otherstone_bricks": ("occultism:otherstone_bricks", "occultism:cracked_otherstone_bricks", 1, 0.3, 200),
    "cracked_otherrock_bricks": ("occultism:otherrock_bricks", "occultism:cracked_otherrock_bricks", 1, 0.3, 200),
    "otherstone": ("occultism:othercobblestone", "occultism:otherstone", 1, 0.5, 200),
    "otherrock": ("occultism:othercobblerock", "occultism:otherrock", 1, 0.5, 200),
    "polished_otherstone": ("occultism:otherstone", "occultism:polished_otherstone", 1, 0.5, 200),
    "polished_otherrock": ("occultism:otherrock", "occultism:polished_otherrock", 1, 0.5, 200),
    "gold_ingot_from_dust": ("#c:dusts/gold", "minecraft:gold_ingot", 1, 0.7, 200),
    "iron_ingot_from_dust": ("#c:dusts/iron", "minecraft:iron_ingot", 1, 0.7, 200),
}

FURNACE_X = 240
FURNACE_Z = 44


def fail(msg: str) -> None:
    print(f"SMELT FAIL: {msg}", flush=True)
    raise SystemExit(1)


def tag_member(tag: str) -> str:
    """Resolve a `#c:dusts/<x>` ingredient to its carried member item."""
    ns, _, path = tag.lstrip("#").partition(":")
    f = PORT / "data" / ns / "tags" / "item" / f"{path}.json"
    if not f.is_file():
        fail(f"the ingredient tag {tag} has no file at {f}")
    values = json.loads(f.read_text()).get("values", [])
    # Prefer a carried occultism item member over a vanilla one.
    for v in values:
        if isinstance(v, str) and v.startswith("occultism:"):
            return v
    for v in values:
        if isinstance(v, str) and v.startswith("minecraft:"):
            return v
    fail(f"the ingredient tag {tag} has no resolvable member: {values}")
    return ""


def read_rows() -> dict[str, dict]:
    files = sorted(SMELT_DIR.glob("*.json"))
    if len(files) != len(EXPECTED):
        fail(f"the package carries {len(files)} smelting rows, expected {len(EXPECTED)}")
    rows: dict[str, dict] = {}
    for f in files:
        stem = f.stem
        if stem not in EXPECTED:
            fail(f"{stem} is not one of the ten carryable smelting rows")
        doc = json.loads(f.read_text())
        want_ingredient, want_result, want_count, want_xp, want_time = EXPECTED[stem]
        if doc.get("type") != "minecraft:smelting":
            fail(f"{stem} is not a smelting recipe: {doc}")
        if doc.get("ingredient") != want_ingredient:
            fail(f"{stem}: ingredient {doc.get('ingredient')!r} != {want_ingredient!r}")
        if doc.get("cookingtime") != want_time:
            fail(f"{stem}: cookingtime {doc.get('cookingtime')} != {want_time}")
        result = doc.get("result") or {}
        if result.get("id") != want_result or result.get("count") != want_count:
            fail(f"{stem}: result {result} != {want_result} x{want_count}")
        if float(doc.get("experience", -1)) != want_xp:
            fail(f"{stem}: experience {doc.get('experience')} != {want_xp}")
        # The 26.3 dialect uses a bare string or `#tag`, not the jar's object.
        if isinstance(doc.get("ingredient"), dict):
            fail(f"{stem} carries the 1.21.1 object-form ingredient: {doc}")
        rows[stem] = doc
    return rows


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    y = 101
    selftest = os.environ.get("SMELT_SELFTEST") == "1" or os.environ.get("SELFTEST") == "smelt"

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            reply = json.loads(out.stdout)
        except json.JSONDecodeError:
            fail(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")
        if "error" in reply:
            fail(f"ctl {' '.join(args)}: {reply}")
        return reply

    rows = read_rows()
    print(f"  read {len(rows)} carryable smelting rows from the package", flush=True)

    def set_furnace() -> None:
        # Air first: a setblock of the same state keeps the old block entity.
        run("command", f"execute in minecraft:the_end run setblock {FURNACE_X} {y} {FURNACE_Z} minecraft:air")
        run("wait", "2")
        run("command", f"execute in minecraft:the_end run setblock {FURNACE_X} {y} {FURNACE_Z} minecraft:furnace")
        run("wait", "4")

    def add(item: str) -> None:
        run("container", str(FURNACE_X), str(y), str(FURNACE_Z), item, "1")

    def slots() -> list:
        return run("container", str(FURNACE_X), str(y), str(FURNACE_Z)).get("slots") or []

    def cook(input_item: str, fuel: bool) -> list:
        set_furnace()
        add(input_item)
        if fuel:
            add("minecraft:coal")
        run("wait", "240")
        return slots()

    names = sorted(rows)
    for i, name in enumerate(names):
        doc = rows[name]
        ingredient = doc["ingredient"]
        input_item = tag_member(ingredient) if ingredient.startswith("#") else ingredient
        result = doc["result"]["id"]
        count = int(doc["result"]["count"])
        xp = float(doc["experience"])
        got = cook(input_item, fuel=True)
        out = got[2] if len(got) > 2 else None
        want_count = count + 1 if (selftest and i == len(names) - 1) else count
        print(
            f"  SMELT {name}: input={input_item} fuel=coal -> "
            f"output={(out or {}).get('item')} x{(out or {}).get('count')} "
            f"xp={xp} cookingtime={doc['cookingtime']} (want {result} x{want_count})",
            flush=True,
        )
        if not out or out.get("item") != result:
            fail(f"{name} made {out}, expected {result}")
        if out.get("count") != want_count:
            fail(f"{name} made x{out.get('count')}, expected x{want_count}")
        if got[0] is not None:
            fail(f"{name} kept its input: {got[0]}")

    if selftest:
        # The flipped row already failed above; unreachable, but keep the guard.
        fail("selftest: a wrong output count was accepted")

    # The two new rows get their own fuel-less and wrong-input differentials.
    for name in ("gold_ingot_from_dust", "iron_ingot_from_dust"):
        ingredient = rows[name]["ingredient"]
        input_item = tag_member(ingredient)
        result = rows[name]["result"]["id"]

        got = cook(input_item, fuel=False)
        print(
            f"  SMELT {name} no fuel: input={(got[0] or {}).get('item') if got else None} "
            f"output={(got[2] or {}).get('item') if len(got) > 2 else None}",
            flush=True,
        )
        if not got or got[0] is None:
            fail(f"{name}: the fuel-less furnace lost its input: {got}")
        if len(got) > 2 and got[2] is not None:
            fail(f"{name}: a fuel-less furnace made {got[2]}")

        got = cook("minecraft:dirt", fuel=True)
        print(
            f"  SMELT {name} wrong input (dirt): "
            f"output={(got[2] or {}).get('item') if len(got) > 2 else None}",
            flush=True,
        )
        if len(got) > 2 and got[2] is not None:
            fail(f"{name}: a wrong input made {got[2]}")

    set_furnace()
    print(
        "PASS: all ten carryable smelting recipes cook in a real furnace at 200 ticks "
        "with their declared result count and experience; a fuel-less furnace and a "
        "wrong input make nothing",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
