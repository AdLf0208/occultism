#!/usr/bin/env python3
"""Check that every carried recipe's ingredients actually resolve.

Adapted from `ports/actuallyadditions/tools/check_recipe_keys.py` for this port,
because the campaign has no `ports/tools/check-recipe-keys.py` (the brief names
one; this is the tool it means, with this port's layout and its own recipe type).

A recipe is dead data when one of its keys names an item or a ``#tag`` no
registry has. Two measured shapes at this pin:

* a missing **tag** is fatal to the file — the loader logs ``recipes: invalid
  ingredient (unresolved tag or malformed input): "#c:cobblestones"`` and drops
  the whole recipe;
* an unknown **item id** leaves the recipe registered but that key can never be
  satisfied, so the recipe is in the book and never crafts.

Neither ``cargo horizon-mod check`` nor a green unit test notices either: both run
before the game resolves tags. This port reads its own ``occultism:spirit_fire``
recipes through the recipe service, so those files are skipped here (their inputs
are validated by ``runtime.rs``'s table tests and counted by `/occultism
recipes`); every crafting and smelting file is checked.

The rule this tool enforces:

* an ingredient resolves when it is a vanilla item (26.3's ``items.json``), an
  item or block item this package declares, or a ``#tag`` this package declares
  with at least one member;
* every declared ``tags/item`` file must be non-empty, because the loader drops
  a tag with no values;
* a recipe that *cannot* resolve yet must be named in ``tools/recipe-blockers.json``
  with the gap symbol it waits on, and a blocker that has become craftable is an
  error too — the record cannot rot into a lie.

    python3 tools/check_recipe_keys.py
    python3 tools/check_recipe_keys.py --resources /path/to/26.3   # offline root

Exit 0 when every recipe is either craftable or explicitly blocked.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

PORT = Path(__file__).resolve().parents[1]
MOD = "occultism"
DEFAULT_RESOURCES = Path(
    os.environ.get(
        "HORIZON_RES",
        "/home/adlf/Desktop/horizon/.local/horizon-client/26.3",
    )
)


def read_json(path: Path) -> object:
    return json.loads(path.read_text())


def declared_items(port: Path) -> dict[str, str]:
    """Item ids this package declares, mapped to where they come from.

    A block declaration creates its block item unless it says `"item": false`
    (`datura` is the one that does), so a block declaration is an item source.
    """
    out: dict[str, str] = {}
    for path in sorted((port / "data" / MOD / "horizon" / "item").glob("*.json")):
        out[f"{MOD}:{path.stem}"] = f"horizon/item/{path.name}"
    for path in sorted((port / "data" / MOD / "horizon" / "block").glob("*.json")):
        declaration = read_json(path)
        if isinstance(declaration, dict) and declaration.get("item") is not False:
            out[f"{MOD}:{path.stem}"] = f"horizon/block/{path.name}"
    return out


def declared_tags(port: Path) -> tuple[dict[str, list[str]], list[str]]:
    """Every `tags/item` file this package carries, and the empty ones."""
    tags: dict[str, list[str]] = {}
    empty: list[str] = []
    for path in sorted((port / "data").glob("*/tags/item/**/*.json")):
        data = read_json(path)
        values = data.get("values", []) if isinstance(data, dict) else []
        if not values:
            empty.append(str(path.relative_to(port)))
        relative = path.relative_to(port / "data").with_suffix("")
        # data/<ns>/tags/item/<path>.json is the tag <ns>:<path>
        parts = relative.parts
        namespace = parts[0]
        tag_path = "/".join(parts[3:])
        tags[f"{namespace}:{tag_path}"] = [str(v) for v in values]
    return tags, empty


def expand_tag(tag: str, tags: dict[str, list[str]], seen: set[str]) -> set[str]:
    """The item ids a tag (transitively) names, for the message only."""
    if tag in seen or tag not in tags:
        return set()
    seen.add(tag)
    out: set[str] = set()
    for value in tags[tag]:
        if value.startswith("#"):
            out |= expand_tag(value[1:], tags, seen)
        else:
            out.add(value)
    return out


def tag_exists(tag: str, tags: dict[str, list[str]]) -> bool:
    return tag in tags


def ingredients(recipe: dict) -> list[tuple[str, str]]:
    """(kind, id) for every ingredient a shaped, shapeless or cooking recipe names."""
    entries: list[object] = []
    key = recipe.get("key")
    if isinstance(key, dict):
        entries.extend(key.values())
    # A cooking recipe (`smelting`, `blasting`, ...) names one `ingredient`; the plural
    # `ingredients` is a shapeless one's list.
    single = recipe.get("ingredient")
    if isinstance(single, (str, dict)):
        entries.append(single)
    ingredients = recipe.get("ingredients")
    if isinstance(ingredients, list):
        for entry in ingredients:
            if isinstance(entry, list):  # shapeless alternatives
                entries.extend(entry)
            else:
                entries.append(entry)
    out: list[tuple[str, str]] = []
    for entry in entries:
        if isinstance(entry, str):
            out.append(("tag" if entry.startswith("#") else "item", entry.lstrip("#")))
        elif isinstance(entry, dict):
            if "tag" in entry:
                out.append(("tag", str(entry["tag"])))
            elif "item" in entry:
                out.append(("item", str(entry["item"])))
    return out


def blockers(port: Path) -> dict[str, dict[str, str]]:
    path = port / "tools" / "recipe-blockers.json"
    if not path.is_file():
        return {}
    return read_json(path)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--port", type=Path, default=PORT)
    parser.add_argument("--resources", type=Path, default=DEFAULT_RESOURCES)
    args = parser.parse_args()

    items_json = args.resources / "items.json"
    if not items_json.is_file():
        print(f"cannot read the target item registry: {items_json}", file=sys.stderr)
        return 2
    vanilla = set(read_json(items_json))
    own = declared_items(args.port)
    tags, empty_tags = declared_tags(args.port)
    blocked = blockers(args.port)

    failures: list[str] = []
    for path in empty_tags:
        failures.append(
            f"{path}: a tag with no values is dropped at load; declare its members"
        )
    for tag, values in tags.items():
        for value in values:
            name = value.lstrip("#")
            if value.startswith("#"):
                if not tag_exists(name, tags) and not _vanilla_tag(args.resources, name):
                    failures.append(f"tag {tag}: names missing tag {value}")
            elif name not in vanilla and name not in own:
                failures.append(f"tag {tag}: names unknown item {value}")

    # Every crafting/smelting file; the port's own `spirit_fire` type is read by
    # `runtime.rs` and counted by `/occultism recipes`, so it is not a grid here.
    recipes = sorted(
        path
        for path in (args.port / "data" / MOD / "recipe").rglob("*.json")
        if "spirit_fire" not in path.parts
    )
    if not recipes:
        failures.append("no crafting recipes found: this would pass vacuously")
    craftable: set[str] = set()
    for path in recipes:
        recipe = read_json(path)
        recipe_id = f"{MOD}:{path.stem}"
        unresolved: list[str] = []
        for kind, name in ingredients(recipe):
            if kind == "tag":
                members = expand_tag(name, tags, set())
                if not members and not _vanilla_tag(args.resources, name):
                    unresolved.append(f"#{name}")
                elif not any(m in vanilla or m in own for m in members):
                    unresolved.append(f"#{name}")
            elif name not in vanilla and name not in own:
                unresolved.append(name)
        if not unresolved:
            craftable.add(recipe_id)
            print(f"ok       {recipe_id}: every ingredient resolves")
            continue
        symbol = blocked.get(recipe_id, {}).get("blocked")
        if symbol:
            print(f"blocked  {recipe_id}: {symbol} — {', '.join(unresolved)}")
        else:
            failures.append(
                f"{recipe_id}: unresolved {', '.join(unresolved)}"
                " (a missing tag drops the recipe at load, an unknown item leaves it"
                " uncraftable; declare the item/tag or add a blocker)"
            )
    for recipe_id, record in blocked.items():
        if recipe_id in craftable:
            failures.append(
                f"{recipe_id}: blocked({record.get('blocked')}) is stale — the recipe resolves now;"
                " remove the blocker and prove it in game"
            )
        elif not any(path.stem == recipe_id.split(":", 1)[1] for path in recipes):
            failures.append(
                f"{recipe_id}: blocked({record.get('blocked')}) names no recipe in this package"
            )

    for failure in failures:
        print(f"FAIL     {failure}", file=sys.stderr)
    print(f"{len(recipes)} recipe(s), {len(craftable)} craftable, "
          f"{len(blocked)} blocked, {len(failures)} failure(s)")
    return 1 if failures else 0


def _vanilla_tag(resources: Path, tag: str) -> bool:
    """A tag the game itself ships (data/<ns>/tags/item/<path>.json)."""
    namespace, _, path = tag.partition(":")
    return (resources / "data" / namespace / "tags" / "item" / f"{path}.json").is_file()


if __name__ == "__main__":
    sys.exit(main())
