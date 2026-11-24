#!/usr/bin/env python3
"""Prove every declared occultism item and block id in a real game (slice 15, phase AA).

This is the parity rule's "every block and item" row for the whole package:
`give` each declared item and read it back from the parsed inventory, place
each declared block and read its default state back from `ctl block`, and
assert a made-up id is refused. One PASS line per id, like phase Z's 32 cuts.

The id set comes from the **artefact**, not the brief:

* `data/occultism/horizon/item/*.json` — explicit item declarations (71);
* `data/occultism/horizon/block/*.json` — block declarations (52), of which
  the two with `"item": false` have no block item;
* `data/occultism/horizon/creative_tab/spirit_fire.json` — the tab's 121 ids.

The giveable set is the union of the explicit item declarations and the block
items; the harness asserts it equals the creative tab's set exactly (121 ids).

The expected default states live in `all_ids_states.json` next to this file.
They were measured on the previous green package and are cross-checked here
against the declaration's `default_state`/`state_properties` and the carried
blockstate JSON before any game assertion runs, so a typo in the golden table
cannot silently weaken the check.

`SELFTEST=all-ids` asserts **one more** id than declared and must FAIL (`EXIT:1`).
`PROBE_STATES=1` prints the observed states instead of asserting them (used
once, to record the golden table).

Usage: python3 tools/all_ids_check.py SOCK CTL X Y Z
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
ITEM_DIR = PORT / "data" / "occultism" / "horizon" / "item"
BLOCK_DIR = PORT / "data" / "occultism" / "horizon" / "block"
TAB = PORT / "data" / "occultism" / "horizon" / "creative_tab" / "spirit_fire.json"
BLOCKSTATE_DIR = PORT / "assets" / "occultism" / "blockstates"
STATES = Path(__file__).resolve().parent / "all_ids_states.json"
UNKNOWN = "occultism:definitely_not_an_item"
# Vanilla template properties the 26.3 blockstate files don't spell out (they
# belong to the block definition, e.g. every slab has `waterlogged`). This is
# independent vanilla data, not something read from the package under test.
TEMPLATE_PROPS: dict[str, set[str]] = {
    "waterlogged": {"true", "false"},
    "powered": {"true", "false"},
    "face": {"floor", "wall", "ceiling"},
    "facing": {"north", "south", "east", "west", "up", "down"},
    "up": {"true", "false"},
    "north": {"none", "low", "tall"},
    "south": {"none", "low", "tall"},
    "east": {"none", "low", "tall"},
    "west": {"none", "low", "tall"},
    "half": {"top", "bottom"},
    "shape": {"straight", "inner_left", "inner_right", "outer_left", "outer_right"},
}
# The probe and the assertion share one scratch column in the End arena.
BLOCK_POS = None  # set from argv: (x, y, z)


def fail(msg: str) -> None:
    raise SystemExit(f"ALL IDS FAIL: {msg}")


def enumerate_ids() -> tuple[list[str], list[str], dict[str, dict]]:
    """Read the id sets from the package's own data files."""
    if not ITEM_DIR.is_dir() or not BLOCK_DIR.is_dir():
        fail(f"no declaration directories {ITEM_DIR} / {BLOCK_DIR}")
    explicit = {f"occultism:{p.stem}" for p in sorted(ITEM_DIR.glob("*.json"))}
    decls = {p.stem: json.loads(p.read_text()) for p in sorted(BLOCK_DIR.glob("*.json"))}
    blocks = [f"occultism:{stem}" for stem in sorted(decls)]
    no_item = [f"occultism:{stem}" for stem, d in decls.items() if d.get("item") is False]
    giveable = sorted(explicit | {b for b in blocks if b not in no_item})
    tab = sorted(set(json.loads(TAB.read_text())["items"]))
    if len(explicit) != 71:
        fail(f"enumerated {len(explicit)} explicit item declarations, expected 71")
    if len(decls) != 52:
        fail(f"enumerated {len(decls)} block declarations, expected 52")
    if set(giveable) != set(tab):
        fail(
            f"the giveable set ({len(giveable)}) differs from the creative tab ({len(tab)}): "
            f"tab-only={sorted(set(tab) - set(giveable))}, "
            f"declaration-only={sorted(set(giveable) - set(tab))}"
        )
    if len(giveable) != 121:
        fail(f"enumerated {len(giveable)} giveable ids, expected 121")
    return giveable, blocks, decls


