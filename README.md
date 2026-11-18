# Occultism — a Horizon port

**Occultism** is a Minecraft mod. A magic mod inspired by the world of Jonathan Stroud's Bartimaeus. With the help of occult rituals players can summon entities from the "Other Side" to perform magic. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [klikli-dev/occultism](https://github.com/klikli-dev/occultism) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Occultism ([source](https://github.com/klikli-dev/occultism)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `occultism.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What works

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

## Not yet ported

- **No 0.2.0 in-game run in this environment.** The e2e needs a game built by
- **`assemble_content.py` in the old tree was a stray Charging Gadgets script**, not this
- Everything in the 0.1.0 gap list stands unchanged: the pentacle rituals, the spirits and

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `occultism.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Occultism was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/klikli-dev/occultism

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run occultism` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/occultism/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
