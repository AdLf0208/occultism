#!/usr/bin/env python3
"""Carry the jar's stonecutting family into the 26.3 recipe dialect (slice 14).

The jar (Occultism 1.225.0, `release/v1.21.1-1.225.0`) has **32** files under
`data/occultism/recipe/stonecutting/` — FINDINGS.md's measured inventory agrees;
the 0.2.14 note's "31" undercounted the raw-stone family. All 32 ingredients and
results are already carried items, so this is pure data with no new code path.

The 1.21.1 ingredient is an object `{"item": "<id>"}`. The 26.3 loader drops
that shape without a word; a vanilla 26.3 file
(`data/minecraft/recipe/andesite_slab_from_andesite_stonecutting.json`) reads:

    {"type": "minecraft:stonecutting",
     "ingredient": "minecraft:andesite",
     "result": {"count": 2, "id": "minecraft:andesite_slab"}}

so the carry replaces the object with its bare id string and leaves the result
(the jar's `{"count", "id"}`) untouched. No other key is added or removed.

    python3 tools/stonecutting_carry.py            # write the 32 files
    python3 tools/stonecutting_carry.py --check    # verify the tree matches the jar
"""
from __future__ import annotations

import argparse
import json
import zipfile
from pathlib import Path

PORT = Path(__file__).resolve().parents[1]
JAR = PORT.parent / "_shared/licence-cache/occultism-1.21.1-neoforge-1.225.0.jar"
PREFIX = "data/occultism/recipe/stonecutting/"
RESULT_PREFIX = "data/occultism/recipe/stonecutting"


def jar_recipes() -> dict[str, dict]:
    out: dict[str, dict] = {}
    with zipfile.ZipFile(JAR) as zf:
        for name in sorted(zf.namelist()):
            if not (name.startswith(PREFIX) and name.endswith(".json")):
                continue
            record = json.loads(zf.read(name))
            if record.get("type") != "minecraft:stonecutting":
                raise SystemExit(f"CARRY FAIL: {name} is not a stonecutting recipe")
            ingredient = record.get("ingredient")
            if not isinstance(ingredient, dict) or "item" not in ingredient:
                raise SystemExit(f"CARRY FAIL: {name} has no 1.21.1 object-form ingredient")
            record["ingredient"] = ingredient["item"]  # the 26.3 bare-string dialect
            out[Path(name).stem] = record
    return out


def rendered(record: dict) -> str:
    return json.dumps(record, indent=2) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true",
                        help="verify the carried files, do not write")
    args = parser.parse_args()

    recipes = jar_recipes()
    if len(recipes) != 32:
        raise SystemExit(f"CARRY FAIL: expected 32 jar recipes, read {len(recipes)}")
    target = PORT / RESULT_PREFIX
    failures: list[str] = []
    if args.check:
        present = {p.stem for p in target.glob("*.json")}
        for stem in sorted(present - recipes.keys()):
            failures.append(f"{stem}: extra file, the jar has no such recipe")
        for stem in sorted(recipes.keys() - present):
            failures.append(f"{stem}: missing file")
    for stem, record in sorted(recipes.items()):
        path = target / f"{stem}.json"
        want = rendered(record)
        if args.check:
            got = path.read_text() if path.is_file() else ""
            if got != want:
                failures.append(f"{stem}: carried bytes differ from the jar rewrite")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(want)
            print(f"{stem}: {record['ingredient']} -> {record['result']['id']} x{record['result']['count']}")
    if failures:
        for failure in failures:
            print(f"CARRY FAIL: {failure}")
        return 1
    print(f"{'checked' if args.check else 'carried'} {len(recipes)} stonecutting recipes "
          f"in the 26.3 dialect")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
