# Occultism — findings (slice 1: the spirit-fire conversion loop)

Ported from **Occultism 1.225.0** (`occultism-1.21.1-neoforge-1.225.0.jar`) for
Minecraft Java 1.21.1 / NeoForge, MIT, read from the exact ATM10 jar and from
`ports/_shared/upstream/occultism` at tag **`release/v1.21.1-1.225.0`**
(`91de0d2a46e05ce315cc5afd292d266d4f46e7ab`, the commit the jar was built from).
The Java was read for behaviour and the jar for assets/data only; the Rust is a
clean-room rewrite. Local-only until publishing is approved (ARCHITECTURE §8).

## Provenance

| Field | Value |
|---|---|
| jar | `ports/_shared/licence-cache/occultism-1.21.1-neoforge-1.225.0.jar` (kept) |
| SHA-1 / SHA-256 | `0bd65c723cb925330dd509901261992cb7b6343d` / `fc222df07edf266b38c538111522799165c1647114e8c37974a975e4a80bf514` |
| size | 6,957,914 B |
| licence (jar) | `license = "MIT"` in `META-INF/neoforge.mods.toml` → **rank 0** |
| upstream | `https://github.com/klikli-dev/occultism`, tag `release/v1.21.1-1.225.0`, commit `91de0d2a…` |
| files read | `common/block/SpiritFireBlock.java`, `crafting/recipe/SpiritFireRecipe.java`, `handlers/PlayerEventHandler.java` (`spiritFire`), `registry/OccultismBlocks.java` (spirit_fire, datura, otherstone/otherrock/otherflower/sapling), `datagen/recipe/SpiritFireRecipeProvider`, `datagen/lang/ENUSProvider` |
| declared libraries | `modonomicon` (required), `curios` (required), `geckolib`, `smartbrainlib`, `theurgy` (optional) — **mapped to gaps**, never required (ARCHITECTURE §8, ruling 2026-11-09) |

**Brief v. artefact:** the brief names the source-map row as `dimsumst/occultism`.
The artefact wins (AGENTS.md): the jar's `neoforge.mods.toml` credits
`Kli Kli`, and `ports/_shared/source-map/atm10-sources.csv` row 345 gives
`klikli-dev/occultism` (`https://github.com/klikli-dev/occultism`) — that is the
repository this job read. `dimsumst` appears nowhere in the jar or the map.

`precheck-mod.py occultism` → exit 0, `DISPATCHABLE`, jar licence MIT rank 0.

## Jar inventory (measured, not guessed)

| Area | Count |
|---|---:|
| blockstates (`assets/occultism/blockstates/`) | 164 |
| block models / item models | 219 / 334 |
| GeckoLib `geo/` + `animations/` | 21 + 21 |
| recipe JSONs under `data/occultism/recipe/` | 930 |
| — `crafting` 179 · `crushing` 240 · `miner` 194 · **`ritual` 158** · `crystallize` 54 · `stonecutting` 32 · `smelting` 19 · `spirit_trade` 14 · `blasting` 11 · **`spirit_fire` 27** · `compat` 2 | |
| loot tables | 171 |
| advancements | 1,131 |
| tags | 99 |
| Modonomicon book pages | 260 |
| worldgen | 12 |
| curios data | 5 |

The mod is a `content` / `deep` core mod (ledger row 345): pentacle rituals that
summon spirits, an Otherworld dimension, animated spirit entities, and a guide
book. None of that fits one slice.

## Why this slice (the smallest honest loop)

Three candidate loops were measured against the **jar**, not the mod's name:

1. **A pentacle ritual** (158 `ritual` recipes). The pentacle *shapes* are not
   data: they are built in code (`PentacleProvider` datagen → `PentacleMapping`),
   and the ritual needs the chalk-glyph multiblock, sacrificial bowls as block
   entities with one item each, the golden bowl's activation, a `duration`
   timer and spirit jobs. That is a machines-tier subsystem on its own — and
   the spirits it summons need the `smartbrainlib` AI that has no Horizon API.
2. **A spirit job** (crushing 240 / miner 194). Every job recipe runs *inside* a
   summoned spirit entity; nothing is obtainable without (1).
3. **Spirit fire** (27 recipes). One block, one rule, fully data-driven, and it
   is the gateway of the whole progression: the book of binding, purified ink,
   the taboo book, all 16 chalks, the spirit-attuned gem, otherstone/otherrock,
   the otherworld sapling and the Otherworld materials all come out of it.
   Lit by throwing datura on the ground and striking it with flint and steel.

So slice 1 is (3): **the conversion loop that produces the ritual chain's own
inputs**, with its book-free data (the 27 `occultism:spirit_fire` recipes). The
pentacle ritual is recorded as slice 2 and the spirits/dimension as deep-tier
gaps.

## Behaviour read from upstream (the spec this port implements)

`SpiritFireBlock` is a `BaseFireBlock` with **0 fire damage**; its properties are
`noCollission`, `instabreak`, `lightLevel 12`, `SoundType.WOOL`, `noLootTable`,
`mapColor COLOR_PURPLE`, and a `color` enum state defaulting to `white`.

