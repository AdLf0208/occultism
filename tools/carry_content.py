#!/usr/bin/env python3
"""Build `content/` for the Occultism spirit-fire slice from the exact ATM10 jar.

Everything here is upstream's own MIT data, re-spelled only where Minecraft 26.3
differs from 1.21.1 (loot conditions are singular and typed) or where the slice
is narrower than the mod. Every skip is written to `carry-report.json` with the
reason, and `--check` re-runs the carry into a temp tree and compares bytes.

    python3 tools/carry_content.py            # write content/
    python3 tools/carry_content.py --check    # verify content/ matches the jar

Read from `occultism-1.21.1-neoforge-1.225.0.jar` (tag release/v1.21.1-1.225.0).
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
JAR = ROOT.parent / "_shared/licence-cache/occultism-1.21.1-neoforge-1.225.0.jar"
JAR_SHA256 = "fc222df07edf266b38c538111522799165c1647114e8c37974a975e4a80bf514"
NS = "occultism"

# The 27 `spirit_fire` recipes' results: 27 distinct items, 4 of them blocks.
PLAIN_ITEMS = [
    "awakened_feather",
    "book_of_binding_empty",
    "chalk_white",
    "chalk_light_gray",
    "chalk_gray",
    "chalk_black",
    "chalk_brown",
    "chalk_red",
    "chalk_orange",
    "chalk_gold",
    "chalk_lime",
    "chalk_green",
    "chalk_cyan",
    "chalk_light_blue",
    "chalk_blue",
    "chalk_purple",
    "chalk_magenta",
    "chalk_pink",
    "otherworld_ashes",
    "otherworld_essence",
    "purified_ink",
    "spirit_attuned_gem",
    "taboo_book",
    # the loop's own inputs
    "datura",
    "demons_dream_essence",
    "chalk_white_impure",
    "chalk_light_gray_impure",
    "chalk_gray_impure",
    "chalk_black_impure",
    "chalk_brown_impure",
    "chalk_red_impure",
    "chalk_orange_impure",
    "chalk_yellow_impure",
    "chalk_lime_impure",
    "chalk_green_impure",
    "chalk_cyan_impure",
    "chalk_light_blue_impure",
    "chalk_blue_impure",
    "chalk_purple_impure",
    "chalk_magenta_impure",
    "chalk_pink_impure",
]

# The four block results, with the template whose behaviour class and states
# they take and the properties read from upstream's registration.
BLOCKS = {
    "otherstone": {
        "template": "minecraft:stone",
        "properties": {
            "hardness": 1.5,
            "explosion_resistance": 30.0,
            "requires_tool": True,
            "map_color": 11,
            "sounds": "minecraft:stone",
        },
        "shapes": {"default": [[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]},
    },
    "otherrock": {
        "template": "minecraft:stone",
        "properties": {
            "hardness": 1.5,
            "explosion_resistance": 30.0,
            "requires_tool": True,
            "map_color": 11,
            "sounds": "minecraft:stone",
        },
        "shapes": {"default": [[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]},
    },
    "otherflower": {
        "template": "minecraft:poppy",
        "properties": {
            "hardness": 0.0,
            "explosion_resistance": 0.0,
            "requires_tool": False,
            "map_color": 7,
            "sounds": "minecraft:short_grass",
            "occludes": False,
        },
        "shapes": {"default": [[0.3125, 0.0, 0.3125, 0.6875, 0.625, 0.6875]]},
        "collision_shapes": {"default": []},
    },
    "otherworld_sapling_natural": {
        # Upstream grows the Otherworld tree from this sapling; the tree feature
        # is worldgen and out of this slice, so the block takes `short_grass`
        # (a vegetation class that never grows) instead of a sapling.
        "template": "minecraft:short_grass",
        "properties": {
            "hardness": 0.0,
            "explosion_resistance": 0.0,
            "requires_tool": False,
            "map_color": 7,
            "sounds": "minecraft:short_grass",
            "occludes": False,
        },
        "shapes": {"default": [[0.125, 0.0, 0.125, 0.875, 0.75, 0.875]]},
        "collision_shapes": {"default": []},
        "state_properties": {"uncovered": {"type": "boolean"}},
        "default_state": {"uncovered": "false"},
    },
}

SPIRIT_FIRE_COLORS = [
    "white",
    "light_gray",
    "gray",
    "black",
    "brown",
    "red",
    "orange",
    "yellow",
    "lime",
    "green",
    "cyan",
    "light_blue",
    "blue",
    "purple",
    "magenta",
    "pink",
    "rainbow",
    "void",
]

SPIRIT_FIRE = {
    "template": "minecraft:torch",
    "item": False,
    "properties": {
        "hardness": 0.0,
        "explosion_resistance": 0.0,
        "requires_tool": False,
        "light": 12,
        "map_color": 24,
        "sounds": "minecraft:white_wool",
        "occludes": False,
    },
    "state_properties": {"color": {"type": "enum", "values": SPIRIT_FIRE_COLORS}},
    "default_state": {"color": "white"},
    "shapes": {"default": [[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]},
    "collision_shapes": {"default": []},
}

# Block models the carried blockstates name, beyond the block's own.
EXTRA_BLOCK_MODELS = {
    "spirit_fire": [
        "spirit_fire",
        "spirit_fire_color",
        "spirit_fire_rainbow",
        "spirit_fire_void",
    ],
    "otherworld_sapling_natural": ["otherworld_sapling_natural", "otherworld_sapling"],
}

# The common tags the loop's ingredients name, as the conventions spell them.
C_TAGS = {
    "feathers": ["minecraft:feather"],
    "dyes/black": ["minecraft:black_dye"],
    "gems/diamond": ["minecraft:diamond"],
}

MOD_TOML = """\
# Port-owned content mod for the Occultism 1.225.0 slice: the spirit-fire block,
# the 45 items the conversion loop reads and writes, and the carried assets/data.
# The block models, textures, blockstates, recipes and lang strings are
# upstream's own MIT assets, read from the exact ATM10 jar
# (see ../../_shared/carry-work/occultism/SOURCE.md). The 4 loot tables and the
# 3 spirit-fire face tags are re-spelled for 26.3 (singular typed conditions) or
# written for the slice (self-drop).
id = "occultism"
name = "Occultism"
version = "0.1.0"
api = "0.2"
side = "both"
capabilities = ["registries"]
"""


class Jar:
    def __init__(self, path: Path):
        self.zip = zipfile.ZipFile(path)
        self.names = set(self.zip.namelist())

    def read(self, name: str) -> bytes | None:
        if name not in self.names:
            return None
        return self.zip.read(name)

    def json(self, name: str):
        data = self.read(name)
        return None if data is None else json.loads(data)


def write(path: Path, data: bytes | str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, str):
        path.write_text(data)
    else:
        path.write_bytes(data)


def dump(value) -> str:
    return json.dumps(value, indent=2, sort_keys=True) + "\n"


def texture_refs(model: dict) -> list[str]:
    return [v for v in (model.get("textures") or {}).values() if isinstance(v, str)]


def carry(jar: Jar, out: Path) -> dict:
    report: dict = {"source": JAR.name, "jar_sha256": JAR_SHA256,
                    "blocks": [], "items": [], "skipped": []}

    # --- blocks and their models, blockstates and textures -------------------
    block_defs = dict(BLOCKS)
    block_defs["spirit_fire"] = SPIRIT_FIRE
    for name, block in block_defs.items():
        write(out / f"data/{NS}/horizon/block/{name}.json", dump(block))
        report["blocks"].append(name)
        state = jar.read(f"assets/{NS}/blockstates/{name}.json")
        if state is None:
            raise SystemExit(f"missing blockstate for {name}")
        write(out / f"assets/{NS}/blockstates/{name}.json", state)
        models = [name, *EXTRA_BLOCK_MODELS.get(name, [])]
        for model in dict.fromkeys(models):
            data = jar.json(f"assets/{NS}/models/block/{model}.json")
            if data is None:
                raise SystemExit(f"missing block model {model}")
            write(out / f"assets/{NS}/models/block/{model}.json", dump(data))
            for texture in texture_refs(data):
                carry_texture(jar, out, texture, report)
        item_model = jar.read(f"assets/{NS}/models/item/{name}.json")
        if item_model is None:
            raise SystemExit(f"missing item model for block {name}")
        write(out / f"assets/{NS}/models/item/{name}.json", item_model)
        for texture in texture_refs(json.loads(item_model)):
            carry_texture(jar, out, texture, report)

    # --- plain items ---------------------------------------------------------
    for name in PLAIN_ITEMS:
        model = jar.json(f"assets/{NS}/models/item/{name}.json")
        if model is None:
            raise SystemExit(f"missing item model {name}")
        write(out / f"data/{NS}/horizon/item/{name}.json",
              dump({"template": "minecraft:paper"}))
        write(out / f"assets/{NS}/models/item/{name}.json", dump(model))
        for texture in texture_refs(model):
            carry_texture(jar, out, texture, report)
        report["items"].append(name)

    # --- recipes ------------------------------------------------------------
    recipes = sorted(n for n in jar.names
                     if n.startswith(f"data/{NS}/recipe/spirit_fire/") and n.endswith(".json"))
    for name in recipes:
        write(out / name, jar.read(name))
    report["recipes"] = [Path(n).name for n in recipes]

    # --- tags ---------------------------------------------------------------
    start = jar.json(f"data/{NS}/tags/item/start_spiritfire.json")
    write(out / f"data/{NS}/tags/item/start_spiritfire.json", dump(start))

    # The 1.21.1 file names book items this slice does not carry; a tag entry the
    # game cannot resolve is dropped, as vanilla drops an unknown tag member.
    books = jar.json(f"data/{NS}/tags/item/books/books_for_empty.json")
    known = set(PLAIN_ITEMS) | set(BLOCKS) | set(PLAIN_ITEMS)
    kept, dropped = [], []
    for value in books["values"]:
        namespace, _, path = value.partition(":")
        if namespace == "minecraft" or path in known:
            kept.append(value)
        else:
            dropped.append(value)
    write(out / f"data/{NS}/tags/item/books/books_for_empty.json",
          dump({"values": kept}))
    report["skipped"].append({
        "what": "data/occultism/tags/item/books/books_for_empty.json members",
        "dropped": dropped,
        "why": "the book-of-binding and book-of-calling items belong to the ritual "
               "slice; the vanilla books stay, so the recipe still matches",
    })

    for tag, values in C_TAGS.items():
        write(out / f"data/c/tags/item/{tag}.json", dump({"values": values}))

    write(out / "data/minecraft/tags/item/small_flowers.json",
          dump({"values": [f"{NS}:otherflower"]}))
    write(out / "data/minecraft/tags/block/mineable/pickaxe.json",
          dump({"values": [f"{NS}:otherstone", f"{NS}:otherrock"]}))
    report["skipped"].append({
        "what": "data/occultism/tags/item/otherworld_logs.json",
        "why": "its only members are Otherworld logs, which are the Otherworld "
               "dimension's wood (worldgen); the spirit_fire/otherworld_ashes "
               "recipe therefore cannot match and is reported as skipped",
    })

    # --- loot tables (26.3: singular, typed condition) -----------------------
    for name in ("otherstone", "otherrock", "otherflower", "otherworld_sapling_natural"):
        write(out / f"data/{NS}/loot_table/blocks/{name}.json", dump({
            "type": "minecraft:block",
            "pools": [{
                "bonus_rolls": 0.0,
                "rolls": 1.0,
                "condition": {"type": "minecraft:survives_explosion"},
                "entries": [{"type": "minecraft:item", "name": f"{NS}:{name}"}],
            }],
            "random_sequence": f"{NS}:blocks/{name}",
        }))
    report["skipped"].append({
        "what": "loot_table/blocks/otherstone.json, otherrock.json",
        "why": "upstream drops othercobblestone/othercobblerock without silk touch; "
               "the cobblestone family is not in this slice, so the block drops itself",
    })

    # --- lang and the creative tab ------------------------------------------
    lang = jar.json(f"assets/{NS}/lang/en_us.json")
    keys = {}
    for name in block_defs:
        for prefix in ("block", "item"):
            key = f"{prefix}.{NS}.{name}"
            if key in lang:
                keys[key] = lang[key]
    for name in PLAIN_ITEMS:
        key = f"item.{NS}.{name}"
        if key not in lang:
            raise SystemExit(f"missing lang key {key}")
        keys[key] = lang[key]
    write(out / f"assets/{NS}/lang/en_us.json", dump(keys))

    write(out / f"data/{NS}/horizon/creative_tab/spirit_fire.json", dump({
        "title": "itemGroup.occultism",
        "icon": f"{NS}:spirit_attuned_gem",
        "items": [f"{NS}:{name}" for name in [
            "spirit_attuned_gem", "datura", "demons_dream_essence",
            "purified_ink", "taboo_book", "awakened_feather",
            "book_of_binding_empty", "otherstone", "otherrock", "otherflower",
            "otherworld_sapling_natural", "otherworld_ashes", "otherworld_essence",
            *[n for n in PLAIN_ITEMS if n.startswith("chalk_") and not n.endswith("_impure")],
            *[n for n in PLAIN_ITEMS if n.endswith("_impure") and n != "demons_dream_essence"],
        ]],
    }))
    keys["itemGroup.occultism"] = "Occultism"
    write(out / f"assets/{NS}/lang/en_us.json", dump(keys))

    write(out / "horizon-mod.toml", MOD_TOML)
    report["counts"] = {
        "blocks": len(block_defs),
        "items": len(PLAIN_ITEMS),
        "recipes": len(recipes),
        "files": sum(1 for p in out.rglob("*")
                     if p.is_file()
                     and p.name != "carry-report.json"
                     and "target" not in p.relative_to(out).parts),
    }
    write(out / "carry-report.json", dump(report))
    return report


def carry_texture(jar: Jar, out: Path, reference: str, report: dict) -> None:
    namespace, _, path = reference.partition(":")
    if namespace != NS:
        return  # a vanilla texture: the game already has it
    for suffix in ("", ".mcmeta"):
        name = f"assets/{NS}/textures/{path}.png{suffix}"
        data = jar.read(name)
        if data is None:
            if suffix == "":
                raise SystemExit(f"missing texture {name}")
            continue
        write(out / name, data)


def tree(root: Path) -> dict[str, str]:
    # `content/target/` is the packaging tool's build output, not carried data.
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob("*"))
            if p.is_file() and "target" not in p.relative_to(root).parts}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="carry into a temp tree and compare with content/")
    args = parser.parse_args()
    if not JAR.is_file():
        print(f"missing jar: {JAR}", file=sys.stderr)
        return 1
    digest = hashlib.sha256(JAR.read_bytes()).hexdigest()
    if digest != JAR_SHA256:
        print(f"jar hashes differ: {digest}", file=sys.stderr)
        return 1
    jar = Jar(JAR)
    if args.check:
        with tempfile.TemporaryDirectory() as temp:
            temp_path = Path(temp)
            carry(jar, temp_path)
            built = tree(temp_path)
            on_disk = tree(ROOT / "content")
            missing = sorted(set(built) - set(on_disk))
            extra = sorted(set(on_disk) - set(built))
            changed = sorted(k for k in set(built) & set(on_disk) if built[k] != on_disk[k])
            if missing or extra or changed:
                print(json.dumps({"missing": missing, "extra": extra, "changed": changed},
                                 indent=2), file=sys.stderr)
                return 1
            print(f"content/ matches the jar: {len(built)} files")
        return 0
    out = ROOT / "content"
    for stale in ("data", "assets"):
        shutil.rmtree(out / stale, ignore_errors=True)
    report = carry(jar, out)
    print(json.dumps(report["counts"], indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
