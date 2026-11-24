# Validation — Occultism slice 1 (spirit fire)

> **Historical (0.1.0, the retired `.hnmod`).** Sections 1–3 below are the record of the
> 2026-11-10/11 `.hnmod` port, kept as evidence. Their paths (`mixins/occultism/runtime.rs`,
> `tools/carry_content.py`, `tools/horizon.py`, `content/…`) and their in-game runs describe the
> retired container; the 0.2.0 SDK package is validated in §4. The in-game harness itself is
> unchanged in behaviour and is now launched by the launcher (`tools/e2e.sh`); see the gap in
> `notes.md`.

Cheapest first. Every command ran through a leased slot with
`CARGO_INCREMENTAL=0`; no bare cargo, no `--target-dir`. Logs are the evidence:
`ports/_shared/evidence/occultism/`.

Where each step ran (the campaign's two machines, `AGENTS.md` "build slots" and
`_shared/remote-build.md`): the standalone tests and `check`/`test` compiles on
the **far box** (`build-slot.sh run`, forwarded, artefacts and evidence pulled
back); the three game legs (`e2e-devmod`, `e2e-embedded`, `e2e-selftest`) through
`build-slot.sh e2e`, forwarded to the far box, which has the prepared 26.3
resources and the published `horizon-ctl`; the content-package-only leg
(`hmod-load`) **local**, because the published shared game needs GLIBC 2.44 and
the far box has 2.43 — the leg skips itself there with `SKIP:` and exit 77
instead of failing.

## 1. The pure runtime, standalone

| Step | Command | Result |
|---|---|---|
| RED | `cargo test --locked --offline` | **FAILED. 6 passed; 10 failed** — the 10 failed on their assertions (`left: None`, `left: [10, 64, -3]`), not on a compile error (the output is quoted below; the log file of that run was not kept) |
| GREEN | `cargo fmt --check` + `cargo clippy --all-targets --locked --offline -- -D warnings` + `cargo test --locked --offline` | **fmt clean, 0 clippy warnings, 16 tests passed** (`evidence/occultism/runtime-green.log`) |
| red→green | 16 tests (see `mixins/occultism/runtime.rs`) | one test's expected box was mis-typed on the first green run (`[11.5, 65.5, -2.5]` for upstream's `[11.5, 65.5, -1.5]`); the implementation was right, the expectation was fixed |

The RED run, verbatim:

```
---- tests::first_rule_wins stdout ----
thread 'tests::first_rule_wins' panicked at mixins/occultism/runtime.rs:166:9:
assertion `left == right` failed
  left: None
 right: Some("occultism:taboo_book")

---- tests::fire_target_moves_onto_the_clicked_face stdout ----
thread 'tests::fire_target_moves_onto_the_clicked_face' panicked at mixins/occultism/runtime.rs:274:9:
assertion `left == right` failed
  left: [10, 64, -3]
 right: [10, 65, -3]

failures:
    tests::a_block_result_is_an_item_id
    tests::accepts_every_tag_member
    tests::chalk_gold_is_yellow
    tests::chalk_items_name_their_colour
    tests::converts_whole_stacks
    tests::datura_box_matches_upstream
    tests::fire_target_moves_onto_the_clicked_face
    tests::first_rule_wins
    tests::lighters_are_flint_and_steel_and_fire_charge
    tests::matches_by_exact_id

test result: FAILED. 6 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


## 2. Inside the game crate (staged at the pin)

| Step | Command | Result |
|---|---|---|
| content carry | `python3 tools/carry_content.py` | 5 blocks, 41 items, 27 recipes, 171 files; `--check` green |
| package | `tools/package.sh` | `.hmod` 86,714 B sha256 `2e2b74de…95a8`, 171 entries, `no code: content only`; `.hnmod` 68,204 B sha256 `dce9da26…2489`, 11 entries (`evidence/occultism/ladder-remote.log`) |
| stage | `python3 tools/horizon.py stage` | ok, `stage_changed: true`, 302 s (the pin's `git archive` + the game.rs hash) |
| check | `python3 tools/horizon.py check` | **`cargo_exit: 0`** — the injected modules compile inside the real `horizon-client` (557 s) |
| test | `python3 tools/horizon.py test` | first run **filtered the copied `game::charginggadgets::tests` and ran 0 tests** (exit 0, vacuous); the filter is now `game::occultism::tests` and the leg is part of `tools/e2e-all.sh`, which fails if 0 tests match |
| build | `python3 tools/horizon.py build` | **`cargo_exit: 0`**, binary `slot-5/target/fast/horizon-client` |

`check` also showed 3 `private_interfaces` warnings (a `pub fn` taking the
private `Game`); each such function now carries the campaign's
`#[allow(private_interfaces, reason = "the injected bundle is this crate's own module")]`,
so the re-run is warning-free.

## 2b. Reproducibility

| Artefact | Rebuild | Result |
|---|---|---|
| `content/occultism.hmod` | `cargo horizon-mod build` again on the far box | **byte-identical**: 86,714 B, sha256 `2e2b74de…95a8`, 171 entries |
| `occultism.hnmod` | `tools/package.py` twice on this box | **byte-identical**: 69,082 B, sha256 `777bba62…b43f`, 11 entries |
| `occultism.hnmod` | the same sources packaged on the far box | **different bytes, identical contents**: 69,053 B, every entry's CRC and size the same, only the deflate streams differ (the two boxes' zlib builds). The registry hash must be the releasing box's file, which is this one. |

## 3. In game

| Step | Command | Result |
|---|---|---|
| content package alone | `tools/hmod-load.sh` | _filled by the run_ |
| e2e (dev-mod content) | `build-slot.sh e2e -- tools/e2e.sh` | _filled by the run_ |
| e2e (embedded, control) | `EMBEDDED=1 …` | _filled by the run_ |
| selftest | `SELFTEST=1 …` | _filled by the run_ (must exit ≠ 0) |
| log checker | `python3 ports/tools/check-e2e-logs.py occultism --strict` | _filled by the run_ |

## 4. SDK 0.2.0 (the flat `.hmod`, pins `0b14f8a` / `e59da90`)

