# Occultism for Horizon — spirit fire, the datura crop, the crafted blocks

A Horizon port of [Occultism](https://github.com/klikli-dev/occultism) 1.225.0
(MIT) for Minecraft Java `26.3`, as a **native SDK package** (mod API 0.2). The
delivered slices are the **spirit-fire conversion loop** (its gateway), the
**datura crop** that lights it, the four carried **loot tables**, the two
**crafting-table recipes** the package can carry (`demons_dream_essence`,
`book_of_binding_empty`), and the **otherstone/otherrock building family**: the
slabs, stairs and walls (0.2.6), the bricks tier — bricks, their slab, stair and
wall, the chiseled pair and the cracked pair (0.2.7) — the polished tier: the
smelted plain pair and their slabs, stairs and walls (0.2.8) — the cobblestone
tier mined from the raw stones (0.2.9), the button and pressure-plate pair
(0.2.10), the frames and pedestals (0.2.11), the sacrificial bowls
(0.2.12) — the tier's first block entity — the 17 impure-chalk craftings
(0.2.13) with the tags and items they need, the otherstone tablet (0.2.14), and the
32 stonecutting recipes (0.2.15) — every stone, polished stone, brick and cobble
cuts into its own slab, stairs, wall or bricks at a real stonecutter.

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
  then shaped into its slabs, stairs and walls. The cobblestone tier is mined from
  the raw stones (a break without silk touch drops `othercobblestone` or
  `othercobblerock`, silk touch keeps the stone) and shaped the same way, and a
  cobble block smelts back into the stone. All of them are pickaxe blocks (a bare
  hand breaks them and drops nothing).
- **Redstone**: one stone in a crafting table makes a stone button and two stones
  in one row make a pressure plate, in both stones; the plates read powered under
  a player and the buttons stay pressed for their template's press duration.
- **Frames and pedestals**: eight stones in a ring make an `otherstone_frame` or
  `otherrock_frame` (plain items — the mod's ritual frames), and five slabs plus
  one stone make a post-shaped `otherstone_pedestal` or
  `otherstone_pedestal_silver` (the otherrock one, under the jar's old id).
- **Bowls**: five `otherstone` make a `sacrificial_bowl`, five `otherrock` the
  dark one; a copper ingot (shapeless) or eight gold ingots around it make the
  copper and golden variants. A bowl holds one item — right-click to put one in
  (the rest of the stack stays in your hand), right-click with an empty hand to
  take exactly that stack back (its components with it), and break it to get the
  bowl and what it held. A non-golden bowl standing on a spirit fire converts what
  it holds, as the fire itself does; the golden pair stores and returns but does
  not convert.
- **Impure chalks**: the 17 grids the jar ships — the 15 coloured ones (an impure
  white chalk plus three materials, two for gray and green) and the two mirrored
  shaped white grids. The white grids take the nested `#occultism:dusts/chalk_base`
  tag (burnt otherstone/otherrock) and `#c:dusts/otherworld_wood` (otherworld
  ashes); both burnt stones smelt from the carried polished pair. The 26 dust,
  paste and essence items the grids name, and the 26 item tags they reach, are
  carried so every cell resolves; the ingredients whose only source is the
  unported crushing/ritual half are declared as items, not obtained.
- **Tablet**: the jar's two `aga`/`gsg`/`aga` grids — four otherworld ashes, four
  gold ingots and one otherstone or otherrock — each make one
  `occultism:otherstone_tablet` (the "Otherworldly Tablet").
- `/occultism info <x> <y> <z>` and `/occultism recipes` for a readout (a bowl
  answers with the stack its block entity holds).

## What it does not do (yet)

The pentacle rituals (158 recipes), the summoned spirits and their AI, the
Otherworld dimension, the guide book, accessory slots, the crushing/spirit-job
recipes and the ritual results that source most of the impure chalks' colour
materials, and the crafting recipes whose result (the rest of the
otherstone/otherrock building family — the storage controllers with their
ritual-only bases) or ingredients this package does not carry yet.
See [`notes.md`](notes.md) for the full gap list and the reason for each.

## Installing and building

One file: `occultism.hmod`. It is the whole mod — `horizon-mod.toml`,
`src/lib.rs` + `src/support.rs` (the code), `src/runtime.rs` (the pure rules and
their 81 tests), `assets/` and `data/` (the carried blocks, items, recipes, loot,
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
