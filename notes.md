# Occultism → Horizon

Ported from **Occultism 1.225.0** (`occultism-1.21.1-neoforge-1.225.0.jar`, MIT,
`klikli-dev/occultism` at tag `release/v1.21.1-1.225.0`), Minecraft Java 1.21.1
NeoForge. The upstream source and the jar were read for behaviour and assets
only; the Rust is a clean-room rewrite. Local-only until publishing is approved
(ARCHITECTURE.md §8). Pin **`1d63e0a5`** (`game.rs` `64813cf4…`), mixins
**`5a64af93…`** (schema 6).

Status: **ported-partial** — slice 1 (the spirit-fire conversion loop, with its
book-free data) is implemented and verified in game. The gap list below is the
rest of the mod. **0.2.0 (2026-11-18):** the same slice republished as the flat
SDK `.hmod` (mod API 0.2) after the `.hnmod` retirement; see
[`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18).

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
- **Readouts.** `/occultism info <x> <y> <z>` (block, colour, conversions of
  that fire, last conversion, rules loaded) and `/occultism recipes` (rules,
  files, distinct inputs, skipped) — the e2e asserts the numbers.
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