Every step ran in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` (the remote box's
sync has no `ports/occultism` path for the brief's `cd`), `CARGO_INCREMENTAL=0`, no
`--target-dir`, and the evidence is `ports/_shared/evidence/occultism/sdk-*.log`.

| Step | Command | Result |
|---|---|---|
| runtime first | `cargo test --offline` on the new crate before any glue | **19 runtime tests** (the byte-identical file) |
| runtime alone | `rustc --edition 2024 --test src/runtime.rs` (no `mod_api` at all) | **19 passed**, 0 failed — `sdk-runtime-standalone.log` |
| glue red | `cargo test --offline` with the JSON-reference helper on `todo!()` | **20 passed, 3 failed** on the `todo!()` (a real red, not a compile error) — `sdk-test-red.log` |
| tests | `cargo test --offline` | **23 passed, 0 failed** (19 `runtime::`, 4 `support::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** — `sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`) | exit 0, empty diff — `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk` / `Built ./target/horizon-mod/occultism.hmod` — `sdk-build.log` |
| registry check | `cargo horizon-mod check occultism.hmod` | `occultism 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)` / **109,133 B**, sha256 `044b8360…4dc8`, **exit 0** — `sdk-check.log` |
| fits the pinned build | `cargo horizon-mod fits occultism.hmod --build nightly` | `Occultism:` / `0b14f8a  made for`, exit 0 — `sdk-fits.log` |
| twice | `rm -f Cargo.lock && cargo horizon-mod build` again | **byte-identical**: 109,133 B, sha256 `044b8360…4dc8` both times — `sdk-reproducible.txt` |
| runtime untouched | `unzip -p occultism.hmod src/runtime.rs \| sha256sum` | `108987a614c73f1e2ad4d76c21ac7d65774333d426e69380d64df83832330f9d` = the retired file and `src/runtime.rs`, byte for byte — `sdk-runtime-sha256.txt` |
| content carry | `diff -r` against the retired `content/occultism.hmod` | 87 `assets/` + 85 `data/` files, **unchanged** — `sdk-content-diff.log` |
| archive | `unzip -l occultism.hmod` | **181 entries**: 87 `assets/`, 85 `data/`, 3 `src/`, 6 root (`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `uses.json`); **no `Cargo.lock`** — `sdk-entries.txt` |
| surface | `unzip -p occultism.hmod uses.json` | 251 entries: 8 hook ids, 24 methods, 55 handles, 17 traits, 127 types, 1 macro, 1 capability, 3 parts — `sdk-uses-summary.txt`, `sdk-uses-from-package.json` |
| in game | `tools/e2e.sh` | **not run**: no launcher (`horizon-launcher`) at this pin, so no launcher-built game exists; recorded as a gap |

## 5. SDK 0.2.2 (review repair: operator-only readout, spectator guard)

Every step in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`;
no `--target-dir`, no private slots. Evidence in `../_shared/evidence/occultism/` and
`../_shared/evidence/occultism-e2e/`.

| Step | Command | Result |
|---|---|---|
| red (unit) | `cargo test --offline` with the new tests | **4 failed** on assertions (`the_info_readout_needs_an_operator`, `a_non_fire_readout_names_no_block`, `a_spectator_click_changes_nothing`, `the_info_command_is_declared_for_operators`) — `sdk-test-red2-0.2.2.log` |
| green (unit) | `cargo test --offline` | **29 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy-0.2.2.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt-0.2.2.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice | **byte-identical**: 113,674 B, sha256 `35714e6f…` — `sdk-reproducible-0.2.2.txt` |
| package | `sha256sum`, `unzip -l` | 113,674 B, **181 entries**, no packed `Cargo.lock`, declares **0.2.2** — `sdk-artifact-0.2.2.txt` |
| package check | `cargo horizon-mod check occultism.hmod` | exit 0 — `sdk-check-0.2.2.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` | `0b14f8a  made for`, exit 0 — `sdk-fits-0.2.2.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 6d765e0` | `6d765e0  fits`, exit 0 — `sdk-fits-next-0.2.2.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** |
| in game (reviewed 0.2.1) | `bash tools/e2e.sh` against the 0.2.1 export | **FAIL** on both findings: `a spectator recoloured the fire to 'red'` (`e2e-red-0.2.1.log`), `a non-fire position answered 'Spirit Fire at … block=minecraft:air …'` (`e2e-red2-0.2.1.log`) |
| in game (0.2.2) | `bash tools/e2e.sh` | **PASS** + `EXIT:0` — `e2e-default-0.2.2.log` |
| in game (+ control leg) | `CONTROL_GAME=…` | **PASS** + `EXIT:0`; the unmodded Nightly has no datura item and no block — `e2e-default-with-control-leg-0.2.2.log` |
| in game (control) | `GAME=$CONTROL_GAME` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.2.log` |
| fail-capability | `SELFTEST=1` | **FAIL** + `EXIT:1` — `e2e-selftest-0.2.2.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | 8 logs, 8 ok, 0 missing-exit, 0 bad-exit |

## 6. SDK 0.2.3 (the datura crop and its seed source), pin `0a875da`

Repinned `0b14f8a` → `0a875da` (`[native] horizon`, `.horizon/sdk` deleted so the SDK fetched
again). Every step in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; no `--target-dir`, no private slots. Evidence in
`../_shared/evidence/occultism/` and `../_shared/evidence/occultism-e2e/`.

| Step | Command | Result |
|---|---|---|
| SDK fetch | `cargo horizon-mod build` after deleting `.horizon/sdk` | `SDK 0a875da … in ./.horizon/sdk` — `sdk-build-fetch-0.2.3.log` |
| red (unit) | `cargo test --offline` with the new tests | **29 passed; 4 failed** on assertions (`a_ripe_datura_is_harvested_and_a_young_one_is_not`, `the_harvest_replants_the_crop_at_age_zero`, `a_hand_break_of_grass_is_a_seed_source`, `the_seed_roll_is_upstreams_two_percent`) — `sdk-test-red-0.2.3.log` |
| green (unit) | `cargo test --offline` | **33 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **byte-identical**: 124,954 B, sha256 `e81ce017…` — `sdk-build-0.2.3.log`, `sdk-build-second-0.2.3.log`, `sdk-reproducible-0.2.3.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 124,954 B, `e81ce0172de24411deefb93a3479b1c043e1e57c93c04924cb726b1ff93e5949`, **202 entries**, no packed `Cargo.lock`, declares **0.2.3** — `sdk-artifact-0.2.3.txt` |
| tree match | rebuild + `cmp occultism.hmod` | `PACKAGE_MATCHES_TREE` (the delivered file is the tree) |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed in the log) | exit 0, `occultism 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` (sha printed) | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** |
| in game (modded) | `launcher-game.sh build occultism` (`Compiling occultism (1 of 1)`) then `bash tools/e2e.sh` | **PASS** + `EXIT:0` — `e2e-default-0.2.3.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.3.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` — `e2e-selftest-0.2.3.log` |
| fail-capability (the crop family) | `NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the ripe crop dropped 1 fruit`) — `e2e-negative-0.2.3.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | 12 logs, 12 ok, 0 missing-exit, 0 bad-exit |

The in-game phases K..N and the measured pin behaviours (an explosion's loot has no block context,
`ctl break` mines the crosshair, the control-leg/HORIZON_HOME interaction, random ticks during the age
read-back) are in `notes.md` under "Slice 2 (0.2.3, 2026-11-21)".

## 7. SDK 0.2.4 (the four carried block loot tables proven in game), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`;
no `--target-dir`, no private slots. Evidence in `../_shared/evidence/occultism/` (ladder) and
`../_shared/evidence/occultism-e2e/` (game legs). The four tables, their block files and the pickaxe
tag are **unchanged** from 0.2.3: the package's only byte change is the version. There is no product
red; the red was in the new leg and in two 0.2.3 harness phases (both fixed, quoted in `notes.md`).

