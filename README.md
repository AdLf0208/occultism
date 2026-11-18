# Occultism for Horizon — spirit fire (slice 1)

A Horizon port of [Occultism](https://github.com/klikli-dev/occultism) 1.225.0
(MIT) for Minecraft Java `26.3`, as a **native SDK package** (mod API 0.2). This
is **slice 1**: the **spirit-fire conversion loop**, the gateway of Occultism's
ritual chain, with the book-free data that loop reads (27
`occultism:spirit_fire` recipes).

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
- `/occultism info <x> <y> <z>` and `/occultism recipes` for a readout.

## What it does not do (yet)

The pentacle rituals (158 recipes), the summoned spirits and their AI, the
Otherworld dimension, the guide book and accessory slots. See
[`notes.md`](notes.md) for the full gap list and the reason for each.

## Installing and building

One file: `occultism.hmod`. It is the whole mod — `horizon-mod.toml`,
`src/lib.rs` + `src/support.rs` (the code), `src/runtime.rs` (the pure rules and
their 19 tests), `assets/` and `data/` (the carried block, items, recipes, loot,
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
