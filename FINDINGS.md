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
| otherstone/otherrock drops | silk touch → itself, else `othercobblestone`/`othercobblerock` | **the same since 0.2.9**: the jar's `alternatives`/`match_tool` table is carried (the 0.2.4 self-drop simplification is gone); the reverse smelt pair (cobble → stone) is still not carried |
| natural sapling | grows the Otherworld tree, `uncovered` reveals it | `short_grass` class (no growth, no feature), `uncovered` never set |
| datura | edible (`SpiritHealingItem`), can grant Third Eye | a plain stack-64 item |
| impure chalks | crafted from crushed materials | carried items; their recipes are slice 3 |

## Upstream defects found (and fixed in the port)

| Artefact | Defect | Proof | Port |
|---|---|---|---|
| `data/occultism/loot_table/blocks/otherrock_slab.json` | **absent** from Occultism 1.225.0, although `OccultismBlocks.OTHERROCK_SLAB` is `LootTableType.CUSTOM` (no generated table either), so the block would drop nothing | `unzip -l occultism-1.21.1-neoforge-1.225.0.jar \| grep loot_table/blocks \| grep slab` lists the otherstone / othercobblestone / otherplanks / otherstone_bricks / polished slabs and not this one; the unit test that every building block has a table fails on the path | the port ships `otherrock_slab.json` (the same table as `otherstone_slab`), proven in game by a wooden-pickaxe break dropping exactly one (0.2.6) |
| `data/occultism/loot_table/blocks/otherrock_bricks_slab.json` | **absent** from the jar, same `LootTableType.CUSTOM` registration (0.2.7's instance) | the same listing: every otherrock brick table is there, the slab's is not | the port ships it (the `otherstone_bricks_slab` table with its `type=double` branch), proven by a pickaxe break (0.2.7) |
| `data/occultism/loot_table/blocks/polished_otherrock_slab.json` | **absent** from the jar, same registration (0.2.8's instance) | the listing of the polished tables is seven of eight, the otherrock slab missing | the port ships it, proven by a pickaxe break and a double slab dropping two (0.2.8) |
| `data/occultism/loot_table/blocks/othercobblerock_slab.json` | **absent** from the jar, same registration (0.2.9's instance). The pattern is now exact: **every `otherrock`-family slab lacks a table** (`otherrock_slab`, `otherrock_bricks_slab`, `polished_otherrock_slab`, `othercobblerock_slab`), while every `otherstone`/`othercobblestone` slab has one (`othercobblestone_slab` is present) | the cobble-ore listing (seven of eight) in `ports/_shared/evidence/occultism/jar-loot-inventory-0.2.9.txt`; the unit test `every_cobble_block_has_a_26_3_loot_table_that_drops_itself` reads each path | the port ships it (the `othercobblestone_slab` table with its `type=double` branch), proven in game by a pickaxe break dropping exactly one and a double slab dropping two (0.2.9) |

## Slice 10 findings (2026-11-23) — the frames are items, the pedestals are plain blocks

Read at tag `release/v1.21.1-1.225.0` (commit `91de0d2a46e05ce315cc5afd292d266d4f46e7ab`): the jar's
`assets/occultism/{blockstates,models}` and `data/occultism/{recipe,loot_table}`, plus
`registry/OccultismItems.java`, `registry/OccultismBlocks.java`,
`common/block/NonPathfindableBlock.java`, `datagen/recipe/OccultismRecipeProvider.java:638-674` and
`datagen/recipe/RitualRecipes.java:1390-1560`.

**`otherstone_frame` / `otherrock_frame` are items** (`OccultismItems.java:407-410`,
`new Item(defaultProperties())`, `minecraft:paper`-equivalent). The jar carries an item model
(`minecraft:item/generated` + `occultism:item/<frame>`) and an `en_us` name for each, and **no**
blockstate, block model, block declaration or loot table. Their only use is as ritual ingredients
(`RitualRecipes`, e.g. the otherworld-goggles ritual) — the recipes are in the jar's 3x3 ring
(`ooo`/`o o`/`ooo`, eight of the stone → one frame). Anything that treats them as "the wall-mounted
item frame" is wrong for this version.

**`otherstone_pedestal` and `otherstone_pedestal_silver` are `NonPathfindableBlock`s**
(`OccultismBlocks.java:535-541`): `Properties.ofFullCopy(OTHERSTONE).noOcclusion()`, so hardness 1.5,
resistance 30, `MapColor.STONE`, `requiresCorrectToolForDrops` — and no block entity, no container, no
`use`: a pedestal is the pentacle's decorative stand, not a display case. `NonPathfindableBlock` adds
the four-box post shape and `isPathfindable → false`.

**The otherrock pedestal's id is the jar's old one**: `OTHERROCK_PEDESTAL = register(
"otherstone_pedestal_silver", …)` with the `TODO: change id in next major version` comment, and its
recipe file is `recipe/crafting/otherrock_pedestal.json` while the result is
`occultism:otherstone_pedestal_silver`. Both were carried as-is.

**The storage-controller bases are ritual results, not crafting results**:
`RitualRecipes.java:1544-1557` makes `storage_controller_base` (`…_dark`) from a Foliot ritual, the
crafting recipes for the actuator need the base **and** a `dimensional_matrix` (another ritual), and
`models/block/storage_controller_base.json` parents `occultism:block/storage_controller` (the machine
the port does not carry). They are recorded `blocked(ritual:craft_storage_controller_base)` rather
than shipped as creative-only blocks.

**The family's loot tables**: `otherstone_pedestal` and `otherstone_pedestal_silver` are present (no
omission this time); the frames have none (items); the two bases' tables exist and are not carried
with the bases.

## Slice 11 findings (2026-11-23) — the sacrificial bowls are a one-slot block entity

Read at tag `release/v1.21.1-1.225.0` (commit `91de0d2a46e05ce315cc5afd292d266d4f46e7ab`): the jar's
`data/occultism/{recipe/crafting,loot_table/blocks}`, `assets/occultism/{blockstates,models,textures}`,
and `common/block/SacrificialBowlBlock.java`, `common/block/GoldenSacrificialBowlBlock.java`,
`common/blockentity/SacrificialBowlBlockEntity.java`,
`common/blockentity/GoldenSacrificialBowlBlockEntity.java`, `registry/OccultismBlocks.java:462-515`.

**Ten bowls, two shapes.** `SacrificialBowlBlock` (`sacrificial_bowl`, `dark_`, `copper_`, `silver_`,
`dark_copper_`, `dark_silver_`) is a `DirectionalBlock` + `EntityBlock` with a `facing` state; the four
`GoldenSacrificialBowlBlock`s (`golden_`, `iesnium_`, `dark_golden_`, `dark_iesnium_`) are plain
`Block`s with a `Block.box(3,2,3,13,8,13)` shape and no `facing`. All ten share
`Properties.of().mapColor(STONE).sound(STONE).strength(1.5f, 30).noOcclusion()` and **none** calls
`requiresCorrectToolForDrops()`: a bare hand drops one (the `minecraft:stone` template's
`requires_tool: true` had to be overridden).

**One slot, limit one.** `SacrificialBowlBlockEntity.itemStackHandler = new ItemStackHandler(1)` with
`getSlotLimit → 1`. `useItemOn` (server, non-sneak) inserts the held stack (`insertItem(0, held,
false)`: one in, rest stays), takes out with an empty hand (`setItemInHand(hand, extractItem(0, 64,
false))`) or into the inventory when the hand is full, and answers `SUCCESS` for every click on one of
its bowls. `onRemove` drops the inventory; in this port the engine's `Containers.dropContents` for a
declared container is that drop.

**The golden pair's difference.** `GoldenSacrificialBowlBlockEntity extends SacrificialBowlBlockEntity`
— the same one-slot store — and its `activate()` is the **ritual** path (identify the ritual recipe,
validate the pentacle, insert the activation item, start it). It also skips the inherited spirit-fire
conversion by an `instanceof` check in `onContentsChanged`. In a world without the ritual registry its
own `activate` finds no recipe and answers `false`, so the inherited storage `useItemOn` runs: the
golden bowl in this port is a storage bowl that does not convert, which is exactly what upstream does
with no rituals available.

**The conversion in a bowl.** `onContentsChanged`: if the bowl is not golden and the block below is a
spirit fire (or the spirit campfire), the held stack is replaced with the `occultism:spirit_fire`
recipe's result — except `otherflower`, which the jar excludes by item.

**The family's loot tables are all present** (ten, one per bowl), so the `otherrock`-slab omission rule
has no instance here; the four bowls this port does not carry are the silver pair (their grid's
`#c:ingots/silver` has no member in the package) and the iesnium pair (ritual results).