| Step | Command | Result |
|---|---|---|
| the leg's first honest run (0.2.3 bytes) | `LOOT_ONLY=1 bash tools/e2e.sh` | **PASS** + `EXIT:0` — the shipped data already behaved as specified — `e2e-loot-only-0.2.3-package.log` |
| tests | `cargo test --offline` | **33 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **byte-identical**: 124,955 B, sha256 `f49ee35df9fc22d00217e611440573c6c771151b8cda963d252f46e7761ca3cd` — `sdk-build-0.2.4.log`, `sdk-build-second-0.2.4.log`, `sdk-reproducible-0.2.4.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 124,955 B, `f49ee35d…ca3cd`, **202 entries**, no packed `Cargo.lock`, declares **0.2.4** — `sdk-artifact-0.2.4.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` (sha printed) | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** |
| game build | `launcher-game.sh build occultism` | `Compiling occultism (1 of 1)`, tag `native-0a875da-3f170bd8dd15`, exported manifest `version = "0.2.4"` — `launcher-build-0.2.4.log` |
| in game (modded) | `launcher-game.sh run occultism` | **PASS** + `EXIT:0` — `e2e-default-0.2.4.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.4.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.4.log` |
| fail-capability (the loot family) | `LOOT_ONLY=1 LOOT_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: occultism:otherstone dropped 1 with a wooden pickaxe`) — `e2e-negative-loot-0.2.4.log` |
| harness diagnostics | `probe-l.sh`, `probe-n.sh` (temporary, deleted after the runs) | `probe-l-0.2.4.log` (a harvest's seeds picked up at 2 blocks), `probe-n-0.2.4.log` (25/25 hand, 25/25 shears after `mine_patch`) |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | 17 logs, 17 ok, 0 missing-exit, 0 bad-exit |

## 8. SDK 0.2.5 (the two carried crafting recipes, and the one-harvest fix), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`;
no `--target-dir`, no private slots. Evidence in `../_shared/evidence/occultism/` (ladder) and
`../_shared/evidence/occultism-e2e/` (game legs). The slice carries two recipe JSONs in the 26.3
dialect and fixes one port defect (the harvest ran once per hand); the enumeration of what was
carried and what was not is in `notes.md`, §"Slice 4".

| Step | Command | Result |
|---|---|---|
| red (unit, the dialect) | `cargo test --offline` with the jar's bytes carried verbatim | **33 passed; 2 failed** — `the_carried_crafting_recipes_are_in_the_26_3_dialect` ("… object-form ingredient \"tag\": …") — `sdk-test-red-0.2.5.log` |
| red (unit, the harvest hand) | `cargo test --offline` with `may_harvest` permissive | **35 passed; 1 failed** — `only_the_main_hand_harvests_a_crop` — `sdk-test-red-hand-0.2.5.log` |
| green (unit) | `cargo test --offline` | **36 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 127,502 B — `sdk-build-0.2.5.log`, `sdk-build-second-0.2.5.log`, `sdk-reproducible-0.2.5.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 127,502 B, `658ed0f7a8ae06c06ae6b786930ac3d4124027631b2935791e3506cdf315e4e4`, **204 entries**, no packed `Cargo.lock`, declares **0.2.5** — `sdk-artifact-0.2.5.txt` |
| package check | `cargo horizon-mod check occultism.hmod` | exit 0, `occultism 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da` | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** |
| game build | `launcher-game.sh build occultism` (stale export + version folder removed) | `Compiling occultism (1 of 1)`, tag `native-0a875da-81d30b2bded4` — `launcher-build-0.2.5.log` |
| red (in game, the family) | `CRAFT_ONLY=1 bash tools/e2e.sh` against the **0.2.4** export | **FAIL** + `EXIT:1` (`the 3x3 fruit grid made None x0`) — `e2e-craft-red-0.2.4-package.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen) | **PASS** + `EXIT:0` — `e2e-default-0.2.5.log` |
| in game (crafting leg) | `CRAFT_ONLY=1` | **PASS** + `EXIT:0` — `e2e-craft-0.2.5.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` | **FAIL** + `EXIT:1` — `e2e-control-0.2.5.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` — `e2e-selftest-0.2.5.log` |
| fail-capability (this family) | `CRAFT_ONLY=1 CRAFT_NEGATIVE=1` | **FAIL** + `EXIT:1` — `e2e-negative-craft-0.2.5.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | 23 logs, 23 ok, 0 missing-exit, 0 bad-exit |

The phase-P transcript, the enumeration of the 166 jar crafting recipes (2 carried, 17 impure
chalks and the 38-recipe otherstone/otherrock family excluded), and the harness behaviours this
slice measured (offscreen for menus, the death screen during phase N, the swallowed break
diagnostics, the planting-click retry) are in `notes.md` under "Slice 4 (0.2.5, 2026-11-21)".

## 9. SDK 0.2.6 (the six `otherstone`/`otherrock` building blocks), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0`; no `--target-dir`, no
private slots. Evidence in `../_shared/evidence/occultism/` (ladder) and
`../_shared/evidence/occultism-e2e/` (game legs). The slice adds six block declarations, six 26.3
recipes, six loot tables (one of which the jar omits), sixteen block models, six blockstates, six
item models, six lang keys, four block tags and sixteen creative-tab entries; the enumeration and
the upstream defect are in `notes.md`, §"Slice 5".

| Step | Command | Result |
|---|---|---|
| red 1 (unit, the dialect) | `cargo test --offline` with the jar's six recipes and five loot tables carried verbatim | **38 passed; 2 failed** — `the_building_recipes_are_the_jars_shapes_in_the_26_3_dialect` ("carries the 1.21.1 object-form ingredient \"item\":") and the loot table's `"function":` shape — `sdk-test-red-0.2.6.log` |
| red 2 (unit, a test assertion) | same, after the dialect rewrite | **39 passed; 1 failed** — my first loot test demanded a `modifier` list in every table; 26.3 does not (fixed the assertion, no product change) — `sdk-test-red2-0.2.6.log` |
| red 3 (unit, the upstream defect) | same, after the recipes | **39 passed; 1 failed** — `otherrock_slab.json: No such file or directory`: the table the jar does not ship — `sdk-test-red3-0.2.6.log` |
| green (unit) | `cargo test --offline` | **40 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 145,445 B, `7b382835…0992c` — `sdk-build-0.2.6.log`, `sdk-build-second-0.2.6.log` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 145,445 B, `7b382835c0b9b7325809c4b542d803c47c0f617865b5d51168aefd828120992c`, **253 entries**, no packed `Cargo.lock`, declares **0.2.6**, `src/runtime.rs` inside = outside (`fea1d79d…`) — `sdk-artifact-0.2.6.txt` |
| tree match | a third build from the final tree + `cmp occultism.hmod` | `PACKAGE_MATCHES_TREE` (the delivered file is the tree's build) — `sdk-artifact-0.2.6.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` (the short names needed GitHub's releases list, HTTP 403 on 2026-11-22; a full SHA resolves from `.horizon/cache/`) |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** |
| game build | `launcher-game.sh build occultism` (stale export removed) | `Compiling occultism (1 of 1)`, tag `native-0a875da-726d4571fa66`, manifest `version = "0.2.6"` — `launcher-build-0.2.6.log` |
| red (in game, the family) | `BUILDING_ONLY=1 bash tools/e2e.sh` against the **0.2.5** export | **FAIL** + `EXIT:1` (`the otherstone slab grid made None x0`) — `e2e-building-red-0.2.5-package.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen) | **PASS** + `EXIT:0` — `e2e-default-0.2.6.log` (A..Q) |
| in game (building leg) | `BUILDING_ONLY=1` | **PASS** + `EXIT:0` — `e2e-building-0.2.6.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.6.log` |
| in game (control ids) | `CONTROL_LEG=1 GAME=$CONTROL_GAME CONTROL_GAME=$CONTROL_GAME bash tools/e2e.sh` | **FAIL** + `EXIT:1`, after `none of the six building blocks either` — `e2e-control-ids-0.2.6.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.6.log` |
| fail-capability (this family) | `BUILDING_ONLY=1 BUILDING_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the slab grid made occultism:otherstone_slab x6, expected nothing`) — `e2e-negative-building-0.2.6.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | 30 logs, 30 ok, 0 missing-exit, 0 bad-exit |