def _read_blockstate_props(path: Path, allowed: dict[str, set[str]]) -> None:
    if not path.is_file():
        return
    bs = json.loads(path.read_text())
    for key in bs.get("variants", {}):
        for part in key.split(","):
            if "=" in part:
                k, v = part.split("=", 1)
                allowed.setdefault(k, set()).add(v.lower())
    for part in bs.get("multipart", []):
        when = part.get("when") or {}
        entries: list[dict] = []
        if "OR" in when:
            entries.extend(when["OR"])
        else:
            entries.append(when)
        for entry in entries:
            for k, v in entry.items():
                allowed.setdefault(k, set()).add(str(v).lower())


def _allowed_props(block_id: str, decl: dict) -> dict[str, set[str]]:
    """Every property value the block's own data and its declared template allow."""
    allowed: dict[str, set[str]] = {}
    for key, value in (decl.get("default_state") or {}).items():
        allowed.setdefault(key, set()).add(str(value).lower())
    for key, spec in (decl.get("state_properties") or {}).items():
        for value in spec.get("values", []):
            allowed.setdefault(key, set()).add(str(value).lower())
    _read_blockstate_props(BLOCKSTATE_DIR / f"{block_id.removeprefix('occultism:')}.json", allowed)
    # The vanilla template owns most of the state properties (type, facing,
    # waterlogged, powered...). Its blockstate is read from the prepared 26.3
    # resources, never from the package under test.
    for key, values in TEMPLATE_PROPS.items():
        allowed.setdefault(key, set()).update(values)
    template = decl.get("template", "")
    if template.startswith("minecraft:"):
        stem = template.removeprefix("minecraft:")
        roots = []
        env_res = os.environ.get("RES") or os.environ.get("HORIZON_RES")
        if env_res:
            roots.append(Path(env_res))
        roots += [
            Path("/home/adlf/.cache/horizon-e2e/root/data/horizon-launcher/versions/"
                 "nightly-2026-11-20-0a875da/.local/horizon-client/26.3"),
            Path("/home/adlf/Desktop/horizon/.local/horizon-client/26.3"),
        ]
        for root in roots:
            candidate = root / "assets" / "minecraft" / "blockstates" / f"{stem}.json"
            if candidate.is_file():
                _read_blockstate_props(candidate, allowed)
                break
    return allowed


def load_states(blocks: list[str], decls: dict[str, dict]) -> dict[str, str]:
    table = json.loads(STATES.read_text())
    missing = [b for b in blocks if b not in table]
    extra = [b for b in table if b not in blocks]
    if missing or extra:
        fail(f"the golden state table is out of sync: missing={missing}, extra={extra}")
    for block_id in blocks:
        expected = table[block_id]
        if not expected.startswith(block_id):
            fail(f"golden state for {block_id} names another block: {expected}")
        suffix = expected[len(block_id):]
        props = {}
        if suffix:
            if not (suffix.startswith("[") and suffix.endswith("]")):
                fail(f"golden state for {block_id} is malformed: {expected}")
            for part in suffix[1:-1].split(","):
                if "=" not in part:
                    fail(f"golden state for {block_id} has a bad property: {part}")
                k, v = part.split("=", 1)
                props[k] = v
        allowed = _allowed_props(block_id, decls[block_id.removeprefix("occultism:")])
        for k, v in props.items():
            if k not in allowed or v.lower() not in allowed[k]:
                fail(
                    f"golden state for {block_id} names {k}={v}, which its own data does not "
                    f"declare (allowed for {k}: {sorted(allowed.get(k, []))})"
                )
    return table


