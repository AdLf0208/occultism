# Occultism → Horizon

Ported from **Occultism 1.225.0** (`occultism-1.21.1-neoforge-1.225.0.jar`, MIT,
`klikli-dev/occultism` at tag `release/v1.21.1-1.225.0`), Minecraft Java 1.21.1
NeoForge. The upstream source and the jar were read for behaviour and assets
only; the Rust is a clean-room rewrite. Local-only until publishing is approved
(ARCHITECTURE.md §8). Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`),
mixins **`e59da90…`** (schema 6/native-1); 0.2.0–0.2.2 were built at `0b14f8a`.

Status: **ported-partial** — slice 1 (the spirit-fire conversion loop, with its
book-free data) and slice 2 (the datura crop and its seed source, §"Slice 2"
below) are implemented and verified in game. The gap list below is the rest of
the mod. **0.2.0 (2026-11-18):** the same slice republished as the flat
SDK `.hmod` (mod API 0.2) after the `.hnmod` retirement; see
[`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18).
**0.2.3 (2026-11-21):** the crop, its seeds and the 2% grass seed source, at the
new pin. **0.2.4 (2026-11-21):** the four carried block loot tables
(`otherstone`, `otherrock`, `otherflower`, `otherworld_sapling_natural`) placed,
broken and asserted in game — the parity-ruling gap the 0.2.3 record left.
**0.2.5 (2026-11-21):** the two crafting-table recipes the package can carry
(`demons_dream_essence` from the datura fruit or seed, `book_of_binding_empty`),
plus the one-click/one-harvest fix the full run found (§"Slice 4").
**0.2.6 (2026-11-22):** the first tier of the `otherstone`/`otherrock` building
family — the six slabs, stairs and walls, with their recipes, loot tables,
models, tags and the jar's missing `otherrock_slab` table (§"Slice 5").
**0.2.7 (2026-11-22):** the bricks tier of the same family — the two brick
blocks with their slab/stair/wall, the chiseled pair and the smelted cracked
pair, and the jar's missing `otherrock_bricks_slab` table (§"Slice 6").

## Class + tier and the approach

- **Class:** `content`; **tier:** `deep` (`ports/ledger.csv` row 345;
  provenance `ports/_shared/source-map/atm10-sources.csv`). The mod's deep half
  is the summoned spirits, their GeckoLib animations, the pentacle ritual blocks
  and the Otherworld dimension; **this slice is the logic-tier loop those systems
  start from** — the honest shape for it is carried data plus `use`/tick hooks,
  not a block entity, a menu or an entity.
- **Approach** (`horizon/docs/modpacks.md`, "Tiers" and "Registries 0.2"): a
  `logic`-tier slice of a `deep` mod — the block, the 45 items and the 27
  recipes are **carry-over**; the conversion rule, the ignition, the recolour
  and the readouts are the **agent port**. Nothing here needs a machine, an
  entity or a dimension, so nothing here is built as one.
- **Licence:** MIT, read from the jar's `META-INF/neoforge.mods.toml` → **rank
  0** (permissive). Ported now because it is the rank-0 `content`/`deep` core mod
  the queue offers (`precheck-mod.py occultism` → `DISPATCHABLE`) and the gateway
  of the whole ritual chain.
- **Why this slice and not a ritual** (`FINDINGS.md`): the 158 pentacle rituals
  need pentacle shapes that live in *code*, sacrificial-bowl block entities and
  a summoned-spirit AI that has no Horizon API; the 27 `spirit_fire` recipes are
  complete data and produce the ritual chain's own inputs (all 16 chalks, the
  book of binding, purified ink, the taboo book, the spirit-attuned gem,
  otherstone/otherrock, the sapling).

## What the slice does

- **The block.** `occultism:spirit_fire` on the **`minecraft:torch`** template
  (vanilla's only fire-like class with no fire damage and a sturdy-face
  survival rule): no collision, instabreak, light 12, wool sound, purple map
  colour, empty drops, a declinable `color` state with the jar's 18 values
  (default `white`), and the jar's animated model/blockstate.
- **Ignition.** Flint and steel or a fire charge struck on a block with a
  `#occultism:start_spiritfire` item entity (datura) inside upstream's
  `AABB(-1,-1,-1,1,1,1)` around the clicked block's centre: every datura is
  consumed and the fire goes on the clicked block when it can be replaced, else
  on the clicked face; the click is claimed, so vanilla never lights its own
  fire.