The phase-Q transcript (the six grids, each block's default state, the hand/pickaxe drop pair, the
wall connection, the double slab dropping two), the brief-vs-artefact conflict on `requires_tool`,
and the two loot engines in the tree are in `notes.md` under "Slice 5 (0.2.6, 2026-11-22)".

## 10. SDK 0.2.7 (the bricks tier of the `otherstone`/`otherrock` family), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease (`BUILD_LOCAL=1` for the SDK ladder, as trap 10 of
0.2.0 records), `CARGO_INCREMENTAL=0`, no `--target-dir`, no private slots. Evidence in
`../_shared/evidence/occultism/` (ladder) and `../_shared/evidence/occultism-e2e/` (game legs). The
slice adds twelve block declarations, ten 26.3 crafting recipes, two smelting recipes, twelve loot
tables (one of which the jar omits), twelve blockstates, twenty-two block models, twelve item models,
six textures, twelve lang keys, four block tags and twelve creative-tab entries; the enumeration, the
chain and the upstream defect are in `notes.md`, §"Slice 6".

| Step | Command | Result |
|---|---|---|
| red 1 (unit, the dialect) | `cargo test --offline` with the jar's ten grids and eleven tables carried verbatim | **40 passed; 5 failed** — `the_brick_recipes_are_the_jars_shapes_in_the_26_3_dialect` ("carries the 1.21.1 object-form ingredient \"item\":") plus the four families not yet written — `sdk-test-red-0.2.7.log` |
| red 2 (unit, a test assertion) | same, after the dialect rewrite | **44 passed; 1 failed** — the chiseled assertion counted `"o"` and its own key is one; fixed the assertion, no product change — `sdk-test-red2-0.2.7.log` |
| green (unit) | `cargo test --offline` | **45 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 185,574 B, `b746bfac…44ff59` — `sdk-build-0.2.7.log`, `sdk-build-second-0.2.7.log`, `sdk-reproducible-0.2.7.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 185,574 B, `b746bfac30f361eaf4693c83cf4aec016702f3f8b3567fbb4480d84cf044ff59`, **341 entries**, no packed `Cargo.lock`, declares **0.2.7**, `src/runtime.rs` inside = outside (`c08c8c8a…`) — `sdk-artifact-0.2.7.txt` |
| tree match | a third build from the final tree + `cmp occultism.hmod` | `PACKAGE_MATCHES_TREE` (the delivered file is the tree's build) — `sdk-reproducible-0.2.7.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` (sha printed) | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** |
| game build | `launcher-game.sh build occultism` (the stale 0.2.6 tag + export removed first) | `Compiling occultism (1 of 1)`, tag `native-0a875da-6ce2f48c7333`, manifest `version = "0.2.7"` — `launcher-build-0.2.7.log` |
| red (in game, the family) | `BRICKS_ONLY=1 bash tools/e2e.sh` against the **0.2.6** export | **FAIL** + `EXIT:1` (`the otherstone_bricks grid made None x0`) — `e2e-bricks-red-0.2.6-package.log` |
| in game (the family) | `BRICKS_ONLY=1` | **PASS** + `EXIT:0` — `e2e-bricks-0.2.7.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen) | **PASS** + `EXIT:0` — `e2e-default-0.2.7.log` (A..R) |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.7.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.7.log` |
| fail-capability (this family) | `BRICKS_ONLY=1 BRICKS_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the bricks grid made occultism:otherstone_bricks x4, expected nothing`) — `e2e-negative-bricks-0.2.7.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | 36 logs, 36 ok, 0 missing-exit, 0 bad-exit |

The phase-R transcript (the ten grids, the two differentials, the four-step chain in one opened table,
the furnace pair with its no-fuel differential, the twelve default states with hand 0 / pickaxe 1, the
brick wall connection and the double brick slab dropping two), the jar's second missing slab table and
the two measured behaviours of the furnace/menu harness are in `notes.md` under "Slice 6".

## 11. SDK 0.2.8 (the polished tier of the `otherstone`/`otherrock` family), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease (`BUILD_LOCAL=1` for the SDK ladder, as trap 10 of
0.2.0 records), `CARGO_INCREMENTAL=0`, no `--target-dir`, no private slots. Evidence in
`../_shared/evidence/occultism/` (ladder) and `../_shared/evidence/occultism-e2e/` (game legs). The
slice adds eight block declarations, six 26.3 crafting recipes, two smelting recipes, eight loot
tables (one of which the jar omits), eight blockstates, fourteen block models, eight item models, two
textures, eight lang keys, four block tags and eight creative-tab entries; the enumeration and the
upstream defect are in `notes.md`, §"Slice 7". The family is **eight** results, not the brief's nine
(the jar has 8 blockstates and 8 lang keys; the artefact wins).