- **Conversion** (`entityInside`): an `ItemEntity` whose stack matches an
  `occultism:spirit_fire` recipe is removed; the recipe result is assembled with
  **`result.setCount(input.getCount())`** and dropped at the block's centre
  (`y + 0.5`); `occultism:start_ritual` plays at the block (blocks, 1.0/1.0).
  `SpiritFireRecipe.matches` is `ingredient.test(stack)`; `isSpecial()`.
- **Ignition** (`PlayerEventHandler.spiritFire`, `RightClickBlock`): with flint
  and steel, a fire charge, or an item with the `light_fire`/`light_campfire`
  ability, if a `#occultism:start_spiritfire` item entity (the jar's tag is
  `[.../datura]`) lies in `AABB(-1,-1,-1,1,1,1).move(center(pos))`, then all such
  entities are removed, the block is placed at `pos` when it can be replaced and
  at `pos.relative(face)` otherwise, the item's own sound plays, the flint and
  steel loses 1 durability (a fire charge is used up), the event is cancelled and
  the arm swings.
- **Recolour** (`useItemOn`): a chalk item right-clicked on the fire sets its
  `color`; the jar has 18 values (`white` … `pink`, `rainbow`, `void`) and 16 of
  them are reachable from the 16 chalk items whose spirit-fire recipes exist.
- **Survival** (`canSurvive`/`updateShape`): it stands only on a sturdy upper
  face and turns to air when that support is gone.

## Slice boundary

**In:** the `occultism:spirit_fire` block (18-value colour state, torch-class
survival, no collision, light 12, no drops); ignition by datura + flint and steel
/ fire charge; the conversion rule for the **26 loadable** `spirit_fire` recipes
(with `count = input count`); chalk recolouring for the 16 carried chalks; the
45 carried items (27 results, 16 impure chalks, datura, demon's dream essence),
4 of them blocks; the 27 recipe files, the 3 tags the loop needs and 3 `c:` tags;
the `/occultism info|fires|recipes` readouts.

**Out (recorded gaps):**
- the pentacle ritual system (158 recipes, chalk glyphs, sacrificial bowls,
  golden bowl, timers, spirit jobs) — **slice 2**;
- the summoned spirits, their GeckoLib animations and `smartbrainlib` AI, the
  Otherworld dimension and its worldgen — deep tier;
- the Modonomicon guide book (260 pages) — a library mapped to a gap (no
  guide-book reader at the pin);
- Curios slots (the familiar ring and the other 5 curios files);
- the datura crop block and its growth: `occultism:datura` is a plain item here;
- the crushing/miner/crystallize/spirit-trade recipes (they need spirits);
- `otherworld_ashes`: its only inputs are `#occultism:otherworld_logs`, which the
  slice does not carry, so the rule is skipped and reported (26 of 27 load);

## Named library gaps (what each would have provided)

| Declared library | Would have provided | What this slice does |
|---|---|---|
| **Modonomicon** (required) | the in-game guide book, its 260 pages, the ritual/pentacle pages, the recipe pages | no book; the same knowledge is in `notes.md`/`FINDINGS.md`. A guide-book reader is an engine feature (ARCHITECTURE §7, `docs/modpacks.md`) |
| **Curios** (required) | accessory slots — the familiar ring, the spirit lamp slots | no accessory slots; a Horizon accessory-slot API does not exist (planned 0.4) |
| **GeckoLib** | the 21 animated spirit models/animations | no spirits, so no animated models; the block uses a carried static model |
| **SmartBrainLib** | the summoned spirits' job AI (crush/mine/transport) | no spirits; the jobs are slice 3+ |

## Deviations from upstream (deliberate, and why)

| Area | Upstream | This port |
|---|---|---|
| conversion timing | `entityInside`, every tick while the entity's box meets the block | the entity's own **tick** (`entity_tick_post`) + the block under its position; the same one-per-tick cadence, a hair narrower at the box edges |
| the converted item | the old entity is discarded and a new stack is dropped at the block centre | the same entity's stack is replaced (`native_set_item_entity`); position, motion and pickup delay are kept |
| ignition reach | *any* right click with a lighter, even on a block that has its own use | only where the clicked block has no use of its own (`use_item_on` runs after the block's use); a chest/table click does not light the datura |
| lighter wear | flint and steel −1 durability, a fire charge is used up | nothing is consumed: the pin has no stamped held-stack damage helper (ARCHITECTURE §3) |
| sounds | `flintandsteel.use`/`firecharge.use`, `occultism:start_ritual` | no sound: `native_play_sound` is `GameLevel`-only and not stamped at this pin |
| fire colour | tinted per colour by `getColor` | the state is carried and settable, but the client draws the untinted model (no stamped block-tint hook) |
| `rainbow`/`void` colours | reachable with the rainbow/void chalks | unreachable: those two chalks are not carried |
| otherstone/otherrock drops | silk touch → itself, else `othercobblestone`/`othercobblerock` | the block drops itself; the cobblestone family and the stone drop chain are not carried |
| natural sapling | grows the Otherworld tree, `uncovered` reveals it | `short_grass` class (no growth, no feature), `uncovered` never set |
| datura | edible (`SpiritHealingItem`), can grant Third Eye | a plain stack-64 item |
| impure chalks | crafted from crushed materials | carried items; their recipes are slice 3 |