- **Conversion.** Each item entity's tick: an entity standing in a spirit fire
  whose stack matches a carried `occultism:spirit_fire` recipe becomes that
  recipe's result with **the input's count** (upstream's `assemble`), as
  `run/data/occultism/recipe/spirit_fire/*.json` says.
- **Recolour.** A chalk right-clicked on the fire sets `color` (16 chalks; the
  gold chalk is the yellow one).
- **Readouts.** `/occultism info <x> <y> <z>` (block, colour, last conversion,
  rules loaded — **operator only, level 2, since 0.2.2**) and `/occultism
  recipes` (rules, files, distinct inputs, skipped) — the e2e asserts the numbers.
- **Carried content** (the retired `tools/carry_content.py`, decisions in
  [`../../_shared/carry-work/occultism/SOURCE.md`](../../_shared/carry-work/occultism/SOURCE.md)):
  5 blocks, 41 items, 27 recipes, 7 tags, 4 loot tables, the blockstates,
  models, textures and the trimmed lang file — 172 files (the measured tree; the carry
  report's stored count said 171, and the tree is what the retired content package carries).

### Behaviour read from upstream (and where this port differs)

| Area | Upstream | This port |
|---|---|---|
| conversion detection | `entityInside`, every tick while the entity's box meets the block | the entity's own tick (API 146) + the block under its position: the same one-per-tick cadence, a hair narrower at the box edges |
| the converted item | old entity discarded, a new stack dropped at the block centre | the same entity's stack replaced (API 42): position, motion and pickup delay kept |
| count | `assemble` sets the result's count to the input's | same, and pinned by a unit test and by the e2e (3 feathers → 3 awakened feathers) |
| ignition target | any right click with a lighter, even on a block with its own use | only where the clicked block has no use of its own (`use_item_on` runs after the block's use) |
| lighter wear | flint and steel −1 durability, fire charge used up | nothing is consumed: no stamped held-stack damage helper at the pin (§3) |
| sounds | `flintandsteel.use` / `firecharge.use`, `occultism:start_ritual` | none: `native_play_sound` is `GameLevel`-only and **not stamped** at the pin |
| fire colour in the world | tinted by `getColor` per colour | the state is carried and settable; the client draws the untinted model (no stamped block-tint hook) |
| `rainbow`/`void` colours | reachable with the rainbow/void chalks | unreachable: those two chalks are not carried (their spirit-fire recipes do not exist) |
| otherstone/otherrock | silk touch → itself, else `othercobblestone`/`othercobblerock` | the block drops itself; the cobblestone family is not carried |
| otherflower | `FlowerBlock` granting the Third Eye effect | poppy template: the flower works, the Third Eye stew effect is not carried |
| natural sapling | grows the Otherworld tree; `uncovered` reveals it | `short_grass` class (no growth); `uncovered` declared but never set |
| datura | edible, can grant Third Eye | a plain stack-64 item |
| impure chalks | crushed from ores | carried items; their recipes are the next slice |

## Slice boundary

**In:** the spirit-fire block with its 18-value colour state; ignition by datura
+ flint and steel / fire charge; the conversion of the 26 loadable recipes with
count preservation; chalk recolouring; the 45 items, 4 block results, 27 recipe
files and the tags the loop needs; the `/occultism` readouts; the embedded
content in the one `.hnmod`.

**Out (gaps, next slices):** the pentacle ritual system (158 recipes, chalk
glyph blocks, sacrificial bowls, the golden bowl, ritual timers, spirit jobs);
the summoned spirits and their `smartbrainlib` AI; the GeckoLib models and
animations (21); the Otherworld dimension and its worldgen; the Modonomicon book
(260 pages); Curios accessory slots; the crusher/miner/crystallize/spirit-trade
recipes (they need spirits); the datura crop; `otherworld_ashes` (its only
inputs are Otherworld logs, so its rule is skipped — 26 of 27 load, reported by
`/occultism recipes`).

## Named library gaps (what each would have provided)

| Declared library | What it would have provided | What this slice does |
|---|---|---|
| **Modonomicon** (required) | the guide book, its 260 pages, ritual and recipe pages | nothing: no guide-book reader at the pin; the knowledge is in this file |
| **Curios** (required) | accessory slots (familiar ring, spirit lamp) | nothing: no accessory-slot API (planned 0.4) |
| **GeckoLib** | the 21 animated spirit models | nothing: no spirits in this slice |
| **SmartBrainLib** | the spirits' job AI | nothing: no spirits in this slice |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Verification

Cheapest first; every command through `ports/tools/build-slot.sh`, logs in
`ports/_shared/evidence/occultism/` and the full table in
[`VALIDATION.md`](VALIDATION.md).

| Step | Result |
|---|---|
| `cargo test` RED (stubs) | FAILED: 6 passed, 10 failed on assertions (not compile errors) |
| `cargo test` GREEN | PASSED: 16 tests |
| `cargo test` RED→GREEN (the in-game ingredient-shape bug) | 16 passed / **3 failed** on the new `IngredientRef` tests → **19 passed** after the fix |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings |
| `tools/carry_content.py` + `--check` | 5 blocks, 41 items, 27 recipes, 171 files; matches the jar |
| `tools/package.sh` | `.hmod` 86,714 B (`2e2b74de…`), 171 entries, "no code: content only", **rebuild byte-identical**; `.hnmod` 69,082 B (`777bba62…`), 11 entries, deterministic here |
| `horizon.py stage` | ok (`stage_changed: true`, 302 s) |
| `horizon.py check` | `cargo_exit: 0` inside the real `horizon-client` |
| `horizon.py test` (`game::occultism::tests`) | **19 passed**, 0 failed, 1240 of the game's own tests filtered out |
| `horizon.py build` | `cargo_exit: 0`, fast binary linked |
| `/occultism recipes` in game | 26 rules, 27 files, 59 accepted items, 1 skipped |
| `tools/e2e.sh` (dev-mod content) | **PASS + EXIT:0** — ignition, 3 feathers → exactly 3 awakened feathers, the stone control, dirt unchanged, the gold chalk → yellow, datura two blocks away ignored |
| `EMBEDDED=1` e2e + embedded control | **PASS + EXIT:0** — the control game has no datura item and no block; the main run writes 86,714 B identical to `content/occultism.hmod` |
| `SELFTEST=1` e2e (must fail) | **FAIL + EXIT:1** — the 3→2 count flip is caught |
| `tools/hmod-load.sh` (local) | **PASS + EXIT:0** — with `--dev-mod content/occultism.hmod` the datura item and `occultism:spirit_fire[color=white]` register on the shared unmodded game; unknown without it |
| `check-e2e-logs.py occultism --strict` | **5 logs, 5 ok**, 0 missing, 0 bad exits |

## Review repairs (PR #590, 2026-11-11)

Review: `ports/_shared/horizon-mods/reviews/pr-590-occultism.md` (CHANGES_REQUESTED, 4 items + source findings).

| Reviewer's item | This job |
|---|---|
| relative link `(klikli-dev/occultism)`; ignition text ends at "every datura is"; the remaining-scope section includes an internal "Skills:" list | **fixed**: link checked, description rewritten (one fire on the target, every datura consumed, chalk colouring, the four libraries table, the real gaps), the Skills line removed from the entry |
| ship the MIT notice; the root LICENSE has a stray ChargingGadgets attribution and does not reproduce upstream's `Copyright 2020 klikli-dev` | **fixed**: `LICENSE` now reads `Copyright 2020 klikli-dev (Occultism)` verbatim plus the port line, and the packager ships it in both archives |
| the sapling item model references models that do not exist | **fixed**: `otherworld_sapling_natural_simulated` and `_inventory` carried from the jar; a sweep of all 57 item models now reports no unresolved parent or override |
| the recipe cache was keyed to recipe count + first id | **fixed** (see below) |
| `tools/test_stage.py` / `tools/test_template.py` still assert one injection/one module | **open**: the fixtures are stale against a six-injection/two-module bundle; they are not used by the ladder (which runs `horizon.py` directly) and are left for the tooling slice rather than half-fixed here |

### the recipe table is rebuilt, never stale

`cached()` reused the table whenever the recipe count and first id matched, so an edited
recipe, another pack with the same ids or another world's tag set kept converting with data
that was no longer loaded. The table is a handful of spirit-fire recipes and a conversion
happens on an ignition, so `with_table()` now rebuilds it on every use — the whole class of
staleness is gone, not just the symptom.

Verified: `python3 tools/horizon.py check` in a build slot — the six-injection bundle with the
rebuilt recipe table compiles against the pin, `cargo_exit: 0`
(`ports/_shared/evidence/occultism/review-repairs-check.log`).

Artifact: `occultism.hnmod` **71,357 B** `d59cfd5dbfa5babd754572330e6983f89f3fbbcd891dfdbb5d4fcd9474e3e25b`
(content `occultism.hmod`).

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod`: the registry closed every
`.hnmod` submission with one reason, and an `.hnmod` no longer loads. This port's former
submission, **PR #834** (`occultism` 0.1.0), was closed by that purge. **0.2.0** is the same
behaviour republished as one flat SDK `.hmod`; the changelog text is at the end of this section.

Pins: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18, `package_format`
6, `content_api` "0.2"), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for 26.3. The
SDK is cached in-tree at `../_shared/sdk/0b14f8a/` and copied into `.horizon/sdk/` (gitignored)
so the build is offline. The retired tree moved to
`../_shared/evidence/occultism/retired-hnmod-0.1.0/` **before the first build** (trap 1); it is
kept as evidence, not deleted.

### File map (0.1.0 → 0.2.0)

| Old `.hnmod` layout | New SDK package |
|---|---|
| `mixins/occultism/runtime.rs` | `src/runtime.rs`, copied with `cp` and proved with `cmp` + three sha256 sums (retired file, `src/`, packaged entry) — **byte-identical**, all `108987a6…30f9d` |
| `mixins/occultism/support.rs` + 6 helper `.rs` files | `src/support.rs` (all the glue) + `src/lib.rs` (the one `impl Hooks`); no per-hook files |
| `mixins/occultism/horizon-mixin.toml` (2 `[[module]]`, 6 `[[injection]]`) | no plan: typed `Hooks` methods; the `[native]` table is the pin. The two modules were `runtime` → `src/runtime.rs` and `state` → `src/support.rs` |
| `mixins/occultism/data/occultism-content.hmod` + `embedded_content.rs` (AE2 pattern) | **gone**: one flat package carries `assets/` and `data/` itself |
| `content/` (`horizon-mod.toml`, `assets/`, `data/`, `occultism.hmod`, `carry-report.json`) | package root: `horizon-mod.toml`, `assets/` (87 files), `data/` (85 files) |
| `occultism.hnmod` (71,357 B) | `occultism.hmod` (109,133 B, 181 entries) |
| `compatibility.json`, `rust-toolchain.toml` | `[native]` in `horizon-mod.toml`; the launcher/SDK decide the toolchain |
| `Cargo.toml` (workspace + `[lib] path = mixins/…`) | `Cargo.toml` (`edition = "2024"`, one dependency: `horizon-mod-api` from `.horizon/sdk/`) |
| `tools/horizon.py`, `package.py`, `package.sh`, `test_stage.py`, `test_template.py`, `audit/`, `hmod-load.sh`, `e2e-all.sh`, `carry_content.py`, `assemble_content.py`, `notice.py`, `PLAN.md`, `AGENTS.md`, `Cargo.lock` | **retired into evidence**; the SDK ladder is `cargo test/clippy/fmt` + `cargo horizon-mod build/check/fits` |

**Kept and updated:** `tools/e2e.sh` (same scenario and assertions; the launch now takes a
launcher-built `GAME=` and an optional `CONTROL_GAME=`), `tools/parse_info.py`, `notes.md`,
`README.md`, `FINDINGS.md`, `VALIDATION.md`, `LICENSE`. `LICENSE-NOTICE.md` is new: the
provenance paragraph that used to live only in `LICENSE`, so both notices are inside the package.

`src/runtime.rs` still opens with its historical "injected as `crate::game::occultism`" header.
Byte-identity forbids editing it, so the wording stays and the module is `crate::runtime`; none
of the file's own text is behaviour.

### Helper → hook table

| 0.1.0 injection (hook, API) | 0.2.0 |
|---|---|
| `embedded-content` (`content_root`, AE2 HEAD, cancellable) | gone — the flat `.hmod` carries the content |
| `use-item-on` (`use_item_on` 13, after the block's own use) | `Hooks::right_click_block` (NeoForge's `PlayerInteractEvent.RightClickBlock`, the pre-block route upstream itself listens on) → `support::right_click` |
| `entity-tick-listened` (146) | `Hooks::entity_tick_listened` → `support::listen` |
| `entity-tick-post` (146) | `Hooks::entity_tick_post` → `support::entity_tick_post` |
| `register-commands` (8) | `Hooks::register_commands` → `support::declare_commands` (`Command`, not `NativeCommand`) |
| `perform-command` (8, split line) | `Hooks::run_mod_command` → `support::run_command` (typed `CommandInvocation`, `Flow::Cancel`) |

The game calls the old glue made, and what replaced them:

| 0.1.0 call | 0.2.0 |
|---|---|
| `game.native_recipes_of_type(RECIPE_TYPE)` | `host.recipes_of_type(RECIPE_TYPE)` (`Arc<[Recipe]>`, rebuilt per use) |
| `game.native_recipe_ingredient(&json)` | `host.recipe_ingredient(json_text)`; the object shape is reduced to a JSON array of `"id"`/`"#tag"` first |
| `game.native_recipe_result(&json)` | `host.recipe_result(json_text)`; the result's id is `stack.item().id(host)` |
| `game.native_item_entities(dim, min, max)` | `level.entities_in(host, min, max)` narrowed with `host.item_entity(..)` |
| `game.native_entity(id)` + `native_entity_block_position` | `host.entity(id)` + `EntityRef::block_position` |
| `game.native_set_item_entity(dim, id, air)` | `EntityRef::discard(host)` (an empty stack cannot be written) |
| `game.native_block(pos)` | `level.block(host, pos)` + `BlockKey::id(host)` (identity compares with `host.block_key(..)`) |
| `game.native_block_property(pos, "color")` | `level.block_property(host, pos, "color")` |
| `game.native_set_block_property` | `host.set_block_property(pos, "color", value)` |
| `game.pack.assets.registry.default_state(SPIRIT_FIRE)` | `host.block_state_id(SPIRIT_FIRE)` |
| `game.native_set_block(dim, pos, state, flags::UPDATE_ALL)` | `level.set_block(host, pos, state, BlockUpdateFlags::NEIGHBORS \| CLIENTS)` |
| `game.res.facts().get(state).replaceable` (+ bounds/loaded) | `host.block_states().replaceable(state)` via `HostBlockStates`; an absent state is "not replaceable" |
| `game.native_on(MOD_ID)` | gone: a running mod is loaded |
| `game.native_feedback(Text::literal(..))` | `host.commands().feedback(Text::literal(..))` |

`uses.json` in the package records **251 surface entries**: the eight hook ids that appear
(`right_click_block`, `entity_tick_listened`, `entity_tick_post`, `register_commands`,
`run_mod_command`, plus `command`, `ingredient` and `items` reached through the `Command`,
`Ingredient` and `ItemStack` types the glue uses), 24 methods, 55 handles, 17 traits, 127 types,
`macro:horizon_mod`, `capability:registries` and the three part families the handle types name.
See `../_shared/evidence/occultism/sdk-uses-summary.txt`.

### Traps this migration hit (the next port must know)

1. **`block_state_replaceable` is not on `Host`.** It is the `BlockStates` service:
   `host.block_states().replaceable(state)` with `use mod_api::HostBlockStates;`. Calling it on
   the host is `E0599` with a misleading "did you mean `block_state_default`" hint.
2. **`right_click_block` fires on both sides, once per hand.** Guard `HostSide::Server` before
   touching the world; the old `use_item_on` injection was server-only. The datura is consumed on
   the first hand's click, so the second hand finds none and returns `Flow::Continue`.
3. **`recipe_ingredient` takes JSON text and expands strings, not objects.** A 1.21.1 file's
   `{"item": …}` / `{"tag": …}` must be reduced to `["id"]` / `["#tag"]` first (the 0.1.0
   ingredient-shape repair, kept): `runtime::IngredientRef::reference` does the spelling and
   `support::reference_text` writes the array. `Recipe.json` is a `serde_json::Value`; use its
   methods through inference — naming the crate would add a dependency the pin refuses.
4. **`ItemStack::new` is fallible and needs an `ItemKey`.** The runtime table names results by
   id, so a conversion resolves `Identifier::parse(&id)` + `host.item_key(&id)` first.
5. **A consumed item entity is discarded, not overwritten with air.** `item_entity_set_stack`
   refuses empty stacks ("vanilla discards the item on its next tick"), so datura goes through
   `EntityRef::discard` — which is upstream's own `discard()`.
6. **`run_mod_command` replaces the split line, but the readout text must not move.** The
   commands are typed (`<pos:block_pos>` → `BlockPos`), and the three strings
   (`Spirit Fire at …`, `Spirit fire recipes: …`, both usage lines) are byte-identical to 0.1.0
   so the existing e2e assertions still match.
7. **A `&dyn Host` is needed while `host` is `&mut`.** Bind `let view: &dyn Host = &*host;` (trait
   upcast) for the recipe read and the command readout, then release it before
   `host.commands()`/the item write.
8. **`entities_in` is half-open at the maximum corner.** Upstream's `AABB(-1,-1,-1,1,1,1)`
   includes the max; this box excludes an entity exactly on it. Measured one hair narrower at
   that exact edge in 0.1.0 too (`native_item_entities`), recorded, not new.
9. **A `cargo test … | tee` pipeline hides cargo's exit status.** The first red run reported a
   lease exit of 0 while cargo had failed; run the ladder with `bash -o pipefail` (the evidence
   logs here do).
10. **`BUILD_LOCAL=1` for this port.** `build-slot.sh run` forwards `test`/`build` to the far box,
    which syncs the port to its own root, so the brief's `cd ports/occultism` does not exist
    there. The SDK ladder ran locally in a lease (as `naturesaura` did); no `--target-dir`, no
    private slots.

### Behaviour that changed with the format (deliberate, and why)

- **The ignition is pre-block now.** The 0.1.0 helper ran after the block's own use and could
  only fire where the clicked block had no use; `right_click_block` is upstream's own
  `PlayerInteractEvent.RightClickBlock`, so a lighter on datura lights the fire on **any** block,
  which is the mod's stated behaviour. The narrower rule was a port deviation, now removed.
- **The datura is discarded rather than rewritten to air** (trap 5): the same observable result,
  through vanilla's removal path.
- **The `native_on(MOD_ID)` gate is gone.** A loaded native package is on; Options > Mods still
  disables it and then no hook is called.
- **The content is in the package itself.** There is no `.horizon-cache` write, no
  `content_root` injection, and no separate `content/occultism.hmod` to keep in step. The one
  `log::warn!` in the retired helper was the embedded-content write failure; that path is gone,
  so no diagnostic is lost — the two commands are the readout, and everything else a conversion
  refuses (an unknown item, an unreadable recipe) is counted in `/occultism recipes` as skipped.

### Gaps after the migration

1. **No 0.2.0 in-game run in this environment.** The e2e needs a game built by
   `horizon-launcher native-build .`, and the launcher is not installed at this pin. The
   scenario is unchanged and still fail-capable (`SELFTEST=1`); the 0.1.0 runs in
   `../_shared/evidence/occultism/` stand as the behaviour's evidence for a `src/runtime.rs`
   that is byte-identical. `tools/e2e-all.sh` and `tools/hmod-load.sh` were retired with the
   format; `tools/e2e.sh` now takes `GAME=` and an optional `CONTROL_GAME=`.
2. **`assemble_content.py` in the old tree was a stray Charging Gadgets script**, not this
   port's carry tool (it assembled `charginggadgets` into `content/`). It is retired into
   evidence with the rest; the real carry is `carry_content.py`, and the result it produced is
   the 172 files now at the root, byte-identical to the retired content package.
3. Everything in the 0.1.0 gap list stands unchanged: the pentacle rituals, the spirits and
   their AI, GeckoLib models, the Otherworld dimension, Modonomicon, Curios, the crusher/miner/
   crystallize/spirit-trade recipes, the datura crop and `otherworld_ashes` (its only inputs are
   Otherworld logs).

### Upstream defects fixed

None found in this migration. The ingredient-shape issue was a **port** defect (the readout
called the game's parser with an object it does not take), already fixed in 0.1.0 and now carried
as trap 3.

### Changelog (0.2.0, supersedes #834)

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (launcher ≥ 0.2.70) retired
> `.hnmod` and closed **#834** (`occultism` 0.1.0); this version supersedes it. One flat
> `occultism.hmod` now carries the code *and* the content (the spirit-fire block, 41 items, 27
> recipes, 7 tags, 4 loot tables, models, textures and lang): `src/lib.rs` is the one
> `impl mod_api::Hooks`, `src/support.rs` the glue, and `src/runtime.rs` is byte-identical to
> 0.1.0, so the conversion, the count rule, the chalk colours and the click geometry behave
> exactly as before. Built against SDK `0b14f8a` / mixins `e59da90` for 26.3. The `.hnmod`, the
> mixin plan, the six per-hook helpers and the embedded-content injection are gone; the click is
> now upstream's pre-block `PlayerInteractEvent.RightClickBlock`, so ignition works on any block
> (the 0.1.0 "no block use of its own" narrowing was a port limitation); a consumed datura is
> discarded instead of rewritten to air. Same feature set and the same gaps (rituals, spirits,
> GeckoLib models, Otherworld, guide book, Curios, the machine recipes). No 0.2.0 in-game run: the
> launcher is not installed here.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

## Review fixes (2026-11-18)

`var/review-all/1241-occultism.md`, the one finding fixed; version **0.2.1** (111,169 B, sha256
`32253022e5e3e358d7832f10b7ffd86d1739521ca1cef9742e70d90662f973a9`).

- **Unbounded process-wide conversion map (src/support.rs:280, :323).** The diagnostic state is
  now `runtime::Recent`, a ring of at most `RECENT_LIMIT = 256` conversions, and each entry is
  keyed by the world's open (`LevelRef::epoch`), the dimension id **and** the position — the same
  coordinates in another dimension or an earlier world are a different fire and are never
  reported. The readout no longer prints a per-fire count (there is no unbounded counter); it
  prints the newest conversion at that key. Tests: `recent_conversions_are_bounded` (257th evicts
  the oldest) and `a_conversion_is_scoped_to_its_world_and_dimension`.
- **The in-game leg found a load crash the review could not see.** The carried
  `occultism:spirit_fire` recipe files name a serializer the 0.2.0 package never registered, so
  the game refused the whole package at load (`unregistered recipe serializer
  occultism:spirit_fire`). `Hooks::register` now declares the recipe type and registers
  `SpiritFireSerializer`, a `CraftingRecipeHandler` that admits the JSON (the port reads the raw
  recipes through the recipe service and needs no crafting-grid callback); the game then loads
  "content from 1 mod(s)" and the e2e's `/occultism recipes` reads 26 of 27 rules.
- **Scenario update for the bounded state.** `tools/parse_info.py` and the e2e no longer parse a
  `conversions=` field (gone by the fix); the assertions now pin `last=` (the newest conversion)
  and the item counts, which is the same evidence without the unbounded counter.
- `runtime.rs` gained the ring and its tests (25 tests); it is no longer byte-identical to the
  retired bundle.

Ladder: `cargo test` 25 passed; `cargo clippy --all-targets -- -D warnings` exit 0;
`cargo fmt --check` clean; `cargo horizon-mod build` twice byte-identical; `check` exit 0 and
`fits --build 0b14f8a` exit 0 (the nightly moved to `6d765e0` during this wave, whose `Item`
constructor change is a later pin; the campaign pin is `0b14f8a`). In-game: `PASS` + `EXIT:0`
(`ports/_shared/evidence/occultism-e2e/e2e-launcher.log`); `GAME=$CONTROL_GAME` → `FAIL`
(`Unknown item 'occultism:datura'`) + `EXIT:1`.

### Registry fit-gate repair (2026-11-20) — `uses.json` named a local type as an SDK type

The reviewer's finding (`levzzz5154`): the generated `uses.json` mapped the port's **local**
`IngredientRef::Item` (`src/runtime.rs:123`) to the SDK's `type:Item`, so `fits` refused the
package against any build other than the one it was made for:

```
6d765e0  does not fit: 1 item changed
    the type `Item` can no longer be built with a literal, and `src/runtime.rs:131` builds it
```

Root cause: `uses.json` is a **source-text scan** (`cargo horizon-mod check --uses`). It resolves
a name against the SDK surface by name, so the tuple variant `Item(String)` and `Self::Item(…)`
were recorded as the SDK struct's literal with `"built": "full"`, the SDK type's field list and
`built_at: src/runtime.rs:131`. `fits` refuses a `built: full` entry when the target type is not
constructible (`constructible: false`; the SDK type is `#[non_exhaustive]`). The scan is
regenerated by every build, so rebuilding at the same pin produces the same false entry; the
honest fix is to rename the local variant so the table is true. `uses.json` was not hand-edited.

Fix: `IngredientRef::Item` → `IngredientRef::ItemId` (`Self::Item` → `Self::ItemId`). The package
still uses the SDK `Item` in the glue, so `type:Item` remains — now with `"built": "none"`, which
is the truth.

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `sdk-test.log` | **25 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `cargo horizon-mod build` twice, `cmp` | — | **byte-identical** |
| package | `sha256sum; stat -c%s` | — | **110,976 B**, SHA-256 `7a528a56ae6e36d4edf5cdeca511af05493da968cabc5727a7d342971adbe340`, 181 entries, no packed `Cargo.lock` |
| package check | `cargo horizon-mod check occultism.hmod` | `sdk-check.log` | exit 0 |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` | `sdk-fits.log` | `0b14f8a  made for`, exit 0 |
| fits (the refusal build) | `cargo horizon-mod fits occultism.hmod --build 6d765e0` | `sdk-fits-next.log` | **`6d765e0  fits`, exit 0** |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| in game (modded) | `launcher-game.sh run occultism` | `e2e-default-0.2.1.log` | **PASS** + `EXIT:0` |
| in game (control) | `GAME=$CONTROL_GAME` | `e2e-control-nopackage.log` | **FAIL** + `EXIT:1` (`Unknown item 'occultism:datura'`) |
| fail-capability | `SELFTEST=1` | `e2e-selftest-0.2.1.log` | **FAIL** + `EXIT:1` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism --strict` | — | 8 ok, 0 missing-exit, 0 bad-exit |

`src/runtime.rs` SHA-256 is now `35ddee0e…6237`; only the variant's name changed, not its data or
the conversion rules.

Skills: superpowers/using-superpowers,systematic-debugging,test-driven-development,verification-before-completion; BeautifulRust/rust-best-practices

## Review fixes (2026-11-21) — 0.2.2: operator-only readout, spectator guard

Review of **#1241** at `55013216` (2026-11-20 22:35), two findings, fixed here; version **0.2.2**
(113,674 B, sha256 `35714e6fd044bf52c4d2a3c63c68dabee5a3aa898525700e5ffdbfd54bdd38e2`, 181 entries,
no packed `Cargo.lock`). The `occultism` id collision with **#1311** is a listing decision, not code:
nothing is renamed here.

| Reviewer's finding (verbatim) | This job |
|---|---|
| **`src/support.rs:292,312`:** `/occultism info` lets any player read the block id at any loaded coordinate, so a non-operator can find hidden ores or containers. Please make it operator-only. | **fixed**: `Command::op("occultism info <pos:block_pos>", 2)` **and** a body re-check of `event.source.permission().level()` (a source reaching the body by another route is refused too). The refusal names no block; a position holding no fire answers `not a spirit fire`, so the readout is not a world probe even for an operator. |
| **`src/support.rs:173,214`:** Horizon calls this hook for spectator clicks too, so a spectator can recolour or place fire and clear datura. Please check the player's game mode before changing anything. | **fixed**: `right_click` resolves the clicking player (`BlockClick::player` → `Host::player` → `PlayerRef::game_mode`) and returns before any read/write when `runtime::Mode::may_change` says no. The guard **fails closed**: an unknown player or mode changes nothing. A spectator recolours nothing, lights nothing and consumes no datura. |

The hook's own doc says the click fires "for a spectator too (the server then only lets one open a
menu)", which is the reviewer's premise; the pre-fix run below reproduces it in game.

### Red → green, unit tests (quoted before the fix)

The tests were written first; the first run failed to compile because the API they demand did not
exist (`error[E0425]: cannot find function 'info_permitted'`, `cannot find value 'NOT_A_FIRE'`,
`cannot find type 'Mode'`), so the functions were then added with **today's behaviour** as their body
(everyone permitted, the block always named, every mode may change) to make the defect a failing
assertion rather than a compile error. `ports/_shared/evidence/occultism/sdk-test-red2-0.2.2.log`:

```
---- runtime::tests::a_non_fire_readout_names_no_block stdout ----
thread 'runtime::tests::a_non_fire_readout_names_no_block' (22674) panicked at src/runtime.rs:559:9:
assertion `left == right` failed
  left: "Spirit Fire at [10, 64, 0]: block=minecraft:diamond_ore, colour=none, last=none, rules=26/27"
 right: "not a spirit fire"

---- runtime::tests::a_spectator_click_changes_nothing stdout ----
thread 'runtime::tests::a_spectator_click_changes_nothing' (22675) panicked at src/runtime.rs:583:9:
a spectator must not recolour, light or consume anything

---- runtime::tests::the_info_readout_needs_an_operator stdout ----
thread 'runtime::tests::the_info_readout_needs_an_operator' (22693) panicked at src/runtime.rs:541:9:
a plain player

---- support::tests::the_info_command_is_declared_for_operators stdout ----
thread 'support::tests::the_info_command_is_declared_for_operators' (22700) panicked at src/support.rs:407:9:
assertion `left == right` failed: a non-operator must not read a block at chosen coordinates
  left: 0
 right: 2

failures:
    runtime::tests::a_non_fire_readout_names_no_block
    runtime::tests::a_spectator_click_changes_nothing
    runtime::tests::the_info_readout_needs_an_operator
    support::tests::the_info_command_is_declared_for_operators

test result: FAILED. 25 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green after the fix: **29 passed, 0 failed** (`sdk-test.log`).

### Red → green, in game (the reviewed 0.2.1 package)

The same scenario run against the **0.2.1** export (the reviewed head) fails on both findings,
which is also the fail-capability proof for the two new legs:

| leg | 0.2.1 (red) | 0.2.2 (green) |
|---|---|---|
| spectator chalk | `e2e-red-0.2.1.log`: `FAIL: a spectator recoloured the fire to 'red' (the click must change nothing)` | `e2e-default-0.2.2.log`: `a spectator's red chalk left the fire white` |
| non-fire readout | `e2e-red2-0.2.1.log`: `FAIL: a non-fire position answered 'Spirit Fire at [200, 101, 4]: block=minecraft:air, colour=none, last=none, rules=26/27'; it must name no block` | `a position holding no fire: not a spirit fire` |

The spectator red line also proves the hook **does** fire for a spectator click in this harness
(the pre-fix build mutated the fire), so the green leg is not vacuous.

### The readout's non-operator leg

A de-opped player cannot be produced in this harness: `op`/`deop` are **dedicated-server-only**
commands (`lan/commands.rs` `DEDICATED_ONLY`), the owner console is level 4, and an integrated
server gives every LAN guest **level 2** when guest commands are on
(`lan/hub.rs::default_permission`). So the non-operator is a LAN guest with guest commands **off**
(level 0), and the guest's own command path is proven alive in the same run:

```
  the level-0 guest runs /occultism recipes: Spirit fire recipes: 26 rules, 27 files, 59 items, 1 skipped.
  as a level-0 guest: occultism info 200 101 0<--[HERE] Incorrect argument for command Spirit fire recipes: ...
```

`/occultism recipes` is a level-0 usage of the same command and answers the guest; `/occultism info`
is refused by the dispatcher and the window carries no `Spirit Fire at` / `block=` line. That pair is
the differential: the refusal is the level gate, not "commands are off for guests".

### Ladder

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `sdk-test.log` | **29 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy-0.2.2.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt-0.2.2.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | `sdk-build-0.2.2.log`, `sdk-build-second-0.2.2.log`, `sdk-reproducible-0.2.2.txt` | **byte-identical**, 113,674 B, `35714e6f…` |
| package | `sha256sum; stat -c%s; unzip -l` | `sdk-artifact-0.2.2.txt` | 113,674 B, `35714e6f…`, **181 entries**, no packed `Cargo.lock`, declares **0.2.2** |
| package check | `cargo horizon-mod check occultism.hmod` | `sdk-check-0.2.2.log` | exit 0, `occultism 0.2.2 (native-mod, … for 26.3)` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` | `sdk-fits-0.2.2.log` | `0b14f8a  made for`, exit 0 |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 6d765e0` | `sdk-fits-next-0.2.2.log` | **`6d765e0  fits`**, exit 0 |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| in game (modded) | `bash tools/e2e.sh` in a slot | `e2e-default-0.2.2.log` | **PASS** + `EXIT:0` |
| in game (+ control leg) | `CONTROL_GAME=…` (the same Nightly without the package) | `e2e-default-with-control-leg-0.2.2.log` | **PASS** + `EXIT:0`; the control game has no datura item and no block |
| in game (control) | `GAME=$CONTROL_GAME` | `e2e-control-0.2.2.log` | **FAIL** + `EXIT:1` (`the datura item did not register`) |
| fail-capability | `SELFTEST=1` | `e2e-selftest-0.2.2.log` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | — | 8 logs, 8 ok, 0 missing-exit, 0 bad-exit |

`src/runtime.rs` gained `INFO_LEVEL`, `INFO_DENIED`, `NOT_A_FIRE`, `info_permitted`, `info_line` and
the `Mode` mirror (the click guard's rule, kept free of `mod_api` types); `src/support.rs` gained
`click_mode` and the body re-check. `src/lib.rs` is unchanged.

### Parity coverage (0.2.2)

Every family this slice ships, and where it is exercised. `tested` names the phase/assertion in
`tools/e2e.sh` (or the unit test when the in-game leg covers only a representative).

| family | state |
|---|---|
| `occultism:spirit_fire` block (id, state, `color` state) | **tested** — C (block at the clicked face, `colour=white`), G (yellow), I (white → red) |
| the 27 carried `occultism:spirit_fire` recipes (26 loadable) | **tested** — B (`26 rules, 27 files, 1 skipped`), D (feathers → awakened), F (an unmatched item stays) |
| conversion count = input count (`assemble`) | **tested** — D (3 feathers → exactly 3 awakened feathers), SELFTEST flips it |
| ignition: datura + flint and steel | **tested** — C (fire lit, datura consumed) |
| ignition: fire charge | **not tested in game** — the same code path as flint and steel (`runtime::starts_fire`), unit-tested only |
| ignition reach (`AABB(-1,-1,-1,1,1,1)`) | **tested** — H (datura two blocks away: no fire, not consumed) |
| chalk recolour, 16 chalks | **tested** — G (gold → yellow), I (red), plus the 18-entry table unit test; the other 14 colours are the same table, not each clicked in game |
| `/occultism info` readout | **tested** — J (operator reads; non-fire answers `not a spirit fire`; level-0 guest refused, no readout leaked) |
| `/occultism recipes` readout | **tested** — B (host), J (level-0 guest) |
| spectator click guard | **tested** — I (chalk, lighter + datura, then the survival differential) |
| the four carried loot tables | **not tested** — they belong to `otherstone`/`otherrock`/`otherflower`/`otherworld_sapling`, which are outside the conversion loop and have no in-game assertion yet; recorded as a gap for the next slice — **closed in 0.2.4** (see §"Slice 3" below) |
| pentacle rituals, spirits/AI, GeckoLib models, Otherworld, guide book, Curios, machine recipes, datura crop | **not ported** — see the gap list above (unchanged) |

Skills: superpowers/using-superpowers,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Slice 2 (0.2.3, 2026-11-21) — the datura crop and its seed source

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`), repinned from
`0b14f8a` here (the `[native] horizon` field moved; `.horizon/sdk` was deleted so the SDK fetched
again — the pin trap). Class **content**, tier **deep** (ledger row 345). Licence **MIT**.

**Gap line it comes from** (`FINDINGS.md`, "Out (recorded gaps)"): *"the datura crop block and its
growth: `occultism:datura` is a plain item here"* — and the 0.2.2 coverage table's *"the four
carried loot tables — not tested"* is the same family's data half. Before this slice datura was
creative-only, so the whole spirit-fire loop was unreachable in survival: this is the mod's own
first step (`getting_started/demons_dream.json`).

**Shape** (logic half of a `deep` mod): carried data (`minecraft:wheat` template block with no block
item, the seeds item, the loot table, the two `c:` tags, the crop models/textures/lang) plus two rules
in `runtime.rs` and three glue points. No block entity, menu, entity or dimension.

- **The block** `occultism:datura`: `CropBlock` on the `minecraft:wheat` template (ages 0..7, farmland
  survival, random tick, instant break, CROP sounds, no block item — upstream registers it with
  `false`, exactly as `ReplantableCropsBlock` does).
- **The seed** `occultism:datura_seeds`: `minecraft:wheat_seeds` template with `"block":
  "occultism:datura"`, so it plants the crop (a creative click keeps the stack, a survival one spends
  it — vanilla's rule, asserted).
- **Growth** is the template's own `CropBlock` random tick (no code): `random_tick_speed 0` keeps the
  age, `400` ripens it to 7 — the differential the `farmersdelight`/`mysticalagriculture` ports use.
- **The harvest** (`Hooks::right_click_block`, upstream's `ReplantableCropsBlock.useItemOn` →
  `IReplantableCrops.onHarvest`): a click on a crop at age 7 rolls the block's own loot table with the
  held stack and the player as breaker (`LevelRef::block_drops` = `Block.getDrops`), writes age 0,
  drops each stack at the block's centre (`LevelRef::spawn_item`, pickup delay 10) and adds
  `EXHAUSTION_PER_HARVEST = 0.005` (`PlayerRef::add_exhaustion`). A young crop is left alone.
- **The carried loot table** `data/occultism/loot_table/blocks/datura.json` (the jar's shape in the
  26.3 dialect): pool 1 `alternatives` (age 7 → 1 `occultism:datura`, else 1 seed), pool 2 (age 7 only)
  1 seed with `apply_bonus`/`binomial_with_bonus_count`, table-level `explosion_decay`. Both branches
  are mined in game (M).
- **The seed source** (`Hooks::loot_drops`, the jar's two `add_item` loot modifiers
  `datura_seed_from_grass`/`…_from_tall_grass`): a broken `short_grass`/`tall_grass` with anything but
  shears has a 0.02 chance of one seed, drawn from the level's own random
  (`LevelRef::with_random`) in vanilla's order.

### Red → green, unit tests (verbatim, quoted before the fix)

The functions were added with today's behaviour as their body (`harvest_age` → `None`,
`seed_source` → `false`, `seed_rolls` → `false`) so the defect is an assertion, not a compile error.
`ports/_shared/evidence/occultism/sdk-test-red-0.2.3.log`:

```
---- runtime::tests::a_hand_break_of_grass_is_a_seed_source stdout ----
thread 'runtime::tests::a_hand_break_of_grass_is_a_seed_source' (162308) panicked at src/runtime.rs:679:13:
minecraft:short_grass by hand

---- runtime::tests::a_ripe_datura_is_harvested_and_a_young_one_is_not stdout ----
thread 'runtime::tests::a_ripe_datura_is_harvested_and_a_young_one_is_not' (162310) panicked at src/runtime.rs:656:9:
assertion `left == right` failed
  left: None
 right: Some("0")

---- runtime::tests::the_harvest_replants_the_crop_at_age_zero stdout ----
thread 'runtime::tests::the_harvest_replants_the_crop_at_age_zero' (162331) panicked at src/runtime.rs:670:9:
assertion `left == right` failed
  left: None
 right: Some("0")

---- runtime::tests::the_seed_roll_is_upstreams_two_percent stdout ----
thread 'runtime::tests::the_seed_roll_is_upstreams_two_percent' (162333) panicked at src/runtime.rs:705:13:
draw 0 is below the chance

test result: FAILED. 29 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green: **33 passed, 0 failed** (`sdk-test.log`).

### Ladder

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `sdk-test.log` | **33 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | `sdk-build-0.2.3.log`, `sdk-build-second-0.2.3.log`, `sdk-reproducible-0.2.3.txt` | **byte-identical**, 124,954 B, `e81ce017…` |
| package | `sha256sum; stat -c%s; unzip -l` | `sdk-artifact-0.2.3.txt` | 124,954 B, `e81ce0172de24411deefb93a3479b1c043e1e57c93c04924cb726b1ff93e5949`, **202 entries**, no packed `Cargo.lock`, declares **0.2.3** |
| tree match | rebuild + `cmp` against `occultism.hmod` | this run | `PACKAGE_MATCHES_TREE` |
| package check | `cargo horizon-mod check occultism.hmod` | `sdk-check.log` (sha printed) | exit 0, `occultism 0.2.3 (native-mod, …/0a875da…/e59da90…, for 26.3)` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da` | `sdk-fits.log` (sha printed) | `0a875da  made for`, exit 0 |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` | `sdk-fits-next.log` (sha printed) | **`0b14f8a  fits`**, exit 0 |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| in game (modded) | `bash tools/e2e.sh` (launcher-built game, `launcher-game.sh build occultism`) | `e2e-default-0.2.3.log` | **PASS** + `EXIT:0` |
| in game (control) | `GAME=$CONTROL_GAME` | `e2e-control-0.2.3.log` | **FAIL** + `EXIT:1` (`the datura item did not register`) |
| fail-capability (scenario) | `SELFTEST=1` | `e2e-selftest-0.2.3.log` | **FAIL** + `EXIT:1` (3 feathers ≠ the flipped 2) |
| fail-capability (this family) | `NEGATIVE=1` | `e2e-negative-0.2.3.log` | **FAIL** + `EXIT:1` (`negative: the ripe crop dropped 1 fruit`) |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | — | 12 logs, 12 ok, 0 missing-exit, 0 bad-exit |

### In game (the transcript, trimmed to the new phases)

```
== K: the datura crop binds ages 0..7, its seed plants it, and it grows by random tick ==
  ages 0..7 read back on occultism:datura (the wheat template's crop state)
  planted occultism:datura[age=0] on farmland
  the same click on stone left air
  control: the planted crop is still age 0 after 200 ticks at random_tick_speed 0
  the planted crop advanced from age 0 to age 7 under random ticks
== L: a ripe crop is harvested by a right click, replanted at age 0 ==
  the harvest replanted occultism:datura[age=0] and dropped 1 fruit + 3 seeds
  a young crop (occultism:datura[age=3]) stayed and dropped nothing
== M: the carried loot table: a ripe crop drops its fruit and seed, a young one a seed ==
  ripe crop drops: {"occultism:datura":1,"occultism:datura_seeds":3}
  young crop drops: {"occultism:datura_seeds":1}
  the table's two branches: age 7 -> 1 fruit + 3 seeds, age 0 -> 1 seed and no fruit
== N: short grass: a hand break can yield the jar's 2% seed; shears never do ==
  hand breaks: 25 of 25 landed, 3 datura seed(s), 0 dropped grass
  shears breaks: 25 of 25 landed, 24 dropped grass, 0 datura seed(s)
```

(L's own line in the final run: `before the assertions: fruit=1 seeds=3 (state
occultism:datura[age=0])` — the fruit and the seeds are both there, from the carried table.)

The harvest's seed count varies with the engine's `apply_bonus` read (1..6 measured on a bare hand);
the assertion is the rule (a ripe crop yields exactly one fruit and at least one seed, a young one
only a seed), not a fixed bonus. The grass round observed a seed in earlier runs of this scenario
(1 seed in 25 aimed hand breaks, twice, and 2 in 50); this run rolled none, which is why the positive
side is not asserted statistically — see the gap below.

### Measured behaviours of this pin (recorded, not worked around silently)

1. **An explosion's loot reaches `loot_drops` with no block**: 432 grass blocks destroyed by 12 TNTs
   produced 0 rolls (`context.block` is None — the blast merges loot before the hook), so the 2%
   modifier can only fire on a player's or mob's break. The e2e's positive leg therefore breaks grass
   by hand.
2. **`ctl break` mines what the crosshair covers.** Without a `look-at-block` per block only the
   blocks in view land (measured 10 of 25, and 0 of 25 after a teleport); aimed *and* teleported
   beside the block (`mine`) the first round at a fresh patch lands 25 of 25 — later rounds in the
   same spot stop landing (33 of 400 over 16 rounds), which is why each patch is used once.
3. **A control launch in the same `HORIZON_HOME` destabilises the long legs**: the main game then has
   no `logs/latest.log` of its own and stops landing breaks after ~100. The control is its own
   invocation (`GAME=$CONTROL_GAME CONTROL_GAME=`), which is what `e2e-control-0.2.3.log` is;
   `CONTROL_LEG=1` keeps the old in-run shape.
4. **Random ticks off while the ages are read back**: at the default `random_tick_speed` a freshly
   planted crop can read age 1 (measured), so K sets 0 and L/M rely on that.
5. **The crop's survival needs hydrated farmland**: the patches are laid with water at both ends and
   the growth loop re-fills `farmland[moisture=7]` each round (a dry farmland turns to dirt and takes
   the crop with it).
6. **`LevelRef::block_drops` can return `None` in the click scope** (it draws the level's random, a
   write). `harvest` therefore **leaves the crop ripe when the roll is refused** instead of resetting
   it without a yield — a later click rolls again — and the e2e retries the click while the crop still
   reads age 7. One run read `0 fruit` with the table's seeds present (a roll that saw the post-write
   state); the drop counts are now printed before the assertion so the next occurrence is diagnosable.

### Parity coverage (0.2.3)

| family | state |
|---|---|
| `occultism:datura` (crop block, ages 0..7, no block item) | **tested** — K (all eight ages read back) |
| `occultism:datura_seeds` (plants the crop on farmland, not on stone) | **tested** — K (planted `age=0`; the stone click left air) |
| crop growth by random tick | **tested** — K (`0` keeps age 0 for 200 ticks, `400` ripens to 7) |
| the crop's carried loot table, both branches | **tested** — M (age 7 → 1 fruit + seeds, age 0 → 1 seed, no fruit) |
| the replant harvest (right click at age 7, age 0 left, drops at the block) | **tested** — L (1 fruit + seeds; a young crop untouched) |
| the seed source: `short_grass`/`tall_grass` without shears | **tested (differential) + observed positive, not asserted** — N: 25 aimed hand breaks landed and yielded 3 seeds (earlier runs: 1, 1 and 2 in the same scenario); 25 shears breaks each rolled the grass's own loot (24 dropped grass) and yielded 0 seeds, which is the modifier's inversion. The 2% rule itself is `runtime::tests::the_seed_roll_is_upstreams_two_percent` + `a_hand_break_of_grass_is_a_seed_source` (red then green). A *statistical* in-game assertion stays `blocked(harness-break-reliability)`: the client lands the first round of 25 aimed breaks at a fresh patch and then stops landing them (measured 33 of 400 over 16 rounds, 12 of 400 in another), so a 400-roll proof (~0.03% for an absent hook) cannot be reproduced on demand. The hand round is therefore a reported measurement, not an assertion; the shears half is the asserted differential |
| the harvest's food exhaustion | **not asserted** — `add_exhaustion` is called, but a 0.005 exhaustion is under the food bar's rounding; the call is the code path, not evidence |
| `/occultism info`, `/occultism recipes`, spirit fire, 26 conversion recipes, chalk, spectator guard | **tested** — the 0.2.2 phases A..J, unchanged |

### Upstream defects fixed

None found in this slice. The jar's `datura` loot table, the two loot modifiers and the harvest rule
are consistent with each other and with the blockstate's 8 ages; nothing was corrected.

### Deviations (deliberate, and why)

| Area | Upstream | This port |
|---|---|---|
| arm swing on harvest | `player.swing(hand)` | none: the pin has no swing helper (`player_arm_swing` is a listen hook), so the arm does not move |
| harvest with an empty hand | `useItemOn` is called with the empty stack and harvests | same (the hook carries `Option<&ItemStack>`; `None` harvests) |
| `#c:tools/shear` | a tag | `minecraft:shears`, the tag's only member at 26.3 (no item-tag read at the pin) |
| the seed chance's source | `random_chance` 0.02 in the loot modifier | the level random through `LevelRef::with_random`, `next_float() < 0.02` |
| the `datura` item | `SpiritHealingItem` food, grants Third Eye | still a plain stack-64 item (unchanged from 0.2.2; the food effect is a later slice) |

### Next slices from this point

The seed source closes the loop: a player can now break grass → plant → harvest → light the fire.
The rest of the gap list is unchanged (pentacle rituals, spirits/AI, GeckoLib, Otherworld,
Modonomicon, Curios, the spirit-job recipes), plus the four older carried loot tables
(`otherstone`/`otherrock`/`otherflower`/`otherworld_sapling_natural`), which are still unexercised in
game and are the cheapest next slice (no code: place, break, assert the drop).

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Slice 3 (0.2.4, 2026-11-21) — the four carried block loot tables, proven in game

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`). Class
**content**, tier **deep** (ledger row 345). Licence **MIT**, ours on `main` → an
`Update occultism 0.2.4` publication.

**Gap line it comes from:** the coverage table above — *"the four carried loot tables — **not
tested** … recorded as a gap for the next slice"* — and §"Next slices from this point", which names
place/break/assert the drop as the cheapest next slice. The 2026-11-20 parity ruling makes an
unexercised carried family a gap in the work.

**Shape:** the data half of a `deep` mod. The four blocks (`data/occultism/horizon/block/*.json`),
their block items (auto-created; no `item` file is needed), the four loot tables
(`data/occultism/loot_table/blocks/*.json`, 26.3's singular typed `"condition"`) and the
`minecraft:mineable/pickaxe` tag already shipped in 0.2.0–0.2.3 and are **unchanged**. No runtime
rule, no glue, no block entity: the engine rolls the table on a player break. The slice's
deliverable is the **in-game assertion**, the coverage rows and the republished version — the
package's only byte change is `version = "0.2.4"` in `horizon-mod.toml`/`Cargo.toml`.

### The rule under test

| block | template | `requires_tool` | table (26.3 dialect) |
|---|---|---|---|
| `occultism:otherstone` | `minecraft:stone` | **true** | 1 `occultism:otherstone`, `survives_explosion` |
| `occultism:otherrock` | `minecraft:stone` | **true** | 1 `occultism:otherrock`, `survives_explosion` |
| `occultism:otherflower` | `minecraft:poppy` | false | 1 `occultism:otherflower`, `survives_explosion` |
| `occultism:otherworld_sapling_natural` | `minecraft:short_grass` | false | 1 of itself, `survives_explosion` |

The fail-capable pair for the two stone blocks is the tool: the engine's `harvest` is
`!requires_tool || correct_for_drops(held, block)` (`game.rs:6982`), so a **hand** break breaks the
block and rolls nothing, and a **wooden pickaxe** break drops exactly one. Without the
`requires_tool` gate both would drop, and the hand assertion fails. `LOOT_NEGATIVE=1` inverts the
pickaxe assertion (must be 0) and the run must exit 1: that is this family's counterfactual. The two
plants have `requires_tool: false` and drop one of themselves to a bare hand.

### Red → green

**There is no product red, and the report says so instead of inventing one.** The new leg's first
honest run — `LOOT_ONLY=1` against the **0.2.3** export, before any product change —
is green: `e2e-loot-only-0.2.3-package.log`, `PASS` + `EXIT:0`, with
`occultism:otherstone by hand: {"drops":{}}`, `… with a wooden pickaxe: {"drops":{"occultism:otherstone":1}}`,
and one each for `otherrock`, `otherflower`, `otherworld_sapling_natural`. The carried data was
already correct; the parity gap was the missing assertion.

The red this slice did produce was in the **leg**, three runs, each diagnosed to its root cause and
fixed (no product change):

1. `jq: parse error: Unmatched '}' at line 1, column 183` /
   `FAIL: line 382: 'jq -r '[(.drops // {})[]] | add // 0' <<< "${1:-{\}}"' exited 5` — a **bash
   parse trap**: a `drops_total` defined *inside* `phase_loot` with `<<<"${1:-{\}}"` hands jq
   `<json>}` (the default's `\}` is not honoured there), so jq parses the reply and then errors on
   the stray brace. Fixed by a plain `local reply=${1:-}` + `[ -n … ] || reply='{}'` and a here-string
   of `"$reply"`; `drops_of` lost its `2>/dev/null || echo 0`, which had been masking exactly this.
2. `FAIL: cannot hand-break occultism:otherstone` with the reply
   `{"error":"the player died","pos":[232,101,30]}` and the game log
   `Player85937 was slain by Enderman` — the End has no sky light, so hostile mobs spawn on the
   fresh stone floor. Phase O now sets `difficulty peaceful` and re-kills non-players first.
3. The full run then exposed **two harness flakes in the 0.2.3 phases** (both fixed, see "Harness
   defects fixed" below): phase L read `fruit=0 seeds=3`, and phase N landed `0 of 25` / `3 of 25`
   breaks.

### In game (phase O, verbatim from `e2e-default-0.2.4.log`)

```
== O: the four carried block loot tables ==
  placed occultism:otherstone at x=232
  occultism:otherstone by hand: {"block_before":"occultism:otherstone","block":"minecraft:air","drops":{}}
  occultism:otherstone with a wooden pickaxe: {"block_before":"occultism:otherstone","block":"minecraft:air","drops":{"occultism:otherstone":1}}
  placed occultism:otherrock at x=234
  occultism:otherrock by hand: {"block_before":"occultism:otherrock","block":"minecraft:air","drops":{}}
  occultism:otherrock with a wooden pickaxe: {"block_before":"occultism:otherrock","block":"minecraft:air","drops":{"occultism:otherrock":1}}
  placed occultism:otherflower at x=236 (on dirt)
  occultism:otherflower by hand: {"block_before":"occultism:otherflower","block":"minecraft:air","drops":{"occultism:otherflower":1}}
  placed occultism:otherworld_sapling_natural[uncovered=false] at x=238 (on dirt)
  occultism:otherworld_sapling_natural by hand: {"block_before":"occultism:otherworld_sapling_natural[uncovered=false]","block":"minecraft:air","drops":{"occultism:otherworld_sapling_natural":1}}
  otherstone/otherrock: hand 0, wooden pickaxe 1 each; otherflower and the natural sapling: hand 1 each
```

The control leg also asserts all four ids are **unknown** without the package (`setblock` leaves air),
so the phase proves the package and not the engine. `LOOT_ONLY=1` is the fast red/green leg for this
family (launch + arena + O); `LOOT_NEGATIVE=1` is its counterfactual.

### Ladder

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `sdk-test.log` | **33 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | `sdk-build-0.2.4.log`, `sdk-build-second-0.2.4.log`, `sdk-reproducible-0.2.4.txt` | **byte-identical**, 124,955 B, `f49ee35d…ca3cd` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | `sdk-artifact-0.2.4.txt` | 124,955 B, `f49ee35df9fc22d00217e611440573c6c771151b8cda963d252f46e7761ca3cd`, **202 entries**, no packed `Cargo.lock`, declares **0.2.4** |
| package check | `cargo horizon-mod check occultism.hmod` (sha in the log) | `sdk-check.log` | exit 0, `occultism 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da` (sha in the log) | `sdk-fits.log` | `0a875da  made for`, exit 0 |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` (sha in the log) | `sdk-fits-next.log` | **`0b14f8a  fits`**, exit 0 |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| game build | `launcher-game.sh build occultism` | `launcher-build-0.2.4.log` | **`Compiling occultism (1 of 1)`** (a cached package would have run the 0.2.3 bytes); exported `native-0a875da-3f170bd8dd15`, manifest `version = "0.2.4"` |
| in game (modded) | `launcher-game.sh run occultism` | `e2e-default-0.2.4.log` | **PASS** + `EXIT:0` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` | `e2e-control-0.2.4.log` | **FAIL** + `EXIT:1` (`the datura item did not register`) |
| fail-capability (scenario) | `SELFTEST=1` | `e2e-selftest-0.2.4.log` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) |
| fail-capability (this family) | `LOOT_ONLY=1 LOOT_NEGATIVE=1` | `e2e-negative-loot-0.2.4.log` | **FAIL** + `EXIT:1` (`negative: occultism:otherstone dropped 1 with a wooden pickaxe`) |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | — | 17 logs, 17 ok, 0 missing-exit, 0 bad-exit |

### Harness defects fixed (and why the product did not change)

Both were found by this slice's full run; each is in `tools/e2e.sh`, proved by the run above.

1. **The harvest's drops were counted in two snapshots that could both miss an item.** Phase L read
   `fruit=0 seeds=3`: the fruit had been picked up between `ctl inventory` (read first) and
   `ctl entities` (read second), so it was in neither. `count_items` is now read **first** and the
   inventory second (a pickup between them is still counted in the world), `click_block` stands
   three blocks away (outside the pickup box), and the "before the assertions" line prints the
   inventory and the item entities so the next occurrence is diagnosable. Evidence of the pickup
   itself: `probe-l-0.2.4.log` (round 2's seeds moved into the inventory at 2 blocks).
2. **`ctl break` was aimed from a stale client position.** The old `mine` teleported the player to
   `x+1` in the *same row* — inside the next grass block — and phase N stopped landing breaks
   (`0 of 25` shears, then `3 of 25` hand; the reply stream is what `mine` swallowed). `mine_patch`
   now stands two rows south of the patch (outside it, or on a row already broken), waits for the
   client to report the teleport (`wait_near`), aims, and re-teleports + re-aims up to three times.
   `probe-n-0.2.4.log`: **25 of 25** hand and **25 of 25** shears; the full run then read
   `hand breaks: 25 of 25 landed … shears breaks: 25 of 25 landed, 25 dropped grass`.

### Parity coverage (0.2.4)

| family | state |
|---|---|
| `occultism:otherstone` + its table | **tested** — O (state read back, hand 0, wooden pickaxe 1, block becomes air); the control game does not know the id |
| `occultism:otherrock` + its table | **tested** — O (same pair) |
| `occultism:otherflower` + its table | **tested** — O (placed on dirt, hand break 1) |
| `occultism:otherworld_sapling_natural` + its table | **tested** — O (placed on dirt as `[uncovered=false]`, hand break 1). Deviation: upstream drops `minecraft:oak_sapling` unless `uncovered=true`, a state this port declares but never sets; dropping itself keeps the item obtainable (recorded in the deviation table) |
| `minecraft:mineable/pickaxe` for the two stone blocks | **tested** — the pickaxe break drops and the hand break does not (the gate reads the block's tag) |
| spirit fire, the 26 conversion recipes, the crop, the seed source, the readouts, the spectator guard | **tested** — phases A..N, unchanged except the two harness fixes above |
| pentacle rituals, spirits/AI, GeckoLib models, Otherworld, guide book, Curios, machine recipes | **not ported** — the gap list above (unchanged) |

### Measured behaviours of this pin (added by this slice)

1. **`requires_tool` gates the drop, not the break**: a hand break of `otherstone` removes the block
   and rolls nothing, so a table without the gate would be *worse*, not just different — the pair is
   the assertion.
2. **Phase O must set `difficulty peaceful`**: the End's arena floor has no sky light and hostile
   mobs spawn on it (an Enderman killed the player 4 s into the first run).
3. **A `ctl break` reply can fail fast** (`{"error": …}`, non-zero exit) when the client's position
   is stale or the crosshair misses; a retry that re-teleports and re-aims is needed, and the error
   JSON must be printed — swallowing it is how 22 of 25 breaks vanished silently.
4. **`${1:-{\}}` as a here-string is unsafe inside a function defined inside a function**: bash hands
   jq the value plus a stray `}`. Use a plain local variable.
5. The 0.2.3 findings stand: an explosion's loot reaches `loot_drops` with no block at this pin, so
   these tables (and the 2% grass seed) are only provable by a player break.

### Upstream defects fixed

None. The four jar tables, the block properties and the pickaxe tag are consistent; the one
divergence (the natural sapling's `uncovered` branch) is the port's recorded deviation, not a jar
defect.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices



## Slice 4 (0.2.5, 2026-11-21) — the two crafting-table recipes, and the double harvest

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`). Class
**content**, tier **deep** (ledger row 345). Licence **MIT**, ours on `main` → an
`Update occultism 0.2.5` publication.

**Gap line it comes from:** `FINDINGS.md` "Out (recorded gaps)" names the obtainability gap the
0.2.3 coverage row records — *"the carried items are creative-only except the crop chain"* — and
the 0.2.4 report's `## Next slice` named the anchor: `demons_dream_essence_from_fruit_or_seed`
makes `occultism:demons_dream_essence` obtainable **for the first time** now that slice 2 carried
the crop and its `#c:crops/datura` / `#c:seeds/datura` tags. This is a behaviour change (new
obtainable items), so it is a real version rather than the evidence-only 0.2.4 (ruling 2026-11-21,
`AGENTS.md`).

### Which jar recipes qualify (enumerated, not guessed)

The rule: **the result and every ingredient the package (or vanilla) already provides**. Measured
against the jar's **166** `data/occultism/recipe/crafting/*.json`:

| family | files | carried? | why |
|---|---:|---|---|
| `demons_dream_essence_from_fruit_or_seed` | 1 | **carried** | result is a carried item; the key is the alternative `#c:crops/datura` / `#c:seeds/datura`, both carried tags |
| `book_of_binding_empty` | 1 | **carried** | shapeless `awakened_feather` + `purified_ink` + `taboo_book`, all carried items |
| impure chalks (`chalk_*_impure`, white twice) | 18 | **not carried** | results are carried, but every one needs an ingredient the package does not carry (`chalk_white_impure` needs `#c:dusts/otherworld_wood` and `#occultism:dusts/chalk_base`, both empty here; the colour ones add `marid_essence`, `cruelty_essence`, `nature_paste`, `afrit_essence`, `demonic_meat`, `cursed_honey`, `gray_paste` or empty `c:dusts/*` tags) |
| the `otherstone` / `otherrock` **storage (building) family** — slabs, stairs, walls, buttons, pressure plates, bricks, polished and chiseled variants, the two cobblestone families, frames, pedestals, tablets, sacrificial bowls, storage controllers | **38** | **not carried** | every ingredient is carried, but the **result** is not: this slice ships no `otherstone_slab`, `otherrock_bricks`, `polished_otherstone`, `othercobblestone`, `sacrificial_bowl`… item. A recipe whose result is unregistered is dead data, so the family stays a gap until those blocks are carried |
| every other crafting file (silver/iesnium/tallow/netherite, candles, lamps, books of calling, wood family, dyes, …) | remainder | **not carried** | result and/or ingredients are metals, woods, pastes and lens items this port does not carry |

So **two** of the 19 crafting recipes whose result is carried qualify, and the file set grows from
202 to 204 entries. The two files are the jar's own shapes written in the **26.3 dialect**: an
ingredient is a bare `"occultism:…"` / `"#c:…"` string, an alternative is a **list of strings**, and
a key's value is never a `{"item": …}` / `{"tag": …}` object. That is not cosmetic — the 26.3
loader's `Ingredient::parse` drops an object without a `type` (`horizon/crates/horizon-client/src/
crafting.rs:1421`), so a byte-for-byte carry registers **no recipe at all** (the
`compactmachines`/`enderio` finding of 2026-11-18). The trap is pinned by a unit test.

### The one-click/one-harvest fix (a port defect the full run found)

The slice's first full run failed phase L with `fruit=2 seeds=6` for one click: the crop harvest ran
**twice**. `Hooks::right_click_block` is called **once per hand** (its own doc says so), where
vanilla calls the block's `useItemOn` for the main hand and reaches the off hand only when the main
hand's action did not consume the click. The harvest writes the block, so the off hand's turn
harvested the crop the main hand had just replanted — one click, two loot rolls, two seeds stacks.
The guard is `runtime::ClickHand::may_harvest` (main hand only), mapped from `mod_api::Hand` in
`support::hand`. It was a **port** defect (present since the harvest landed in 0.2.3), not an
upstream one; without it a player could farm the crop at double yield.

### Red → green

**Unit red 1 — the dialect** (`sdk-test-red-0.2.5.log`), with the jar's bytes carried verbatim:

```
---- runtime::tests::the_carried_crafting_recipes_are_in_the_26_3_dialect stdout ----
thread 'runtime::tests::the_carried_crafting_recipes_are_in_the_26_3_dialect' (645252) panicked at src/runtime.rs:731:17:
demons_dream_essence_from_fruit_or_seed carries the 1.21.1 object-form ingredient "tag":, which the 26.3 loader drops: {
  "type": "minecraft:crafting_shaped",
…
test result: FAILED. 33 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green after the dialect rewrite: **35 passed** (the two recipe tests + the 33 from 0.2.4).

**Unit red 2 — the harvest hand** (`sdk-test-red-hand-0.2.5.log`), written with today's
`may_harvest` returning `true` for both hands so the defect is an assertion, not a compile error:

```
---- runtime::tests::only_the_main_hand_harvests_a_crop stdout ----
thread 'runtime::tests::only_the_main_hand_harvests_a_crop' (2402044) panicked at src/runtime.rs:677:9:
the off hand's turn must not harvest the crop the main hand replanted
test result: FAILED. 35 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green after the guard: **36 passed, 0 failed** (`sdk-test.log`).

**In-game red — the family** (`e2e-craft-red-0.2.4-package.log`, the **0.2.4** export, `CRAFT_ONLY=1`):

```
== P: the two carried crafting recipes in a real crafting table ==
  survival, table at 222,101,38
CRAFT FAIL: the 3x3 fruit grid made None x0, expected occultism:demons_dream_essence x1
FAIL: the crafting recipe check failed (see the CRAFT FAIL line above)
EXIT:1
```

The 0.2.4 package has the item but no recipe, so the grid yields nothing — the exact behaviour the
slice changes. The 0.2.5 package, same leg: `PASS` + `EXIT:0` (`e2e-craft-0.2.5.log`).

**In-game red — the double harvest** was the first full 0.2.5 run
(`ports/_shared/evidence/occultism-e2e/`, the run that ended `FAIL: the ripe crop dropped 2 fruit
(and 6 seeds), expected exactly 1`); the green line after the guard is
`before the assertions: fruit=1 seeds=3` in `e2e-default-0.2.5.log`.

### In game (phase P, verbatim from `e2e-craft-0.2.5.log`)

```
== P: the two carried crafting recipes in a real crafting table ==
  survival, table at 222,101,38
  the 3x3 fruit grid: the result slot reads occultism:demons_dream_essence x1
  the essence: taken into the hotbar (inventory {"occultism:demons_dream_essence": 1})
  the fruit grid: every ingredient was consumed ({"occultism:demons_dream_essence": 1})
  the 3x3 seed grid: the result slot reads occultism:demons_dream_essence x1
  the seed-grid essence: taken into the hotbar (inventory {"occultism:demons_dream_essence": 1})
  the seed grid: every ingredient was consumed ({"occultism:demons_dream_essence": 1})
  the mixed 5 fruit + 4 seed grid: the result slot reads occultism:demons_dream_essence x1
  the mixed-grid essence: taken into the hotbar (inventory {"occultism:demons_dream_essence": 1})
  the mixed grid: every ingredient was consumed ({"occultism:demons_dream_essence": 1})
  nine dirt: the result slot stayed empty
  eight fruit with one dirt in the last cell: the result slot stayed empty
  the shapeless book: the result slot reads occultism:book_of_binding_empty x1
  the book: taken into the hotbar (inventory {"occultism:book_of_binding_empty": 1})
  the book recipe: every ingredient was consumed ({"occultism:book_of_binding_empty": 1})
```

The driver (`tools/craft_recipes.py`) opens a **real** crafting table (`ctl interact`), fills the
3x3 grid one item per cell through the screen's slots (`100..108`, result `200`), reads the result,
takes it, and reads the inventory back; the player is in **survival** so consumption is the vanilla
rule. The differential is two-sided: nine dirt make nothing **and** eight fruit with one dirt in the
ninth cell make nothing, so the result is the recipe and not "any 3x3". `CRAFT_NEGATIVE=1` inverts
the fruit assertion and the run must fail (`e2e-negative-craft-0.2.5.log`:
`CRAFT FAIL: negative: the 3x3 fruit grid made occultism:demons_dream_essence x1, expected nothing`,
`EXIT:1`).

### Ladder

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `sdk-test.log` | **36 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | `sdk-build-0.2.5.log`, `sdk-build-second-0.2.5.log`, `sdk-reproducible-0.2.5.txt` | **`REPRODUCIBLE`**, 127,502 B |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | `sdk-artifact-0.2.5.txt` | 127,502 B, `658ed0f7a8ae06c06ae6b786930ac3d4124027631b2935791e3506cdf315e4e4`, **204 entries** (both recipes packed), no packed `Cargo.lock`, declares **0.2.5** |
| runtime inside/outside | `unzip -p … src/runtime.rs \| sha256sum` vs `sha256sum src/runtime.rs` | `sdk-artifact-0.2.5.txt` | identical, `e6fca922217b…` |
| package check | `cargo horizon-mod check occultism.hmod` | `sdk-check.log` | exit 0, `occultism 0.2.5 (native-mod, …/0a875da…/e59da90…, for 26.3)` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da` | `sdk-fits.log` | `0a875da  made for`, exit 0 |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` | `sdk-fits-next.log` | **`0b14f8a  fits`**, exit 0 (first attempt hit a GitHub `Connection reset by peer`, exit 2; the retry is the recorded line) |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| game build | `launcher-game.sh build occultism` | `launcher-build-0.2.5.log` | **`Compiling occultism (1 of 1)`** after removing the stale `native-0a875da-…` folder and export (the cached-package trap), tag `native-0a875da-81d30b2bded4`, manifest `version = "0.2.5"` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen, launcher-built game) | `e2e-default-0.2.5.log` | **PASS** + `EXIT:0` (A..P, including the harvest at 1 fruit and phase P) |
| in game (crafting leg) | `CRAFT_ONLY=1` | `e2e-craft-0.2.5.log` | **PASS** + `EXIT:0` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | `e2e-control-0.2.5.log` | **FAIL** + `EXIT:1` (`the datura item did not register`) |
| fail-capability (scenario) | `SELFTEST=1` | `e2e-selftest-0.2.5.log` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) |
| fail-capability (this family) | `CRAFT_ONLY=1 CRAFT_NEGATIVE=1` | `e2e-negative-craft-0.2.5.log` | **FAIL** + `EXIT:1` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | — | 23 logs, 23 ok, 0 missing-exit, 0 bad-exit |

### Parity coverage (0.2.5)

| family | state |
|---|---|
| `demons_dream_essence_from_fruit_or_seed` | **tested** — P (3x3 fruit, 3x3 seed, 5 fruit + 4 seeds each make exactly 1 `occultism:demons_dream_essence`, every ingredient consumed; 9 dirt and 8 fruit + 1 dirt make nothing) |
| `book_of_binding_empty` (shapeless) | **tested** — P (1 book, its three items consumed) |
| the 17 impure-chalk craftings and the 38-recipe otherstone/otherrock building family | **not carried** — results or ingredients are not in the package (table above); a gap for a slice that carries those items |
| the datura harvest (one click, one roll) | **tested** — L (`before the assertions: fruit=1 seeds=3`); the double harvest it caught is fixed and pinned by `only_the_main_hand_harvests_a_crop` |
| spirit fire, the 26 conversion recipes, the crop, the seed source, the readouts, the spectator guard, the four loot tables | **tested** — phases A..O, unchanged |
| pentacle rituals, spirits/AI, GeckoLib models, Otherworld, guide book, Curios, machine recipes | **not ported** — the gap list above (unchanged) |

### Measured behaviours of this pin (added by this slice)

1. **A crafting-menu phase needs `--offscreen`, not `--headless`.** Under `--headless` there is no
   renderer, so `ctl screen` never answers and `ctl click-slot` answers "no renderer yet"; the first
   phase-P run timed out on `screen` (measured 2026-11-21). The whole leg now launches offscreen;
   phases A..O are unchanged by it.
2. **The End's arena floor breeds hostile mobs during a long survival phase.** Two rounds of 25
   aimed breaks take minutes, and an Enderman killed the player mid-phase-N, leaving the **death
   screen** open: every later `ctl break` answered `{"error":"the dead screen is open; close it
   first"}`, so 9 and 11 of 25 landed (measured twice). Phase N now sets `difficulty peaceful`,
   kills non-players per patch and closes a stale screen; both rounds then landed 25 of 25.
3. **Phase N's diagnostics were being swallowed.** `reply=$(mine_patch … 2>/dev/null)` discarded the
   failure reply, which is how the death screen stayed invisible for two runs and how the e2e guard
   saw 900 s of silence and killed the leg. stderr now passes through, and each round takes up to
   three **fresh** patches (the client drops aimed breaks as a session goes on) — the 20-of-25
   landing threshold is unchanged.
4. **A phase that opens a menu must close it before any block action**, and a lost planting click
   needs a retry: phase K's plant now retries while the block still reads air (one run read
   `the seeds planted 'minecraft:air'` under load).

### Upstream defects fixed

None found. The 26.3 dialect rewrite is a *platform* translation (the jar's files are valid 1.21.1),
and the double harvest was a **port** defect (above), not a jar one.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Slice 5 (0.2.6, 2026-11-22) — the six `otherstone`/`otherrock` building blocks

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`). Class
**content**, tier **deep** (ledger row 345). Licence **MIT**, ours on `main` → an
`Update occultism 0.2.6` publication.

**Gap line it comes from:** the 0.2.5 coverage table — *"the 17 impure-chalk craftings and the
38-recipe otherstone/otherrock building family — **not carried** … every ingredient is carried, but
the **result** is not"* — and the 0.2.5 `## Next slice`, which named the first tier of that family:
the six slab/stair/wall blocks whose recipe result was the missing item. Slice 5 carries the
**results**, so the first six of those 38 recipes become real, and the blocks themselves become
obtainable and placeable.

**Shape:** the data half of a `deep` mod, exactly as slices 3 and 4. No runtime rule, no glue, no
block entity: the six blocks copy vanilla's slab/stair/wall templates (their states, placement,
shapes and the `requires_tool` gate come from the template rows), the engine rolls the loot tables
on a player break, and the crafting table matches the recipes. The slice's product change is six
blocks, six block items, six recipes, six loot tables and the models/blockstates/textures/lang/tags
they need.

### What was carried (and from where)

| family | files | source |
|---|---|---|
| block declarations | 6 (`data/occultism/horizon/block/<name>.json`) | port-authored from `OccultismBlocks` (read at tag `release/v1.21.1-1.225.0`) |
| recipes | 6 (`.../recipe/crafting/<name>.json`) | the jar's `building_blocks/crafting/*` rewritten to the 26.3 dialect |
| loot tables | 6 (`.../loot_table/blocks/<name>.json`) | the jar's five tables + **the one the jar does not have** (`otherrock_slab`) |
| blockstates | 6 | the jar's (unchanged format) |
| block models | 16 | the jar's (slab + slab_top, stairs + inner + outer, wall post + side + side_tall, per stone) |
| item models | 6 | the jar's |
| textures | 0 | reuse the carried `occultism:block/otherstone` / `otherrock` |
| lang | 6 keys | the jar's `en_us` (`Otherstone Slab`, `Otherrock Wall`, …) |
| tags | `minecraft:mineable/pickaxe` (+6), plus new `minecraft:walls`, `minecraft:slabs`, `minecraft:stairs` | the jar's datagen output |
| creative tab | 6 entries | the jar's `OccultismCreativeModeTabs` order |

### The rule under test

| block | template | hardness / resistance | `requires_tool` | recipe |
|---|---|---|---|---|
| `otherstone_slab` | `minecraft:stone_slab` | 1.5 / 30 | **true** | `ooo` (middle row) → 6 |
| `otherstone_stairs` | `minecraft:stone_stairs` | 1.5 / 1.5 | **true** | `#  `/`## `/`###` → 4 |
| `otherstone_wall` | `minecraft:cobblestone_wall` | 1.5 / 1.5 | **true** | `ooo`/`ooo` → 6 |
| `otherrock_*` | the same three | the same | **true** | the same, `otherrock` |

Upstream (`OccultismBlocks.java:158-215`): the slab is `new SlabBlock(Properties.ofFullCopy(OTHERSTONE))`
(hardness 1.5, resistance 30, `requiresCorrectToolForDrops`), the stair and wall are
`Properties.of().strength(1.5f).requiresCorrectToolForDrops()` (hardness and resistance 1.5). All six
are pickaxe-gated.

**The brief's premise was wrong, and the artefact wins.** The brief asked for a hand break of a slab
that still drops it (`requires_tool` false) as the differential. The measured facts say otherwise:
`minecraft:stone_slab`, `minecraft:stone_stairs` and `minecraft:cobblestone_wall` all carry
`requires_tool: true` in the prepared 26.3 facts, and upstream's `requiresCorrectToolForDrops()` says
the same. A hand break therefore breaks the block and rolls **nothing**; the fail-capable pair is the
same one slice 3 used for `otherstone`/`otherrock` (hand 0, wooden pickaxe 1), and the engine's
`harvest` gate is what makes the hand assertion fail if the gate were lost. Recorded, not worked
around.

### Upstream defect fixed

**`otherrock_slab` has no loot table in Occultism 1.225.0.** The jar carries
`data/occultism/loot_table/blocks/{otherstone_slab,otherstone_stairs,otherstone_wall,otherrock_stairs,otherrock_wall}.json`
and the polished/bricks/cobblestone/planks slabs — but not `otherrock_slab.json`, although
`OccultismBlocks.OTHERROCK_SLAB` is registered with `LootTableType.CUSTOM` (so nothing generates a
default table). At 1.225.0 the block would drop **nothing** at all.

- **Proved** by the jar listing (`unzip -l … | grep loot_table/blocks | grep slab` shows every other
  slab and not this one) and by the red run below, where the test that every building block has a
  table fails on that exact path.
- **Fixed** minimally: `ports/occultism/data/occultism/loot_table/blocks/otherrock_slab.json`, the
  same table as `otherstone_slab` (itself with the `type=double` → 2 branch).
- **Proved fixed in game**: the wooden-pickaxe break drops exactly one `occultism:otherrock_slab`
  (with no table it would drop nothing — the hand/pickaxe pair would both be 0).
- **User-visible**: the slab is obtainable, instead of vanishing on break.

Worth an upstream report (a one-file datagen omission); not filed — the campaign files one only on
request.

### Red → green

**Unit red 1 — the dialect** (`sdk-test-red-0.2.6.log`), with the jar's six recipes and five loot
tables carried **byte for byte** first:

```
---- runtime::tests::the_building_recipes_are_the_jars_shapes_in_the_26_3_dialect stdout ----
thread 'runtime::tests::the_building_recipes_are_the_jars_shapes_in_the_26_3_dialect' (1840010) panicked at src/runtime.rs:845:17:
otherstone_slab carries the 1.21.1 object-form ingredient "item":, which the 26.3 loader drops: {
…
test result: FAILED. 38 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The second failure in that run is the loot dialect (`"functions"` instead of `"modifier"`). Both are
the same defect class as 0.2.5's recipes: the 26.3 loaders read a specific shape and drop the other
without a word (loot: `horizon-items/src/items.rs:1608/1717` reads `entry["modifier"]`, and a
function's `type` **or** legacy `function` inside it; the 1.21.1 `functions` list is never read).

**Unit red 2 — in the test, not the product** (`sdk-test-red2-0.2.6.log`): my first loot test also
demanded a `"modifier"` list in *every* table, which 26.3 does not (vanilla's `cobblestone_wall`
table is a bare item). Fixed the assertion to "no 1.21.1 `function`/`functions` shape", which is the
real rule.

**Unit red 3 — the upstream defect** (`sdk-test-red3-0.2.6.log`):

```
thread 'runtime::tests::every_building_block_has_a_26_3_loot_table_that_drops_itself' (1876551) panicked at src/runtime.rs:831:59:
…/data/occultism/loot_table/blocks/otherrock_slab.json: No such file or directory (os error 2)
test result: FAILED. 39 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Green after the table is added: **40 passed, 0 failed** (`sdk-test.log`; the four new tests are the
dialect, the loot/self-drop, the template properties and the state/model/tag/name set).

**In-game red — the family** (`e2e-building-red-0.2.5-package.log`, the **0.2.5** export,
`BUILDING_ONLY=1`):

```
== Q: the six otherstone/otherrock building blocks ==
BUILDING FAIL: the otherstone slab grid made None x0, expected occultism:otherstone_slab x6
FAIL: the building block check failed (see the BUILDING FAIL line above)
EXIT:1
```

The 0.2.5 package has the ingredient but no result, so the grid yields nothing: the exact behaviour
this slice changes. The 0.2.6 package, same leg: `PASS` + `EXIT:0` (`e2e-building-0.2.6.log`).

### In game (phase Q, verbatim from `e2e-default-0.2.6.log`, abridged to the assertions)

```
== Q: the six otherstone/otherrock building blocks ==
  the otherstone slab grid: the result slot reads occultism:otherstone_slab x6
  the otherstone_slab recipe: every ingredient was consumed ({"occultism:otherstone_slab": 6})
  the otherstone stairs grid: the result slot reads occultism:otherstone_stairs x4
  the otherstone wall grid: the result slot reads occultism:otherstone_wall x6
  the otherrock slab grid: the result slot reads occultism:otherrock_slab x6
  the otherrock stairs grid: the result slot reads occultism:otherrock_stairs x4
  the otherrock wall grid: the result slot reads occultism:otherrock_wall x6
  three dirt in the middle row: the result slot stayed empty
== the six blocks: default state, hand 0, wooden pickaxe 1 ==
  placed occultism:otherstone_slab[type=bottom,waterlogged=false] at x=226
  occultism:otherstone_slab by hand: {"block_before": "occultism:otherstone_slab[type=bottom,waterlogged=false]", "block": "minecraft:air", "drops": {}}
  occultism:otherstone_slab with a wooden pickaxe: {"block_before": "occultism:otherstone_slab[type=bottom,waterlogged=false]", "block": "minecraft:air", "drops": {"occultism:otherstone_slab": 1}}
  placed occultism:otherstone_stairs[facing=north,half=bottom,shape=straight,waterlogged=false] at x=228
  occultism:otherstone_stairs by hand: {"…", "block": "minecraft:air", "drops": {}}
  occultism:otherstone_stairs with a wooden pickaxe: {"…", "block": "minecraft:air", "drops": {"occultism:otherstone_stairs": 1}}
  placed occultism:otherstone_wall[east=none,north=none,south=none,up=true,waterlogged=false,west=none] at x=230
  occultism:otherstone_wall by hand: {"…", "block": "minecraft:air", "drops": {}}
  occultism:otherstone_wall with a wooden pickaxe: {"…", "block": "minecraft:air", "drops": {"occultism:otherstone_wall": 1}}
  placed occultism:otherrock_slab[type=bottom,waterlogged=false] at x=232 … (same pair)
  placed occultism:otherrock_stairs[facing=north,half=bottom,shape=straight,waterlogged=false] at x=234 … (same pair)
  placed occultism:otherrock_wall[east=none,north=none,south=none,up=true,waterlogged=false,west=none] at x=236 … (same pair)
  wall pair: occultism:otherstone_wall[east=low,…] / occultism:otherstone_wall[…,west=low]
  double slab: the pickaxe break dropped 2 (the jar table's double branch)
```

The driver (`tools/building_recipes.py`) opens a **real** crafting table (`ctl interact`), fills the
grid one item per cell through the screen's slots and reads the result; the player is in survival, so
consumption is vanilla's rule. Each block is then placed by `setblock`, its **whole default state**
read back (the template's `type`/`facing`/`half`/`shape`/`up`/`waterlogged`, not a plain block), a
bare hand breaks it (`drops: {}`), and a wooden pickaxe breaks it (`drops: {<id>: 1}`) — with the
`ctl break` reply's `.drops`, never a later inventory read. Two walls side by side show
`west=low`/`east=low`, which is the `minecraft:walls` tag the block needs to connect at all. The
double slab is set to `type=double` and drops **2**, the jar table's branch.

Differentials: three dirt in the same middle row make nothing; a hand break of every block drops
nothing while the pickaxe drops one (the `requires_tool` gate). `BUILDING_NEGATIVE=1` asserts the
slab grid makes nothing and the run exits 1 (`e2e-negative-building-0.2.6.log`:
`BUILDING FAIL: negative: the slab grid made occultism:otherstone_slab x6, expected nothing`). The
control game does not know any of the six ids (`e2e-control-ids-0.2.6.log`).

### Ladder

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `sdk-test.log` | **40 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | `sdk-build-0.2.6.log`, `sdk-build-second-0.2.6.log` | **`REPRODUCIBLE`**, 145,445 B, `7b382835c0b9b7325809c4b542d803c47c0f617865b5d51168aefd828120992c` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | `sdk-artifact-0.2.6.txt` | 145,445 B, **253 entries**, no packed `Cargo.lock`, declares **0.2.6**, `src/runtime.rs` inside = outside (`fea1d79d…`) |
| tree match | a third build from the final tree + `cmp` | `sdk-artifact-0.2.6.txt` | **`PACKAGE_MATCHES_TREE`** — the delivered file is the tree's build, `7b382835…0992c` |
| package check | `cargo horizon-mod check occultism.hmod` | `sdk-check.log` | exit 0, `occultism 0.2.6 (native-mod, …/0a875da…/e59da90…, for 26.3)` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` | `sdk-fits.log` | `0a875da  made for`, exit 0 |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` | `sdk-fits-next.log` | **`0b14f8a  fits`**, exit 0 |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| game build | `launcher-game.sh build occultism` | `launcher-build-0.2.6.log` | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-726d4571fa66`, manifest `version = "0.2.6"` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen, launcher-built game) | `e2e-default-0.2.6.log` | **PASS** + `EXIT:0` (A..Q) |
| in game (building leg) | `BUILDING_ONLY=1` | `e2e-building-0.2.6.log` | **PASS** + `EXIT:0` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | `e2e-control-0.2.6.log` | **FAIL** + `EXIT:1` (`the datura item did not register`) |
| in game (control ids) | `CONTROL_LEG=1 GAME=$CONTROL_GAME CONTROL_GAME=$CONTROL_GAME` | `e2e-control-ids-0.2.6.log` | **FAIL** + `EXIT:1`, after `none of the six building blocks either` |
| fail-capability (scenario) | `SELFTEST=1` | `e2e-selftest-0.2.6.log` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) |
| fail-capability (this family) | `BUILDING_ONLY=1 BUILDING_NEGATIVE=1` | `e2e-negative-building-0.2.6.log` | **FAIL** + `EXIT:1` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | — | 30 logs, 30 ok, 0 missing-exit, 0 bad-exit |

The `fits` runs used the **full** commit SHAs: the short names (`--build 0a875da`) resolve through
GitHub's releases list, which answered HTTP 403 (the unauthenticated 60/hour limit, exhausted by the
shared machine) on 2026-11-22; a full SHA resolves from `.horizon/cache/` without the network. The
hashes in each log are the delivered package's.

### Parity coverage (0.2.6)

| family | state |
|---|---|
| `otherstone_slab`, `otherstone_stairs`, `otherstone_wall` + their recipes | **tested** — Q (each recipe's grid, count and consumption; default state read back; hand 0 / wooden pickaxe 1 drop) |
| `otherrock_slab`, `otherrock_stairs`, `otherrock_wall` + their recipes | **tested** — Q (same six assertions; `otherrock_slab`'s table is the port's fix for the jar's omission) |
| the slab tables' `type=double` branch | **tested** — Q (double slab drops 2) |
| `minecraft:walls` connection | **tested** — Q (two walls side by side read `west=low`/`east=low`) |
| `minecraft:mineable/pickaxe`, `slabs`, `stairs`, `walls`, lang, blockstates, item models | **tested** — the unit test reads each file and tag; in game the pickaxe break is the tag's proof |
| the 32 other building recipes (bricks, polished, chiseled, cobblestone families, frames, pedestals, tablets, sacrificial bowls, controllers) | **not carried** — their results are still unregistered; the next tier of the family |
| the 17 impure-chalk craftings | **not carried** — ingredients missing (unchanged) |
| spirit fire, the 26 conversion recipes, the crop, the seed source, the readouts, the spectator guard, the four loot tables, the two 0.2.5 recipes | **tested** — phases A..P, unchanged |
| pentacle rituals, spirits/AI, GeckoLib models, Otherworld, guide book, Curios, machine recipes | **not ported** — the gap list above (unchanged) |

### Measured behaviours of this pin (added by this slice)

1. **Two loot engines exist in the tree**: `horizon-world/src/loot.rs` (the older one, without
   `match_block`) and `horizon-items/src/items.rs`, which is what `block_drops_by_player` uses and
   which **does** read `modifier`, `type`/`function` and `match_block`/`block_state_property`. Reading
   the wrong one almost made this report claim a false gap; the in-game double slab (2 drops) settled
   it.
2. **`blockstates`, block models and item models are format-stable** across 1.21.1 → 26.3: the jar's
   six blockstates, sixteen block models and six item models were carried unchanged, and the game
   renders the states the blockstate names.
3. **A mod wall needs `minecraft:walls` to connect**: the engine's `WallBlock.connectsTo` reads the
   block tag, so a wall outside it stays `none` on every side even next to another wall.
4. The 0.2.4/0.2.5 findings stand (peaceful in the End, the break reply as the drop assertion, a
   fresh table per grid, `--offscreen` for menu phases).

### Deviations (deliberate, and why)

| Area | Upstream | This port |
|---|---|---|
| map colour of the stairs and walls | `Properties.of()` leaves `MapColor.NONE` | the template's stone colour (a map shows them as stone, not as the default) |
| `otherrock_slab` table | missing from the jar | shipped (the upstream-defect fix above) |
| double-slab drop | the table's `type=double` branch | the same, proven in game |

One packaging note: `README.md` is packed, and the 0.2.6 artifact was built before this section and the
README were written. Rather than ship a package that drifts from the tree, the tree's README was
restored to the packed bytes, and a third build proves `PACKAGE_MATCHES_TREE` (same
`7b382835…0992c`). So the packed README is the 0.2.5 text — user-facing and slightly stale — while the
0.2.6 story is in the manifest `description` (packed, and what the registry/launcher shows) and in this
file. The README is refreshed at the next rebuild.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

### Next slice

The **next tier of the same family**: `otherstone_bricks`, `otherrock_bricks` and their slab/stair/wall
(`data/occultism/recipe/crafting/otherstone_bricks*.json`, `otherrock_bricks*.json`), which are the
same shape as this slice — four plain-block results (bricks, chiseled, cracked) plus their stone
variants — with one new wrinkle: their recipes are made *from* `otherstone`/`otherrock` again, so
nothing new has to be carried except the results. The in-game assertion is the same six-block
pattern: craft each brick family, place it, read `occultism:otherstone_bricks` back, break it with a
wooden pickaxe and assert one in `.drops`; the `chiseled`/`cracked` pair additionally lets the slice
prove a **two-step** chain (stone → bricks → chiseled) in one grid. Tier fits: still `data` inside a
`deep` mod. Gap line: the same 0.2.5 coverage row, now 32 of 38 recipes left.

## Slice 6 (0.2.7, 2026-11-22) — the bricks tier of the building family

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`). Class
**content**, tier **deep** (ledger row 345). Licence **MIT**, ours on `main` → an
`Update occultism 0.2.7` publication.

**Gap line it comes from:** the 0.2.5 coverage table — *"the 17 impure-chalk craftings and the
38-recipe otherstone/otherrock building family — **not carried**"* — and the 0.2.6 parity row
*"the 32 other building recipes (bricks, polished, chiseled, cobblestone families, frames, pedestals,
tablets, sacrificial bowls, controllers) — **not carried** … the next tier of the family"*, whose
`## Next slice` named this exact tier.

**Shape:** the data half of a `deep` mod, exactly as slices 3–5. No runtime rule, no glue, no block
entity: twelve blocks copy vanilla's stone/slab/stair/wall templates (states, placement, shapes and
the `requires_tool` gate come from the template rows), the engine rolls the loot tables on a player
break, the crafting table matches the ten grids and a vanilla furnace runs the two smelting recipes.

### What was carried (and from where)

| family | files | source |
|---|---|---|
| block declarations | 12 (`data/occultism/horizon/block/<name>.json`) | port-authored from `OccultismBlocks` (tag `release/v1.21.1-1.225.0`) |
| crafting recipes | 10 (`.../recipe/crafting/<name>.json`) | the jar's `building_blocks/crafting/*` rewritten to the 26.3 dialect |
| smelting recipes | 2 (`.../recipe/smelting/cracked_*.json`) | the jar's `smelting/cracked_*` in the 26.3 dialect |
| loot tables | 12 (`.../loot_table/blocks/<name>.json`) | the jar's eleven + **the one the jar does not have** (`otherrock_bricks_slab`) |
| blockstates | 12 | the jar's (unchanged format) |
| block models | 22 | the jar's (bricks, slab + slab_top, stairs + inner + outer, wall post + side + side_tall, chiseled, cracked, per stone) |
| item models | 12 | the jar's |
| textures | 6 | the jar's `block/{otherstone,otherrock,chiseled_*,cracked_*}_bricks.png` |
| lang | 12 keys | the jar's `en_us` (`Otherstone Bricks`, `Cracked Otherrock Bricks`, …) |
| tags | `minecraft:mineable/pickaxe` (+10), `walls`/`slabs`/`stairs` (+2 each) | the jar's datagen output |
| creative tab | 12 entries | after the six 0.2.6 building blocks, in the jar's family order |

### The rule under test

| block | template | hardness / resistance | `requires_tool` | recipe |
|---|---|---|---|---|
| `otherstone_bricks`, `chiseled_*`, `cracked_*` | `minecraft:stone` | 1.5 / 30 | **true** | bricks `oo`/`oo` → 4; chiseled two brick slabs → 1; cracked by smelting |
| `otherstone_bricks_slab` | `minecraft:stone_slab` | 1.5 / 30 | **true** | `ooo` (middle row) → 6 |
| `otherstone_bricks_stairs` | `minecraft:stone_stairs` | 1.5 / 1.5 | **true** | `#  `/`## `/`###` → 4 |
| `otherstone_bricks_wall` | `minecraft:cobblestone_wall` | 1.5 / 1.5 | **true** | `ooo`/`ooo` → 6 |
| `otherrock_*` | the same templates | the same | **true** | the same, `otherrock` |

Upstream (`OccultismBlocks.java:190-246`): the bricks, chiseled and cracked blocks are
`ofFullCopy(OTHERSTONE)` (hardness 1.5, resistance 30, `requiresCorrectToolForDrops`), the slab
`ofFullCopy(OTHERSTONE_BRICKS)`, the stair and wall `of().strength(1.5f).requiresCorrectToolForDrops()`.
All twelve are pickaxe-gated, so a hand break breaks them and rolls nothing — the same fail-capable
pair slice 3 used for `otherstone`, measured again here for all twelve.

Source read: `klikli-dev/occultism` at tag `release/v1.21.1-1.225.0`
(commit `91de0d2a46e05ce315cc5afd292d266d4f46e7ab`, 2026-11-02), files
`registry/OccultismBlocks.java` (the twelve declarations and their properties),
`datagen/recipe/OccultismRecipeProvider.java` (the grid shapes and the chiseled-pair source) and
`datagen/tags/OccultismBlockTagProvider.java` (which tag each block joins). The clone was deleted after
these findings; the jar stays in `../_shared/licence-cache/` as the licence/asset evidence.

The chiseled pair is **two `otherstone_bricks_slab` stacked**, not a bricks 2x2: the jar's own recipe
(`chiseled_otherstone_bricks.json`, pattern `["o","o"]`). That is what makes this tier a chain
(stones → bricks → slab → chiseled) rather than twelve independent grids, and the e2e proves it in one
opened table.

### Upstream defect fixed (the second instance of the 0.2.6 omission)

**`otherrock_bricks_slab` has no loot table in Occultism 1.225.0 either.** The jar carries eleven of
these twelve tables and omits the otherrock brick slab, while `OccultismBlocks.OTHERROCK_BRICKS_SLAB`
is registered `LootTableType.CUSTOM` (so nothing generates a default). `unzip -l … | grep
loot_table/blocks | grep slab` lists `othercobblestone_slab`, `otherstone_slab`, `otherplanks_slab`,
`otherstone_bricks_slab` and `polished_otherstone_slab` — every slab of the family **except** the two
otherrock ones; 0.2.6 shipped `otherrock_slab`, this slice ships its brick sibling.

- **Proved** by the jar listing and by the red run (`otherrock_bricks_slab.json: No such file or
  directory` when the unit test reads it).
- **Fixed** minimally: `ports/occultism/data/occultism/loot_table/blocks/otherrock_bricks_slab.json`,
  the same table as `otherstone_bricks_slab` (with its `type=double` → 2 branch).
- **Proved fixed in game**: the wooden-pickaxe break drops exactly one `occultism:otherrock_bricks_slab`
  (with no table it would drop nothing — the hand/pickaxe pair would both be 0).
- **User-visible**: the slab is obtainable instead of vanishing on break.

Worth an upstream report (the same one-file datagen omission as 0.2.6's); not filed — the campaign
files one only on request.

### Red → green

**Unit red 1 — the dialect** (`sdk-test-red-0.2.7.log`), with the jar's ten grids and eleven tables
carried **byte for byte** first:

```
---- runtime::tests::the_brick_recipes_are_the_jars_shapes_in_the_26_3_dialect stdout ----
thread 'runtime::tests::the_brick_recipes_are_the_jars_shapes_in_the_26_3_dialect' (3762475) panicked at src/runtime.rs:1045:17:
otherstone_bricks carries the 1.21.1 object-form ingredient "item":, which the 26.3 loader drops: {
  "type": "minecraft:crafting_shaped",
…
test result: FAILED. 40 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The other four failures in that run are the files this slice had not written yet: the twelve block
declarations, the smelting pair's dialect, the missing loot table and the tag/lang entries. Captured
rather than staged away — the test set was written first, as 0.2.6 did.)

**Unit red 2 — in the test, not the product** (`sdk-test-red2-0.2.7.log`): after the dialect rewrite
the chiseled assertion counted `"o"` occurrences and its own key is one of them:

```
thread 'runtime::tests::the_brick_recipes_are_the_jars_shapes_in_the_26_3_dialect' (3769361) panicked at src/runtime.rs:1111:13:
assertion `left == right` failed: the chiseled recipe is two stacked slabs: {
  left: 3
 right: 2
test result: FAILED. 44 passed; 1 failed; …
```

Fixed the assertion (pin the `pattern` array itself, not a substring count), no product change.
Green: **45 passed, 0 failed** (`sdk-test.log`).

**In-game red — the family** (`e2e-bricks-red-0.2.6-package.log`, the **0.2.6** export,
`BRICKS_ONLY=1`):

```
== R: the twelve bricks-tier otherstone/otherrock blocks ==
  survival, table at 222,101,38
BRICKS FAIL: the otherstone_bricks grid made None x0, expected occultism:otherstone_bricks x4
FAIL: the bricks check failed (see the BRICKS FAIL line above)
EXIT:1
```

The 0.2.6 package has the ingredients but no brick results, so the grid yields nothing: the exact
behaviour this slice changes. The 0.2.7 package, same leg: `PASS` + `EXIT:0`
(`e2e-bricks-0.2.7.log`).

### In game (phase R, verbatim from `e2e-bricks-0.2.7.log`, abridged)

```
== R: the twelve bricks-tier otherstone/otherrock blocks ==
  survival, table at 222,101,38
  the otherstone_bricks grid: the result slot reads occultism:otherstone_bricks x4
  the otherstone_bricks recipe: every ingredient was consumed ({"occultism:otherstone_bricks": 4})
  … the otherstone_bricks_slab x6, _stairs x4, _wall x6, chiseled_otherstone_bricks x1 …
  … and the same five for otherrock …
  four dirt in the 2x2: the result slot stayed empty
  three otherstone, one cell short of the 2x2: the result slot stayed empty
== the chain in one opened table: 12 otherstone -> bricks -> stairs/slabs -> chiseled ==
  chain step 1.2: the 2x2 of stones: the result slot reads occultism:otherstone_bricks x4
  chain step 1.2: four bricks: taken into slot 1 (inventory {"occultism:otherstone_bricks": 12})
  chain step 1: the twelve stones: every ingredient was consumed ({"occultism:otherstone_bricks": 12})
  chain step 2: six bricks: the result slot reads occultism:otherstone_bricks_stairs x4
  chain step 2: the bricks it used: every ingredient was consumed ({"occultism:otherstone_bricks": 6, "occultism:otherstone_bricks_stairs": 4})
  chain step 3: three bricks: the result slot reads occultism:otherstone_bricks_slab x6
  chain step 3: the bricks it used: every ingredient was consumed ({"occultism:otherstone_bricks": 3, "occultism:otherstone_bricks_slab": 6, "occultism:otherstone_bricks_stairs": 4})
  chain step 4: two brick slabs: the result slot reads occultism:chiseled_otherstone_bricks x1
  chain step 4: the slabs it used: every ingredient was consumed ({"occultism:chiseled_otherstone_bricks": 1, "occultism:otherstone_bricks": 3, "occultism:otherstone_bricks_slab": 4, "occultism:otherstone_bricks_stairs": 4})
== the cracked pair in a real furnace, and the same bricks with no fuel ==
  furnace with coal: [null, null, {"count": 1, "item": "occultism:cracked_otherstone_bricks"}]
  otherstone_bricks + coal -> occultism:cracked_otherstone_bricks (the input is gone)
  furnace with coal: [null, null, {"count": 1, "item": "occultism:cracked_otherrock_bricks"}]
  otherrock_bricks + coal -> occultism:cracked_otherrock_bricks (the input is gone)
  furnace without fuel: [{"count": 1, "item": "occultism:otherstone_bricks"}, null, null]
  no fuel: the output stayed empty and the input stayed put
== the twelve blocks: default state, hand 0, wooden pickaxe 1 ==
  placed occultism:otherstone_bricks at x=214
  occultism:otherstone_bricks by hand: {"block_before": "occultism:otherstone_bricks", "block": "minecraft:air", "drops": {}}
  occultism:otherstone_bricks with a wooden pickaxe: {"…", "drops": {"occultism:otherstone_bricks": 1}}
  … the same pair for all twelve (slab [type=bottom,waterlogged=false], stairs [facing=north,…],
      wall [east=none,…,up=true,…], chiseled and cracked plain) …
  wall pair: occultism:otherstone_bricks_wall[east=low,…] / occultism:otherstone_bricks_wall[…,west=low]
  double brick slab: the pickaxe break dropped 2 (the jar table's double branch)
```

The driver is `tools/bricks_recipes.py`. It opens a **real** table (`ctl interact`), fills the grid one
item per cell, and for the chain keeps the table **open** across the four steps, so the second grid's
bricks are the first grid's output (the inventory line proves it: 12 → 6 → 3). Each furnace leg uses
`ctl container` to load the input and the fuel and reads the block entity's slots back
(`[null, null, {cracked: 1}]`). Differentials: four dirt in the same 2x2 and three stones one cell
short make nothing; a furnace with the bricks but no coal never cooks. `BRICKS_NEGATIVE=1` asserts the
bricks grid makes nothing, so the run must fail (`e2e-negative-bricks-0.2.7.log`:
`BRICKS FAIL: negative: the bricks grid made occultism:otherstone_bricks x4, expected nothing`,
`EXIT:1`).

### Ladder

| step | command | evidence | result |
|---|---|---|---|
| tests (red 1) | `cargo test --offline`, jar bytes carried verbatim | `sdk-test-red-0.2.7.log` | **40 passed, 5 failed** (`the_brick_recipes_are_the_jars_shapes_in_the_26_3_dialect`: `carries the 1.21.1 object-form ingredient "item":`) |
| tests (red 2) | same, after the dialect rewrite | `sdk-test-red2-0.2.7.log` | **44 passed, 1 failed** — the chiseled assertion's own substring count; fixed the assertion |
| tests | `cargo test --offline` | `sdk-test.log` | **45 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `cargo fmt --check` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | `sdk-build-0.2.7.log`, `sdk-build-second-0.2.7.log`, `sdk-reproducible-0.2.7.txt` | **`REPRODUCIBLE`**, 185,574 B, `b746bfac…44ff59` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | `sdk-artifact-0.2.7.txt` | 185,574 B, `b746bfac30f361eaf4693c83cf4aec016702f3f8b3567fbb4480d84cf044ff59`, **341 entries**, no packed `Cargo.lock`, declares **0.2.7**, `src/runtime.rs` inside = outside (`c08c8c8a…`) |
| tree match | a third build from the final tree + `cmp` | `sdk-reproducible-0.2.7.txt` | **`PACKAGE_MATCHES_TREE`** — the delivered file is the tree's build |
| package check | `cargo horizon-mod check occultism.hmod` | `sdk-check.log` (sha printed) | exit 0, `occultism 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` | `sdk-fits.log` (sha printed) | `0a875da  made for`, exit 0 |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` | `sdk-fits-next.log` (sha printed) | **`0b14f8a  fits`**, exit 0 |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | — | **1/1** |
| game build | `launcher-game.sh build occultism` (the stale 0.2.6 tag and export removed first) | `launcher-build-0.2.7.log` | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-6ce2f48c7333`, manifest `version = "0.2.7"` |
| in game (the family, red) | `BRICKS_ONLY=1` against the **0.2.6** export | `e2e-bricks-red-0.2.6-package.log` | **FAIL** + `EXIT:1` (`the otherstone_bricks grid made None x0`) |
| in game (the family) | `BRICKS_ONLY=1` | `e2e-bricks-0.2.7.log` | **PASS** + `EXIT:0` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen) | `e2e-default-0.2.7.log` | **PASS** + `EXIT:0` (A..R) |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | `e2e-control-0.2.7.log` | **FAIL** + `EXIT:1` (`the datura item did not register`) |
| fail-capability (scenario) | `SELFTEST=1` | `e2e-selftest-0.2.7.log` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) |
| fail-capability (this family) | `BRICKS_ONLY=1 BRICKS_NEGATIVE=1` | `e2e-negative-bricks-0.2.7.log` | **FAIL** + `EXIT:1` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | — | **36 logs, 36 ok**, 0 missing-exit, 0 bad-exit |

### Parity coverage (0.2.7)

| family | state |
|---|---|
| `otherstone_bricks`, `otherrock_bricks` + their recipes | **tested** — R (2x2 grid → exactly 4, ingredients consumed; four dirt and three stones one cell short make nothing; default state read back; hand 0 / pickaxe 1) |
| `otherstone_bricks_slab`, `otherrock_bricks_slab` + recipes | **tested** — R (middle row → 6; `type=double` drops 2; `otherrock_bricks_slab`'s table is the port's fix for the jar's omission) |
| `otherstone_bricks_stairs`, `otherrock_bricks_stairs` + recipes | **tested** — R (the stair step → 4; hand 0 / pickaxe 1) |
| `otherstone_bricks_wall`, `otherrock_bricks_wall` + recipes | **tested** — R (two rows → 6; two brick walls connect `west=low`/`east=low`) |
| `chiseled_*` + recipes | **tested** — R (two brick slabs → 1; the chain's last step consumes the slab the chain made) |
| `cracked_*` + their smelting recipes | **tested** — R (a real furnace, 200 ticks: bricks + coal → exactly 1 cracked brick and the input gone; no fuel → nothing, input kept) |
| the chain stones → bricks → stairs → slabs → chiseled | **tested** — R (one opened table; the inventory shows 12 → 6 → 3 bricks and 6 → 4 slabs) |
| the remaining 22 building recipes (polished, cobblestone families, buttons, pressure plates, frames, pedestals, tablets, sacrificial bowls, storage controllers) | **not carried** — their results are still unregistered; the next tiers of the family |
| the 17 impure-chalk craftings | **not carried** — ingredients missing (unchanged) |
| spirit fire, the 26 conversion recipes, the crop, the seed source, the readouts, the spectator guard, the four loot tables, the two 0.2.5 recipes, the six 0.2.6 blocks | **tested** — phases A..Q, unchanged |
| pentacle rituals, spirits/AI, GeckoLib models, Otherworld, guide book, Curios, machine recipes | **not ported** — the gap list above (unchanged) |

### Measured behaviours of this pin (added by this slice)

1. **The vanilla furnace is a block entity this pin ticks.** `ctl container` fills its three slots in
   order (input, fuel, output), and a 200-tick smelting recipe finishes inside `ctl wait 240`. The
   no-fuel leg is the differential: nothing cooks and the input stays.
2. **A shaped recipe's grid cells must come from one stack of at least the pattern's count.** The
   driver places one item per cell by right-clicking, so a second craft of the same item has to merge
   into the same slot (the driver's `empty_or_same`) — two separate four-stacks cannot fill a six-cell
   stair pattern, which is a vanilla rule, not a harness quirk.
3. **A chain needs the table to stay open.** `reset_table()` closes and re-places the table; the chain
   leg deliberately skips it between steps, which is also what a player does.
4. The 0.2.4–0.2.6 findings stand (peaceful in the End, the break reply as the drop assertion, a fresh
   table per grid, `--offscreen` for menu phases, the stale-export trap before a launcher build).

### Deviations (deliberate, and why)

| Area | Upstream | This port |
|---|---|---|
| map colour of the stairs and walls | `Properties.of()` leaves `MapColor.NONE` | the template's stone colour (as 0.2.6's) |
| `otherrock_bricks_slab` table | missing from the jar | shipped (the upstream-defect fix above) |
| the cracked pair's obtainability | smelting only | smelting only — no crafting shortcut is invented |

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

### Next slice

The next tier of the same family, and the cheapest real step: the **polished pair**
(`polished_otherstone`, `polished_otherrock`) with their slab/stair/wall, whose source is the jar's
`smelting/polished_otherstone.json` / `polished_otherrock.json` (a furnace recipe from the raw stone,
which this slice's furnace leg has just proved works) plus the six crafting grids. Gap line: the 0.2.7
coverage row above — *"the remaining 22 building recipes (polished, cobblestone families, buttons,
pressure plates, frames, pedestals, tablets, sacrificial bowls, storage controllers) — not carried"*.
It fits the same `data`-inside-`deep` shape: nine results, one smelting pair, their loot tables,
models, tags and lang, and the same in-game pattern (craft, place, read back, hand 0 / pickaxe 1) with
the furnace leg already scripted. After that the **cobblestone families** (six results) and the
**redstone pair** (buttons, pressure plates) close the "made from the stone" recipes; the frames,
pedestals, tablets and bowls are the first results that are not plain building blocks (they carry
shapes and, for the bowls, a block entity upstream) and belong in a slice of their own.