| Step | Command | Result |
|---|---|---|
| red 1 (unit, the dialect) | `cargo test --offline` with the jar's 6 grids, 7 tables and 2 smelting files carried verbatim | **47 passed; 3 failed** — `the_polished_recipes_are_the_jars_shapes_in_the_26_3_dialect` ("carries the 1.21.1 object-form ingredient"), `the_polished_pair_is_smelted_from_the_raw_stones` and the loot table's `"function":` shape — `sdk-test-red-0.2.8.log` |
| red 2 (unit, the upstream defect) | same, after the dialect rewrite | **49 passed; 1 failed** — `polished_otherrock_slab.json: No such file or directory` — `sdk-test-red2-0.2.8.log` |
| green (unit) | `cargo test --offline` | **50 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 207,277 B, `c1faa25a…1d0e2` — `sdk-build-0.2.8.log`, `sdk-build-second-0.2.8.log`, `sdk-reproducible-0.2.8.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 207,277 B, `c1faa25ad14a03c78cdea9183c48b38985997518e033f1e0d1a089f7f1e1d0e2`, **401 entries**, no packed `Cargo.lock`, declares **0.2.8**, `src/runtime.rs` inside = outside (`798ab04a…`) — `sdk-artifact-0.2.8.txt` |
| tree match | a build from the final tree + `cmp occultism.hmod` | **`PACKAGE_MATCHES_TREE`** — the delivered file is the tree's build — `sdk-reproducible-0.2.8.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.8 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` (sha printed) | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** — `sdk-gate-0.2.8.txt` |
| game build | `launcher-game.sh build occultism` (the stale 0.2.7 export removed first) | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-9231e7cc9d32`, manifest `version = "0.2.8"` — `launcher-build-0.2.8.log` |
| red (in game, the family) | `POLISHED_ONLY=1` against the **0.2.7** export | **FAIL** + `EXIT:1` (`the furnace made None from otherstone, expected one occultism:polished_otherstone`) — `e2e-polished-red-0.2.7-package.log` |
| in game (the family) | `POLISHED_ONLY=1` | **PASS** + `EXIT:0` — `e2e-polished-0.2.8.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen) | **PASS** + `EXIT:0` (A..S) — `e2e-default-0.2.8.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | **FAIL** + `EXIT:1` (`Unknown item 'occultism:datura'`) — `e2e-control-0.2.8.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.8.log` |
| fail-capability (this family) | `POLISHED_ONLY=1 POLISHED_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the polished slab grid made occultism:polished_otherstone_slab x6, expected nothing`) — `e2e-negative-polished-0.2.8.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **42 logs, 42 ok**, 0 missing-exit, 0 bad-exit |

The phase-S transcript (the six grids with consumption, the two furnace legs at 230 101 42 with a
240-tick wait and a no-fuel differential, the eight default states with hand 0 / pickaxe 1, the
polished wall connection, the double slab dropping two and the missing-table slab dropping one) and
the family's one unported recipe half (the six stonecutting shortcuts) are in `notes.md` under
"Slice 7".

## 12. SDK 0.2.9 (the cobblestone tier of the `otherstone`/`otherrock` family), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease (`BUILD_LOCAL=1` for the SDK ladder, as trap 10 of
0.2.0 records), `CARGO_INCREMENTAL=0`, no `--target-dir`, no private slots. Evidence in
`../_shared/evidence/occultism/` (ladder) and `../_shared/evidence/occultism-e2e/` (game legs). The
slice adds eight block declarations, six 26.3 crafting recipes, eight loot tables (one of which the
jar omits), eighteen block models, eight blockstates, eight item models, two textures, eight lang
keys, four block tags and eight creative-tab entries; it also carries the jar's silk-touch
alternatives table for `otherstone`/`otherrock`, closing the 0.2.4 deviation and making the pair
obtainable. The enumeration, the jar's loot-table inventory (seven of eight), the stonecutting note
and the upstream defect are in `notes.md`, §"Slice 8".

| Step | Command | Result |
|---|---|---|
| red (unit) | `cargo test --offline` with the jar's six grids, seven tables, assets and lang carried verbatim | **50 passed; 5 failed** on assertions — the dialect (`othercobblestone_slab carries the 1.21.1 object-form ingredient "item":`), the loot `"function":` shape, the missing block declarations, the tags/lang, and `otherstone has no silk-touch alternative` — `sdk-test-red-0.2.9.log` |
| green (unit) | `cargo test --offline` | **55 passed, 0 failed** — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 228,663 B, `8c31154f…75e8` — `sdk-build-0.2.9.log`, `sdk-build-second-0.2.9.log`, `sdk-reproducible-0.2.9.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 228,663 B, `8c31154f9a07a6a6d97c62a0d919127ad0a41707341d5ff71be5591d002375e8`, **459 entries**, no packed `Cargo.lock`, declares **0.2.9**, `src/runtime.rs` inside = outside (`79f10b12…`) — `sdk-artifact-0.2.9.txt` |
| tree match | a second build from the final tree + `cmp` | **`PACKAGE_MATCHES_TREE`** — `sdk-artifact-0.2.9.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.9 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` (sha printed) | `0b14f8a  fits`, exit 0 — `sdk-fits-next.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** — `sdk-gate-0.2.9.txt` |
| game build | `launcher-game.sh build occultism` (the stale 0.2.8 export removed first) | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-87d09c7f95d3`, exported `native-mods.json` version `0.2.9` — `launcher-build-0.2.9.log` |
| red (in game, the family) | `COBBLE_ONLY=1` against the **0.2.8** export | **FAIL** + `EXIT:1` (`missing ingredient occultism:othercobblestone`) — `e2e-cobble-red-0.2.8-package.log` |
| in game (the family) | `COBBLE_ONLY=1` | **PASS** + `EXIT:0` — `e2e-cobble-0.2.9.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen) | **PASS** + `EXIT:0` (A..T) — `e2e-default-0.2.9.log` |
| in game (phase O) | `LOOT_ONLY=1` | **PASS** + `EXIT:0` — `e2e-loot-0.2.9.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.9.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.9.log` |
| fail-capability (this family) | `COBBLE_ONLY=1 COBBLE_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the cobble slab grid made occultism:othercobblestone_slab x6, expected nothing`) — `e2e-negative-cobble-0.2.9.log` |
| fail-capability (phase O) | `LOOT_ONLY=1 LOOT_NEGATIVE=1` | **FAIL** + `EXIT:1` (`otherstone dropped 1 othercobblestone with a wooden pickaxe`) — `e2e-negative-loot-0.2.9.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **50 logs, 50 ok**, 0 missing-exit, 0 bad-exit |

The phase-T transcript (the six grids with consumption, the two differentials, the eight default
states with hand 0 / pickaxe 1, the cobble wall connection, the double slab dropping two, the
missing-table slab dropping one, and the raw stones' wooden-pickaxe/silk-touch pair) and the
family's one unported recipe half (the six stonecutting shortcuts, with 0.2.8's reason corrected) are
in `notes.md` under "Slice 8".

## 13. SDK 0.2.10 (the redstone pair of the `otherstone`/`otherrock` family, and the reverse smelt), pin `0a875da`