def main() -> int:
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = (int(v) for v in sys.argv[3:6])
    BLOCK_POS = (x, y, z)
    selftest = (
        os.environ.get("ALL_IDS_SELFTEST", "0") not in ("", "0")
        or os.environ.get("SELFTEST", "0") == "all-ids"
    )
    probe = os.environ.get("PROBE_STATES", "0") not in ("", "0")
    items, blocks, decls = enumerate_ids()
    states: dict[str, str] = {}
    if not probe:
        states = load_states(blocks, decls)
    print(
        f"enumerated {len(items)} giveable item ids (71 explicit items + "
        f"{len(items) - 71} block items) and {len(blocks)} block ids "
        f"(2 with no item) from {PORT}"
    )
    if selftest:
        print(f"(selftest: asserting {len(items) + 1} items and {len(blocks) + 1} blocks)")

    def call(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            fail(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def raw(*args: str) -> tuple[int, str, str]:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        return out.returncode, out.stdout, out.stderr

    def wait(ticks: int = 2) -> None:
        call("wait", str(ticks))

    def counts() -> dict[str, int]:
        return call("inventory").get("counts", {})

    def clear() -> None:
        call("command", "clear @s")
        call("command", "kill @e[type=minecraft:item]")
        wait(2)

    def setblock(block: str) -> dict:
        bx, by, bz = BLOCK_POS
        call("command", f"execute in minecraft:the_end run setblock {bx} {by} {bz} {block}")
        wait(2)
        return call("block", str(bx), str(by), str(bz))

    # ---- item ids: give -> parsed inventory ---------------------------------
    seen_items = 0
    for item in ([] if probe else items):
        clear()
        rc, out, err = raw("give", item, "1")
        wait(2)  # the give lands on the next tick
        got = counts()
        if rc != 0:
            fail(f"give {item} exited {rc}: {out.strip()} {err.strip()}")
        if got.get(item) != 1 or sum(got.values()) != 1:
            fail(f"give {item} 1 -> inventory {json.dumps(got, sort_keys=True)}, expected exactly one stack of {item}")
        seen_items += 1
        print(f"PASS item {item}: give 1 -> inventory {json.dumps(got, sort_keys=True)}")

    if not probe:
        clear()
        rc, out, err = raw("give", UNKNOWN, "1")
        wait(2)
        got = counts()
        if sum(got.values()) != 0 or got.get(UNKNOWN, 0) != 0:
            fail(f"the made-up id {UNKNOWN} gave {json.dumps(got, sort_keys=True)}")
        print(f"PASS unknown item {UNKNOWN}: rc={rc}, gave nothing ({out.strip()}{err.strip()})")

    # ---- block ids: setblock -> parsed block state --------------------------
    seen_blocks = 0
    for block in blocks:
        reply = setblock(block)
        name = reply.get("block")
        state = reply.get("state")
        if name != block:
            fail(f"setblock {block} -> ctl block names {name!r}")
        if probe:
            print(f"STATE {block} {state}")
        else:
            if state != states[block]:
                fail(f"setblock {block} -> state {state!r}, declared default {states[block]!r}")
            print(f"PASS block {block}: {state}")
        seen_blocks += 1
        setblock("minecraft:air")
        wait(1)

    setblock(UNKNOWN)
    reply = call("block", str(x), str(y), str(z))
    if reply.get("block") != "minecraft:air":
        fail(f"the made-up id {UNKNOWN} placed {reply.get('block')!r}")
    print(f"PASS unknown block {UNKNOWN}: setblock left {reply.get('block')}")

    # ---- the fail-capability count flip ------------------------------------
    want_items = len(items) + (1 if selftest else 0)
    want_blocks = len(blocks) + (1 if selftest else 0)
    if not probe and (seen_items != want_items or seen_blocks != want_blocks):
        fail(
            f"asserted {seen_items} item ids and {seen_blocks} block ids, "
            f"expected {want_items} and {want_blocks} (declared {len(items)}/{len(blocks)})"
        )
    print(
        f"PASS: all {seen_items} declared item ids give exactly one stack, all {seen_blocks} "
        "declared block ids read their declared default state, and a made-up id is refused "
        "as an item and as a block"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
