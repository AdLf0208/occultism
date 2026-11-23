# Occultism for Horizon — spirit fire, the datura crop, the crafted blocks

A Horizon port of [Occultism](https://github.com/klikli-dev/occultism) 1.225.0
(MIT) for Minecraft Java `26.3`, as a **native SDK package** (mod API 0.2). The
delivered slices are the **spirit-fire conversion loop** (its gateway), the
**datura crop** that lights it, the four carried **loot tables**, the two
**crafting-table recipes** the package can carry (`demons_dream_essence`,
`book_of_binding_empty`), and the **otherstone/otherrock building family**: the
slabs, stairs and walls (0.2.6), the bricks tier — bricks, their slab, stair and
wall, the chiseled pair and the cracked pair (0.2.7) — and the polished tier:
the smelted plain pair and their slabs, stairs and walls (0.2.8).

## What works

- `occultism:spirit_fire` — a fire-like block (no collision, light 12, no
  damage, stands only on a solid face, 18 recolourable colours).
- **Lighting it**: throw a Demon's Dream Fruit (datura) on the ground and strike
  it with flint and steel (or a fire charge).
- **Converting**: any item dropped into the fire that one of the 27 recipes
  accepts becomes the recipe's result, keeping the stack's count (3 feathers →
  3 awakened feathers). This is how the mod gets its chalks, purified ink, the
  taboo book, the book of binding, the spirit-attuned gem, otherstone and
  otherrock.
- **Painting it**: right-click the fire with a chalk to change its colour.
- **Growing it**: `occultism:datura_seeds` plant the `occultism:datura` crop on
  farmland (ages 0..7, the wheat template), it ripens by random tick, a right
  click harvests the ripe crop (its loot table's fruit and seed, replanted), and
  short or tall grass broken by hand has the jar's 2% chance of a seed.
- **Crafting**: a 3x3 grid of Demon's Dream Fruit — or of the seeds, or a mix —
  makes one `occultism:demons_dream_essence`; the shapeless awakened feather +
  purified ink + taboo book makes the Book of Binding.
- **Building**: the six `otherstone`/`otherrock` slabs, stairs and walls, and
  the bricks tier on top of them — a 2x2 of the stone makes four bricks, the
  brick slab/stair/wall recipes work from those bricks, two brick slabs stacked
  make the chiseled block, and a furnace turns the bricks into the cracked pair.
  The polished pair is smelted from the raw stone (200 ticks, 0.5 experience) and
  then shaped into its slabs, stairs and walls. All of them are pickaxe blocks (a
  bare hand breaks them and drops nothing).
- `/occultism info <x> <y> <z>` and `/occultism recipes` for a readout.

## What it does not do (yet)

The pentacle rituals (158 recipes), the summoned spirits and their AI, the
Otherworld dimension, the guide book, accessory slots, and the crafting recipes
whose result (the rest of the otherstone/otherrock building family — the
cobblestone family, buttons, pressure plates, frames, pedestals, tablets, bowls —
and the impure chalks)
or ingredients this package does not carry yet. See
[`notes.md`](notes.md) for the full gap list and the reason for each.

## Installing and building

One file: `occultism.hmod`. It is the whole mod — `horizon-mod.toml`,
`src/lib.rs` + `src/support.rs` (the code), `src/runtime.rs` (the pure rules and
their 50 tests), `assets/` and `data/` (the carried blocks, items, recipes, loot,
tags and lang) and the licence notices.

A native SDK package is built against the game by the launcher:

```sh
horizon-launcher native-build .
```

The campaign's compile gate (the launcher is not installed in this environment)
is the crate's own ladder:

```sh
ports/tools/build-slot.sh run --job occ-sdk --port ports/occultism --step test -- \
    bash -o pipefail -c 'cd ports/occultism && CARGO_INCREMENTAL=0 cargo test --offline'
ports/tools/build-slot.sh run --job occ-sdk --port ports/occultism --step lint -- \
    bash -o pipefail -c 'cd ports/occultism && CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings'
ports/tools/build-slot.sh run --job occ-sdk --port ports/occultism --step build -- \
    bash -o pipefail -c 'cd ports/occultism && rm -f Cargo.lock && cargo horizon-mod build'
cargo horizon-mod check occultism.hmod
cargo horizon-mod fits occultism.hmod --build nightly
```

`tools/e2e.sh` drives the in-game scenario with `horizon-ctl`; it needs a
launcher-built `GAME=` (and optionally a `CONTROL_GAME=` without the package).

## Licence

MIT, like the original. The upstream copyright and licence travel with the port
([`LICENSE`](LICENSE), [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md)); everything
carried from the jar is upstream's MIT data. Local-only until the maintainer
approves publishing (ARCHITECTURE.md §8).