Every step in a `ports/tools/build-slot.sh` lease (`BUILD_LOCAL=1` for the SDK ladder), `CARGO_INCREMENTAL=0`,
no `--target-dir`, no private slots. Evidence in `../_shared/evidence/occultism/` (ladder) and
`../_shared/evidence/occultism-e2e/` (game legs). The slice adds four block declarations (the two
buttons on `minecraft:stone_button`, the two plates on `minecraft:stone_pressure_plate`), four 26.3
crafting recipes, four loot tables, two reverse-smelting recipes, four blockstates, eight block
models, four item models, four lang keys, three tag files (plus two ids in `mineable/pickaxe`) and
four creative-tab entries; the templates bring the engine's `ButtonBlock`/`PressurePlateBlock`
behaviour, so no runtime rule and no glue were added. All four of the family's loot tables exist in
the jar (the first family here with no omission), the brief's `requires_tool` claim is corrected
against the artefact (false for all four), and the pin cannot express upstream's 30-tick stone button
(it ships the template's 20). The enumeration, the deviation and the upstream item-tag defect are in
`notes.md`, §"Slice 9".

| Step | Command | Result |
|---|---|---|
| red 1 (unit) | `cargo test --offline`, the tests written before any file | **55 passed; 5 failed** — every family file absent (`...: No such file or directory`) — `sdk-test-red-0.2.10.log` |
| red 2 (unit) | the jar's four recipes, four tables, two smelting recipes, assets and lang carried verbatim | **55 passed; 5 failed** — `otherstone_button carries the 1.21.1 string-form condition kind`, `otherstone's smelting recipe carries the 1.21.1 object-form ingredient "item":`, the missing declarations/tags — `sdk-test-red2-0.2.10.log` |
| green (unit) | `cargo test --offline` | **60 passed, 0 failed** — `sdk-test-0.2.10.log` (copied to `sdk-test.log`, which the gate reads) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy-0.2.10.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt-0.2.10.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 241,148 B, `b4807a00…d978` — `sdk-build-0.2.10.log`, `sdk-build-second-0.2.10.log`, `sdk-reproducible-0.2.10.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 241,148 B, `b4807a007516b46c6a5b8684a2bd84f0d65a7e14d65391fd3a62d8d3deb4d978`, **492 entries**, no packed `Cargo.lock`, declares **0.2.10**, `src/runtime.rs` inside = outside (`8ee7d89a…`) — `sdk-artifact-0.2.10.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.10 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check-0.2.10.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits-0.2.10.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` (sha printed) | **`0b14f8a  fits`**, exit 0 — `sdk-fits-next-0.2.10.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** (60 tests) — `sdk-gate-0.2.10.txt` |
| game build | `launcher-game.sh build occultism` (the stale 0.2.9 export renamed first) | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-a79120a2bd80`, exported manifest `version = "0.2.10"` — `launcher-build-0.2.10.log` |
| red (in game, the family) | `REDSTONE_ONLY=1` against the **0.2.9** export | **FAIL** + `EXIT:1` (`the furnace made None from othercobblestone`) — `e2e-redstone-red-0.2.9-package.log` |
| in game (the family) | `REDSTONE_ONLY=1` | **PASS** + `EXIT:0` — `e2e-redstone-0.2.10.log` |
| in game (all phases) | `launcher-game.sh run occultism` (offscreen) | **PASS** + `EXIT:0` (A..U) — `e2e-default-0.2.10.log` |
| in game (control ids) | `CONTROL_LEG=1 REDSTONE_ONLY=1` | **PASS** + `EXIT:0` (the four ids unknown as blocks and items) — `e2e-control-redstone-0.2.10.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.10.log` |
| fail-capability (button) | `REDSTONE_ONLY=1 BUTTON_SELFTEST=1` | **FAIL** + `EXIT:1` (`button selftest: … reads '…[powered=true]' after the click`) — `e2e-selftest-button-0.2.10.log` |
| fail-capability (plate) | `REDSTONE_ONLY=1 PLATE_SELFTEST=1` | **FAIL** + `EXIT:1` (`plate selftest: … reads '…[powered=true]' under the player`) — `e2e-selftest-plate-0.2.10.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **57 logs, 57 ok**, 0 missing-exit, 0 bad-exit |

The phase-U transcript (the reverse-smelt furnace pair with its no-fuel differential, the four grids
with consumption, the three wrong-ingredient controls plus the one-stone-makes-the-button case, the
four default states with hand 1 / pickaxe 1 and the `otherstone` hand-0 differential, both plates
pressed and released, and both buttons clicked, held and released) and the family's deviations (the
20-tick press, the item-tag fix) are in `notes.md` under "Slice 9".

## 14. SDK 0.2.11 (the frames and pedestals), pin `0a875da`

The slice, the two brief premises it corrected (the frame is an **item**, the pedestal holds **no**
stack — neither has a block entity), the jar's loot-table inventory for the family, and the phase-V
transcript are in `notes.md`, §"Slice 10".

| Step | Command | Result |
|---|---|---|
| red (unit) | `cargo test --offline`, the four tests written before any file | **60 passed; 4 failed** — every family file absent (`...: No such file or directory`), plus `occultism:otherstone_pedestal is not in mineable/pickaxe` — `sdk-test-red-0.2.11.log` |
| green (unit) | `cargo test --offline` | **64 passed, 0 failed** — `sdk-test-0.2.11.log` (copied to `sdk-test.log`, which the gate reads) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy-0.2.11.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt-0.2.11.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 256,975 B, `3d335fe8…d5998` — `sdk-build-0.2.11.log`, `sdk-build-second-0.2.11.log`, `sdk-reproducible-0.2.11.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 256,975 B, `3d335fe802e7668d869bac6ab47ffacb9770b1d9b602cc7fe74805e4fe8d5998`, **514 entries**, no packed `Cargo.lock`, declares **0.2.11**, `src/runtime.rs` and `README.md` inside = outside, 0 empty files — `sdk-artifact-0.2.11.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.11 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check-0.2.11.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits-0.2.11.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` (sha printed) | **`0b14f8a  fits`**, exit 0 — `sdk-fits-next-0.2.11.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** (64 tests, 0 clippy warnings) — `sdk-gate-0.2.11.txt` |
| recipe types | `python3 ports/tools/check-recipe-types.py occultism` | `occultism: ok`, exit 0 — `check-recipe-types-0.2.11.log` |
| recipe keys | `python3 tools/check_recipe_keys.py` | `44 recipe(s), 44 craftable, 0 blocked, 0 failure(s)`, exit 0 — `check-recipe-keys-0.2.11.log` |
| game build | `launcher-game.sh build occultism` (the stale 0.2.10 tag and export removed first) | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-f6977ddf144c`, exported manifest `version = "0.2.11"` — `launcher-build-0.2.11.log` |
| red (in game, the family) | `PEDESTAL_ONLY=1` against the **0.2.10** export | **FAIL** + `EXIT:1` (`the otherstone_frame grid … made None x0`) — `e2e-pedestal-red-0.2.10-package.log` |
| in game (the family) | `PEDESTAL_ONLY=1` | **PASS** + `EXIT:0` — `e2e-pedestal-0.2.11.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen, launcher-built game) | **PASS** + `EXIT:0` (A..V) — `e2e-default-0.2.11.log` |
| in game (control ids) | `CONTROL_LEG=1 PEDESTAL_ONLY=1` | **PASS** + `EXIT:0` (the four ids unknown as a block and an item) — `e2e-control-frames-0.2.11.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.11.log` |
| fail-capability (grid) | `PEDESTAL_ONLY=1 PEDESTAL_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the otherstone_frame grid made occultism:otherstone_frame x1`) — `e2e-negative-pedestal-0.2.11.log` |
| fail-capability (item) | `PEDESTAL_ONLY=1 PEDESTAL_SELFTEST=1` | **FAIL** + `EXIT:1` (`frame selftest: occultism:otherstone_frame left air …`) — `e2e-selftest-frame-0.2.11.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.11.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **65 logs, 65 ok**, 0 missing-exit, 0 bad-exit |

Every leg above ran on the **final** 256,975 B artifact / tree (the first 256,642 B build, before the
README refresh, was replaced and every leg re-run). `E2E_TMP` (`~/.cache/horizon-e2e/scratch`) was
used throughout; nothing was written under `/tmp` except the short probe log quoted in `notes.md`.

## 15. SDK 0.2.12 (the six sacrificial bowls), pin `0a875da`

The slice (the tier's first block entity), the jar's slot shape, what the golden variant changes, the
jar's ten-table loot inventory for the family (all present), the brief's `requires_tool` line
(measured **false**) and the phase-W transcript are in `notes.md`, §"Slice 11".

| Step | Command | Result |
|---|---|---|
| red 1 (unit, the data) | `cargo test --offline`, the family tests written before any file | **68 passed; 5 failed** — every family file absent (`…sacrificial_bowl.json: No such file or directory`), plus the two ingot tags — `sdk-test-red-0.2.12.log` |
| red 2 (unit, the rule) | `bowl_conversion` added returning `None` | **68 passed; 6 failed** (`left: None / right: Some(Conversion { result: "occultism:awakened_feather", count: 1 })`) — `sdk-test-red2-0.2.12.log` |
| green (unit) | `cargo test --offline` | **75 passed, 0 failed** — `sdk-test-0.2.12.log` (copied to `sdk-test.log`, which the gate reads) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy-0.2.12.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt-0.2.12.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 287,175 B, `d8c69505…aa292` — `sdk-build-0.2.12.log`, `sdk-build-second-0.2.12.log`, `sdk-reproducible-0.2.12.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -l` | 287,175 B, `d8c69505e24b2be9012ab6e5ad326735063fbe514bb2c82b90599a1b098aa292`, **558 entries**, no packed `Cargo.lock`, declares **0.2.12**, 0 empty files — `sdk-artifact-0.2.12.txt` |
| package check | `cargo horizon-mod check occultism.hmod` (sha printed) | exit 0, `occultism 0.2.12 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check-0.2.12.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` (sha printed) | `0a875da  made for`, exit 0 — `sdk-fits-0.2.12.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` (sha printed) | **`0b14f8a  fits`**, exit 0 — `sdk-fits-next-0.2.12.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** (75 tests, 0 clippy warnings, package = tree) — `sdk-gate-0.2.12.txt` |
| recipe types | `python3 ports/tools/check-recipe-types.py occultism` | `occultism: ok`, exit 0 — `check-recipe-types-0.2.12.log` |
| recipe keys | `python3 tools/check_recipe_keys.py` | `50 recipe(s), 50 craftable, 0 blocked, 0 failure(s)`, exit 0 — `check-recipe-keys-0.2.12.log` |
| game build | `launcher-game.sh build occultism` (the stale 0.2.11 tag and export removed first) | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-c79386f47e7e`, exported manifest `version = "0.2.12"` — `launcher-build-0.2.12.log` |
| red (in game, the product) | the first 0.2.12 build with `PROBE` lines | **FAIL** + `EXIT:1`: the main hand inserted the sword and the **off hand** took it back out from the same click — `e2e-bowl-probe-0.2.12.log` |
| in game (the family) | `BOWL_ONLY=1` | **PASS** + `EXIT:0` — `e2e-bowl-0.2.12.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen, launcher-built game) | **PASS** + `EXIT:0` (A..W) — `e2e-default-0.2.12.log` |
| in game (control ids) | `CONTROL_LEG=1 BOWL_ONLY=1` | **PASS** + `EXIT:0` (the six ids unknown as a block and an item) — `e2e-control-bowls-0.2.12.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME=` | **FAIL** + `EXIT:1` (`the datura item did not register`) — `e2e-control-0.2.12.log` |
| fail-capability (grid) | `BOWL_ONLY=1 BOWL_NEGATIVE=1` | **FAIL** + `EXIT:1` (`negative: the sacrificial_bowl grid made occultism:sacrificial_bowl x1, expected nothing`) — `e2e-negative-bowl-0.2.12.log` |
| fail-capability (take-out) | `BOWL_ONLY=1 BOWL_SELFTEST=1` | **FAIL** + `EXIT:1` (`bowl selftest: minecraft:diamond_sword returned to the hand`) — `e2e-selftest-bowl-0.2.12.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (3 awakened feathers ≠ the flipped 2) — `e2e-selftest-0.2.12.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **72 logs, 72 ok**, 0 missing-exit, 0 bad-exit |

Every leg above ran on the final 287,175 B artifact / tree. `E2E_TMP` (`~/.cache/horizon-e2e/scratch`)
was used throughout; nothing was written under `/tmp` except the probe scripts.

## 5. SDK 0.2.13 — the 17 impure-chalk craftings

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`). The family,
the tags and the in-game transcript are in `notes.md`, §"Slice 12". Every step ran in a
`ports/tools/build-slot.sh` lease (the check/fits/build steps with `BUILD_LOCAL=1`: the far box has
no `cargo-horizon-mod`), `CARGO_INCREMENTAL=0`, no `--target-dir`.

| Step | Command | Result |
|---|---|---|
| red (unit) | `cargo test --offline`, the four family tests written before any file | **75 passed; 4 failed**, each on a missing file (`…chalk_base.json: No such file or directory`) — `sdk-test-red-0.2.13.log` |
| green (unit) | `cargo test --offline` | **79 passed, 0 failed** — `sdk-test-0.2.13.log` (copied to `sdk-test.log`, which the gate reads) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy-0.2.13.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt-0.2.13.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 309,344 B, `8dc4b063…ba09` — `sdk-build-0.2.13.log`, `sdk-build-second-0.2.13.log`, `sdk-reproducible-0.2.13.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -Z1 \| wc -l` | 309,344 B, `8dc4b0633c3ad413217ac673b085ff8f98fa880992eea8052c8c06f2d29bba09`, **629 entries**, 17 impure recipes packed, no packed `Cargo.lock`, declares **0.2.13**, 0 empty files — `sdk-artifact-0.2.13.txt` |
| package check | `cargo horizon-mod check occultism.hmod` | exit 0, `occultism 0.2.13 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check-0.2.13.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` | `0a875da  made for`, exit 0 — `sdk-fits-0.2.13.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` | **`0b14f8a  fits`**, exit 0 — `sdk-fits-next-0.2.13.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** (79 tests, 0 clippy warnings, package = tree) — `sdk-gate-0.2.13.txt` |
| recipe types | `python3 ports/tools/check-recipe-types.py occultism` | `occultism: ok`, exit 0 — `check-recipe-types-0.2.13.log` |
| recipe keys | `python3 tools/check_recipe_keys.py` | `69 recipe(s), 69 craftable, 0 blocked, 0 failure(s)`, exit 0 — `check-recipe-keys-0.2.13.log` |
| game build | `launcher-game.sh build occultism` | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-b7522f162df2` — `launcher-build-0.2.13.log` |
| in game (the family) | `CHALK_ONLY=1` | **PASS** + `EXIT:0` (17 grids, the nested tag's both members, the two smelting recipes, no `invalid ingredient`) — `e2e-chalk-0.2.13.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen, launcher-built game) | **PASS** + `EXIT:0` (A..X) — `e2e-default-0.2.13.log` |
| in game (control ids) | `CONTROL_LEG=1 CHALK_ONLY=1` | **PASS** + `EXIT:0` (16 chalk + 5 ingredient ids unknown as items) — `e2e-control-chalks-0.2.13.log` |
| in game (control) | `GAME=$CONTROL_GAME CHALK_ONLY=1` | **FAIL** + `EXIT:1` (`CHALK FAIL: missing ingredient occultism:burnt_otherstone`) — `e2e-control-0.2.13.log` |
| fail-capability (this family) | `CHALK_ONLY=1 CHALK_SELFTEST=1` | **FAIL** + `EXIT:1` (the first grid made x1, the flipped assertion wants x2) — `e2e-selftest-chalk-0.2.13.log` |
| fail-capability (scenario) | `SELFTEST=1` | **FAIL** + `EXIT:1` (the 0.2.12 phase-A flip, unchanged) — `e2e-selftest-0.2.12.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **77 logs, 77 ok**, 0 missing-exit, 0 bad-exit |

Every leg above ran on the final 309,344 B artifact / tree. `E2E_TMP`
(`~/.cache/horizon-e2e/scratch`) was used throughout; the only scratch file outside it is the
one-off `/tmp/opencode/gen_impure_chalks.py` that translated the 17 jar recipes (not part of the
port).

## 6. SDK 0.2.14 — the otherstone tablet

Pin **`0a875da`** (SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins `e59da90`). The family,
the jar evidence and the in-game transcript are in `notes.md`, §"Slice 13". Every `run` step below
used `build-slot.sh run --job G11-occultism --port ports/occultism --step <…> -- <cmd>` with the
**relative** `--port`; each printed `build-slot: forwarding <step> for ports/occultism to the
remote runner` and ran on **AdLf-PC**. No `BUILD_LOCAL=1`, no `--target-dir`, no private
`BUILD_SLOTS_DIR`, `CARGO_INCREMENTAL=0`. The e2e legs stayed local on slot 1/3/5 (the
launcher-migrated route: `build-slot.sh e2e` prepends the retired `tools/horizon.py`), with
`E2E_TMP` from `launcher-game.sh env` for every scratch path.

| Step | Command | Result |
|---|---|---|
| red (unit) | `cargo test --offline`, the two family tests written before any file | **79 passed; 2 failed**, each on a missing tablet file (`…otherstone_tablet.json: No such file or directory`) — `sdk-test-red-0.2.14.log` |
| green (unit) | `cargo test --offline` | **81 passed, 0 failed** — `sdk-test.log` (regenerated for 0.2.14, which the gate reads) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy-0.2.14.log` |
| format | `cargo fmt --check` | exit 0 — `sdk-fmt-0.2.14.log` |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build` twice, `cmp` | **`REPRODUCIBLE`**: 313,735 B, `bfef6df2…eeebfc` — `sdk-build-0.2.14.log`, `sdk-build-second-0.2.14.log`, `sdk-reproducible-0.2.14.txt` |
| package | `sha256sum`, `stat -c%s`, `unzip -Z1 \| wc -l` | 313,735 B, `bfef6df24eef45495c74318b30515993e94caf608662b6f9068b799a6eeeebfc`, **634 entries**, both tablet recipes packed, no packed `Cargo.lock`, declares **0.2.14**, 0 empty json — `sdk-artifact-0.2.14.txt` |
| runtime inside/outside | `unzip -p … src/runtime.rs \| sha256sum` vs the tree | identical, `75456d98…f8479c` — `sdk-artifact-0.2.14.txt` |
| tree match | `verify-sdk-package.py` | **package matches the tree** (manifest, README and lang edited before the final build) — `sdk-gate-0.2.14.txt` |
| package check | `cargo horizon-mod check occultism.hmod` | exit 0, `occultism 0.2.14 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` — `sdk-check-0.2.14.log` |
| fits (pin) | `cargo horizon-mod fits occultism.hmod --build 0a875da91dbb…` | `0a875da  made for`, exit 0 — `sdk-fits-0.2.14.log` |
| fits (another build) | `cargo horizon-mod fits occultism.hmod --build 0b14f8acfd97…` | **`0b14f8a  fits`**, exit 0 — `sdk-fits-next-0.2.14.log` |
| gate | `python3 ports/tools/verify-sdk-package.py occultism` | **1/1** (81 tests, 0 clippy warnings, package = tree, R9 tables/deps, no lock) — `sdk-gate-0.2.14.txt` |
| recipe types | `python3 ports/tools/check-recipe-types.py occultism` | `occultism: ok`, exit 0 — `check-recipe-types-0.2.14.log` |
| recipe keys | `python3 tools/check_recipe_keys.py` | **`71 recipe(s), 71 craftable, 0 blocked, 0 failure(s)`** — `check-recipe-keys-0.2.14.log` |
| game build | `launcher-game.sh build occultism` (every `native-*` tag carrying this mod removed first — none remained) | **`Compiling occultism (1 of 1)`**, tag `native-0a875da-78d415e66bb2`, exported manifest `version = "0.2.14"` — `launcher-build-0.2.14.log` |
| in game (the family) | `TABLET_ONLY=1` | **PASS** + `EXIT:0` (both grids, slot-200 result, consumption, grid empty, three ashes make nothing, no `invalid ingredient`) — `e2e-tablet-0.2.14.log` |
| in game (all phases) | `bash tools/e2e.sh` (offscreen, launcher-built game) | **PASS** + `EXIT:0` (A..Y) — `e2e-default-0.2.14.log` |
| in game (control ids) | `CONTROL_LEG=1 TABLET_ONLY=1` | **PASS** + `EXIT:0` (the tablet and ashes unknown as items) — `e2e-control-ids-tablet-0.2.14.log` |
| in game (control) | `GAME=$CONTROL_GAME CONTROL_GAME= TABLET_ONLY=1` | **FAIL** + `EXIT:1` (`TABLET FAIL: missing ingredient occultism:otherworld_ashes`) — `e2e-control-tablet-0.2.14.log` |
| fail-capability (this family) | `TABLET_ONLY=1 SELFTEST=tablet` | **FAIL** + `EXIT:1` (grid made `occultism:otherstone_tablet x1`, the flipped assertion wants x2) — `e2e-selftest-tablet-0.2.14.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py occultism-e2e --strict` | **82 logs, 82 ok**, 0 missing-exit, 0 bad-exit |

Every leg above ran on the final 313,735 B artifact / tree. `E2E_TMP`
(`~/.cache/horizon-e2e/scratch`) was used throughout; no leg wrote under `/tmp`.
