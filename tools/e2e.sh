#!/usr/bin/env bash
# In-game check for Occultism slice 1: the spirit-fire conversion loop.
#
# This scenario is unchanged from 0.1.0; only the **launch** moved, because the
# 2026-11-18 SDK package is a native `.hmod` built and run by `horizon-launcher`
# (mod API 0.2), not by a staged `tools/horizon.py` build with `--dev-mod content`.
# The launcher is not installed in the campaign environment, so **no 0.2.0 run is
# claimed**: the 0.1.0 runs in `ports/_shared/evidence/occultism/` remain the
# behaviour's evidence for a `src/runtime.rs` that is byte-identical (see notes.md).
#
# Requirements:
#   GAME=...          a game built by `horizon-launcher native-build .` for this
#                     package (the launcher compiles it into the game).
#   CTL=...           a `horizon-ctl` built from the same SDK build (default:
#                     $PORT/target/launcher/horizon-ctl).
#   CONTROL_GAME=...  a game **without** this package: the control leg is
#                     `GAME=$CONTROL_GAME CONTROL_GAME= bash tools/e2e.sh` (the
#                     datura item, its seeds and both blocks must be unknown).
#                     CONTROL_LEG=1 also runs it first on the same socket, which
#                     destabilises the long hand-break phase (see below).
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs the
# console path and bypasses the command hook. Item entities are summoned at exact
# positions (upstream's own flow is "throw it on the ground"; the harness only
# needs the entity there, and a throw's landing spot is not deterministic).
#
# The End, in the void at x=200 (away from the central island and its ambient
# item entities), on a stone floor at y=100 with clear air above:
#   z=0   the conversion station: datura -> spirit fire -> feathers converted;
#   z=4   the control: the same feather on plain stone stays a feather;
#   z=-4  the recolour station: a gold chalk paints the fire yellow;
#   z=8   the far-datura case: datura two blocks out must not be consumed and
#         must not light spirit fire (vanilla still lights its own fire);
#   z=-8  the spectator probe: a spectator's chalk must not recolour the fire,
#         a spectator's lighter must not light one nor consume the datura;
#         z=-12 is the spectator ignition probe. A survival player then does the
#         same two clicks and both work (the differential: the guard refuses the
#         spectator, not everyone).
#
# `/occultism info` is operator-only (review finding 1): an operator reads the
# fire, a non-fire position answers "not a spirit fire" (no block id), and a LAN
# guest at permission level 0 is refused and receives no readout.
#
# The datura crop (0.2.3) has its own stations east of the fire arena:
#   x=221..230, z=0   a hydrated farmland strip: the crop's eight ages, the seed
#                     planting (farmland and the stone control) and its growth
#                     under `random_tick_speed` 0 vs 400;
#   x=221..230, z=6   the ripe-crop right-click harvest (empty hand, replanted)
#                     and the young-crop control;
#   x=221..230, z=10  the loot table by mining: a ripe crop drops 1 fruit and
#                     1..4 seeds, a young one 1 seed and no fruit;
#   x=216..220, z=14..18  a 5x5 short_grass patch, re-filled between rounds: a
#                     hand break has the jar's 2% chance of a seed (stopping at
#                     the first), and the same patch sheared drops the grass
#                     itself and no seed — the differential for the modifier's
#                     shears condition.
#
# The two carried crafting-table recipes (0.2.5) craft at a real table south of
# the farm stations (phase P): the jar's 3x3 `ppp/ppp/ppp` whose key is the
# alternative `#c:crops/datura` / `#c:seeds/datura` makes `occultism:demons_dream_essence`
# (nine fruit, nine seeds, then a mixed five-fruit/four-seed grid), and the
# shapeless awakened feather + purified ink + taboo book makes
# `occultism:book_of_binding_empty`. Nine dirt, and eight fruit with one dirt, make
# nothing - the differential. Each run fills the grid one item per cell, reads the
# result slot, takes it and reads the inventory back (ingredients consumed).
# CRAFT_NEGATIVE=1 inverts the fruit assertion, so the run must FAIL.
# CRAFT_ONLY=1 launches, lays the arena and runs only phase P.
#
# The four carried block loot tables (0.2.4) are a row beyond the crop stations:
#   x=232,234, z=30  otherstone and otherrock (`requires_tool: true`): a hand
#                    break drops nothing, a wooden pickaxe break drops exactly
#                    one of the block — the pair is the fail-capable check;
#   x=236,238, z=30  otherflower and otherworld_sapling_natural on dirt: a bare
#                    hand drops exactly one. Each block is placed, its state read
#                    back, broken in survival, and the `ctl break` reply's
#                    `.drops` asserted.
#
# The six otherstone/otherrock building blocks (0.2.6) are phase Q: their six
# recipes at the phase-P table (three stones in the middle row -> six slabs, the
# stair step -> four, two rows -> six, plus a three-dirt differential), then each
# block placed, its default state read back, a hand break (no drop) and a wooden
# pickaxe break (exactly one), two walls connecting, and a double slab dropping
# two. BUILDING_ONLY=1 is this family's red/green leg; BUILDING_NEGATIVE=1
# inverts the slab assertion, so the run must FAIL.
#
# The bricks tier (0.2.7) is phase R, on the row north of those blocks and at the
# same table: the ten bricks-tier grids (a 2x2 of the stone -> four bricks, the
# brick slab/stair/wall recipes, two brick slabs -> one chiseled), a chain in one
# opened table (twelve stones -> bricks -> six bricks -> four stairs -> three
# bricks -> six slabs -> two slabs -> one chiseled), the cracked pair smelting in
# a real furnace with a no-fuel differential, then each of the twelve blocks
# placed, its default state read back, a hand break (no drop) and a wooden
# pickaxe break (exactly one), a brick wall pair connecting, and a double brick
# slab dropping two. BRICKS_ONLY=1 is this family's red/green leg;
# BRICKS_NEGATIVE=1 inverts the bricks-grid assertion, so the run must FAIL.
#
# The polished tier (0.2.8) is phase S, on the same row and at the same table
# after phase R has emptied them: the six polished-tier grids (three polished
# blocks in the middle row -> six slabs, the stair step -> four, two rows -> six),
# with a 2x2 of polished blocks as the differential, the polished pair
# smelted from the raw stone in a real furnace (200 ticks, 0.5 xp) with a
# no-fuel differential, then each of the eight blocks placed, its default state
# read back, a hand break (no drop) and a wooden pickaxe break (exactly one), a
# polished wall pair connecting, a double slab dropping two, and
# polished_otherrock_slab — the jar's third missing table — dropping one.
# POLISHED_ONLY=1 is this family's red/green leg; POLISHED_NEGATIVE=1 inverts the
# polished slab grid assertion, so the run must FAIL.
#
# The cobblestone tier (0.2.9) is phase T, on the polished row and at the same
# table: the six cobble grids (three cobble blocks in the middle row -> six slabs,
# the stair step -> four, two rows -> six), with a 2x2 of the cobble as the
# differential; then each of the eight blocks placed, its default state read back,
# a hand break (no drop) and a wooden pickaxe break (exactly one), a cobble wall
# pair connecting, a double cobble slab dropping two, and othercobblerock_slab —
# the jar's fourth missing table — dropping one. It also proves the plain pair's
# obtainability: otherstone/otherrock broken with a wooden pickaxe drop one
# othercobblestone/othercobblerock (the jar's alternatives table), and the same
# block broken with a silk-touch pickaxe keeps the stone itself.
# COBBLE_ONLY=1 is this family's red/green leg; COBBLE_NEGATIVE=1 inverts the
# cobble slab grid assertion, so the run must FAIL.
#
# The redstone pair (0.2.10) is phase U, on the cobble row and at the same table:
# the two buttons (one stone in the grid -> one button, shapeless) and the two
# pressure plates (two stones in one row -> one plate), with a cobble button
# grid and a cobble plate grid as wrong-ingredient differentials and one stone in
# the plate grid making the button rather than the plate (the plate needs two); the jar's reverse smelt pair (cobble -> stone, 200 ticks,
# 0.5 xp) in a real furnace with a no-fuel differential; then each of the four
# blocks placed, its default state read back, a bare hand (exactly one — these
# templates are `requires_tool: false`) and a wooden pickaxe (exactly one), with
# a bare-hand break of `otherstone` (nothing) as the differential that makes the
# hand leg fail-capable; then a **press** (a plate under the player reads
# `powered=true`, `false` after stepping off) and a **click** (a button reads
# `powered=true`, is still pressed ten ticks later, and `false` after the
# template's press duration). REDSTONE_ONLY=1 is this family's red/green leg;
# PLATE_SELFTEST=1 and BUTTON_SELFTEST=1 each invert their own leg, so those runs
# must FAIL.
#
# The frames and pedestals (0.2.11) are phase V, on the last floor row (z=50) and
# at the same table: eight stones in a ring (the centre empty) make one otherstone
# or otherrock frame, five slabs + one stone make one pedestal (the otherrock one,
# whose file is `otherrock_pedestal.json`, makes `otherstone_pedestal_silver`),
# with a ninth stone in the frame's hole and one slab short of the pedestal as the
# wrong-shape differentials; the frames are **items** — `give` puts one in the
# inventory and a `setblock` of the id must leave air; then each pedestal placed,
# its state read back, a bare hand (nothing: `requires_tool` is true on the
# `minecraft:stone` template) and a wooden pickaxe (exactly one), with a
# bare-hand break of `otherstone` (nothing) as the differential. PEDESTAL_ONLY=1
# is this family's red/green leg; PEDESTAL_NEGATIVE=1 flips the frame grid and
# PEDESTAL_SELFTEST=1 flips the item leg, so both runs must FAIL.
#
# The sacrificial bowls (0.2.12) are phase W, on the last floor row (z=44) and at
# the same table: the six grids (five stones in a 5-cell ring -> one plain or dark
# bowl, eight gold ingots around that bowl -> the golden pair, a bowl + a copper
# ingot (shapeless) -> the copper pair) with four-stone and seven-gold grids as the
# wrong-shape differentials; each of the six blocks placed and read back; the
# storage round trip on a plain and a golden bowl with a **damaged diamond sword**
# (right-click puts it in, `/occultism info` and `/data get block` read it back
# through the block entity, an empty-hand right-click returns the same stack with
# its component); five dirt put one in the bowl and leave four in the hand (the
# jar's slot limit of one); breaking a full bowl drops the bowl and spills its
# stack; a feather in a plain bowl over spirit fire becomes an awakened feather
# while the same in a golden bowl stays a feather (upstream's `instanceof` check);
# a sneaking click changes nothing; and every bowl drops exactly one to a bare hand
# **and** to a wooden pickaxe (`requires_tool` is false — the `minecraft:stone`
# template's true would have made the hand break drop nothing), with otherstone's
# bare-hand break (nothing) as the differential. BOWL_ONLY=1 is this family's
# red/green leg; BOWL_NEGATIVE=1 flips the first grid and BOWL_SELFTEST=1 flips the
# take-out leg, so both runs must FAIL.
#
# The impure chalks (0.2.13) are phase X, at the same table: the 17 crafting
# grids the jar ships (15 shapeless colour grids, plus the two mirrored shaped
# white grids over the nested `#occultism:dusts/chalk_base` tag and
# `#c:dusts/otherworld_wood`), each filled exactly as its recipe declares, the
# result read back, taken, and the grid emptied with every ingredient consumed.
# The two white grids fill the nested chalk base with **both** its members
# (burnt_otherstone and burnt_otherrock), so a nested tag that resolves to
# nothing cannot pass; the phase then greps the game log for `invalid
# ingredient`, which is what a tag-keyed cell that did not resolve leaves behind.
# A wrong-ingredient pink grid (dirt for one demonic meat) and a shaped grid with
# the two chalk-base columns and no otherworld wood make nothing; finally the two
# carried smelting recipes make burnt otherstone/otherrock from the carried
# polished pair in a real furnace (200 ticks, 0.15 experience) while a fuel-less
# one stays idle.
# CHALK_ONLY=1 is this family's red/green leg; CHALK_SELFTEST=1 flips the first
# grid's count, so a working package must FAIL.
#
# The otherstone tablet (0.2.14) is phase Y, at the same table: the jar's two
# shaped grids (`aga`/`gsg`/`aga`) — four #c:dusts/otherworld_wood (ashes), four
# #c:ingots/gold and one otherstone (the first grid) or otherrock (the second) —
# each make exactly one occultism:otherstone_tablet; the result is taken, every
# ingredient is consumed and the grid is emptied. A three-ashes grid makes
# nothing, and the game log must carry no `invalid ingredient` line. The result
# is read from **slot 200 of the table screen** (a plain inventory screen has no
# such slot). TABLET_ONLY=1 is this family's red/green leg; `SELFTEST=tablet`
# (TABLET_SELFTEST=1) flips the first grid's count, so a working package must
# FAIL. It is a tablet-only mode, not a second full sweep.
#
# All 32 carried stonecutting recipes (0.2.16: 0.2.15 proved only the two slabs)
# are cut at a **real** stonecutter beside the table (phase Z). The 32 ids are
# enumerated from `data/occultism/recipe/stonecutting/*.json` by the harness, not
# from the brief. Each declared ingredient goes into station slot 600, its jar
# recipe is selected by a GUI cell click (`ctl click-gui`), and slot 650's result
# plus the consumed input are read through `ctl menu` — the stonecutter has no
# block entity at this pin, so its container *is* the guarded station menu (a
# plain inventory screen has no station slots; `ctl menu` lists only non-empty
# slots, so the guard is the station name). A different selected cell must yield
# a different result and vanilla stone offers no occultism recipe. A
# `ctl container` probe is kept in the log for the transcript. STONECUT_ONLY=1
# runs this phase alone; `SELFTEST=stonecut-all` (STONECUT_SELFTEST=stonecut-all)
# flips the bricks cut's count from one to two, so a working package must FAIL
# that run. `SELFTEST=stonecut` (STONECUT_SELFTEST=1) is the older alias.
#
# SELFTEST=1 flips the count assertion (3 awakened feathers -> 2): it must FAIL,
# proving the scenario can fail. NEGATIVE=1 flips the ripe-crop harvest's
# assertion (a ripe crop must drop a fruit) the same way; LOOT_NEGATIVE=1 is the
# loot family's own counterfactual (a wooden pickaxe break of otherstone must drop
# no othercobblestone, so it must FAIL).
#
# LOOT_ONLY=1 launches, lays the arena and runs only phase O — the red/green leg
# for the loot family. Every scratch path honours `E2E_TMP` (the campaign's
# `/tmp` is a 7.5 GB tmpfs).
set -Eeuo pipefail
export LC_ALL=C
[ -z "${OCC_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/occultism
GAME=${GAME:-}
CTL=${CTL:-$PORT/target/launcher/horizon-ctl}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
# The campaign's /tmp is a 7.5 GB tmpfs; `launcher-game.sh env` exports E2E_TMP
# under ~/.cache, so every scratch path here follows it.
TMP=${E2E_TMP:-/tmp/opencode}
SOCK=${SOCK:-$TMP/occultism-e2e.sock}
GSOCK=${GSOCK:-$TMP/occultism-e2e-guest.sock}
SELFTEST=${SELFTEST:-0}
NEGATIVE=${NEGATIVE:-0}
# The loot family's own counterfactual: the pickaxe break must drop nothing, so
# this run MUST fail (see phase O).
LOOT_NEGATIVE=${LOOT_NEGATIVE:-0}
# `LOOT_ONLY=1` launches and runs only the four-loot-table phase (the red/green
# leg for that family), then exits 0.
LOOT_ONLY=${LOOT_ONLY:-0}
# The crafting family's own counterfactual: a 3x3 grid of datura fruit must make
# nothing, so this run MUST fail (see phase P).
CRAFT_NEGATIVE=${CRAFT_NEGATIVE:-0}
# `CRAFT_ONLY=1` launches and runs only the crafting phase (the red/green leg for
# that family), then exits 0.
CRAFT_ONLY=${CRAFT_ONLY:-0}
# The building family's own counterfactual: a middle row of three stones must
# make nothing, so this run MUST fail (see phase Q).
BUILDING_NEGATIVE=${BUILDING_NEGATIVE:-0}
# `BUILDING_ONLY=1` launches and runs only the six-building-block phase (the
# red/green leg for that family), then exits 0.
BUILDING_ONLY=${BUILDING_ONLY:-0}
# The bricks tier's own counterfactual: a 2x2 of otherstone must make nothing, so
# this run MUST fail (see phase R).
BRICKS_NEGATIVE=${BRICKS_NEGATIVE:-0}
# `BRICKS_ONLY=1` launches and runs only the bricks-tier phase (the red/green leg
# for that family), then exits 0.
BRICKS_ONLY=${BRICKS_ONLY:-0}
# The polished tier's own counterfactual: three polished blocks in the middle row
# must make nothing, so this run MUST fail (see phase S).
POLISHED_NEGATIVE=${POLISHED_NEGATIVE:-0}
# `POLISHED_ONLY=1` launches and runs only the polished-tier phase (the red/green
# leg for that family), then exits 0.
POLISHED_ONLY=${POLISHED_ONLY:-0}
# The cobblestone tier's own counterfactual: the cobble slab grid must make
# nothing, so this run MUST fail (see phase T).
COBBLE_NEGATIVE=${COBBLE_NEGATIVE:-0}
# `COBBLE_ONLY=1` launches and runs only the cobblestone-tier phase (the red/green
# leg for that family), then exits 0.
COBBLE_ONLY=${COBBLE_ONLY:-0}
# `REDSTONE_ONLY=1` launches and runs only the redstone-pair phase (the red/green
# leg for that family), then exits 0.
REDSTONE_ONLY=${REDSTONE_ONLY:-0}
# The redstone pair's own counterfactuals: `PLATE_SELFTEST=1` asserts a plate is
# NOT powered under the player, and `BUTTON_SELFTEST=1` asserts a clicked button
# is NOT pressed. Both must fail (see phase U).
PLATE_SELFTEST=${PLATE_SELFTEST:-0}
BUTTON_SELFTEST=${BUTTON_SELFTEST:-0}
# The frames-and-pedestals family's own counterfactual: the otherstone frame grid
# must make nothing, so this run MUST fail (see phase V).
PEDESTAL_NEGATIVE=${PEDESTAL_NEGATIVE:-0}
# `PEDESTAL_SELFTEST=1` asserts the otherstone frame id **does** place a block, so
# a working package (the frame is an item) must fail this run (see phase V).
PEDESTAL_SELFTEST=${PEDESTAL_SELFTEST:-0}
# `PEDESTAL_ONLY=1` launches and runs only the frames-and-pedestals phase (the
# red/green leg for that family), then exits 0.
PEDESTAL_ONLY=${PEDESTAL_ONLY:-0}
# The bowls' own counterfactual: the plain bowl grid must make nothing, so this run
# MUST fail (see phase W).
BOWL_NEGATIVE=${BOWL_NEGATIVE:-0}
# `BOWL_SELFTEST=1` asserts the take-out click does **not** return the stored
# stack, so a working package must fail this run (see phase W).
BOWL_SELFTEST=${BOWL_SELFTEST:-0}
# `BOWL_ONLY=1` launches and runs only the sacrificial-bowls phase (the red/green
# leg for that family), then exits 0.
BOWL_ONLY=${BOWL_ONLY:-0}
# `CHALK_SELFTEST=1` asserts the first impure-chalk grid makes two, so a working
# package must fail this run (see phase X).
CHALK_SELFTEST=${CHALK_SELFTEST:-0}
# `CHALK_ONLY=1` launches and runs only the impure-chalk phase (the red/green leg
# for that family), then exits 0.
CHALK_ONLY=${CHALK_ONLY:-0}
# `SELFTEST=tablet` is the tablet family's own failure mode: it flips the first
# grid's expected output, so a working package must FAIL this run.
TABLET_SELFTEST=${TABLET_SELFTEST:-0}
case "$SELFTEST" in
    tablet) TABLET_SELFTEST=1; SELFTEST=0 ;;
esac
# `TABLET_ONLY=1` launches and runs only the tablet phase (the red/green leg for
# that family), then exits 0.
TABLET_ONLY=${TABLET_ONLY:-0}
# `SELFTEST=stonecut-all` (STONECUT_SELFTEST=stonecut-all) flips the bricks cut's
# expected result from one to two, so a working package must FAIL this run.
# `SELFTEST=stonecut` (STONECUT_SELFTEST=1) is the older alias.
STONECUT_SELFTEST=${STONECUT_SELFTEST:-0}
case "$SELFTEST" in
    stonecut) STONECUT_SELFTEST=1; SELFTEST=0 ;;
    stonecut-all) STONECUT_SELFTEST=stonecut-all; SELFTEST=0 ;;
esac
# `STONECUT_ONLY=1` launches, lays the arena and runs only phase Z (the red/green
# leg for the 32 stonecutting recipes), then exits 0.
STONECUT_ONLY=${STONECUT_ONLY:-0}
# `CONTROL_LEG=1` runs the packaged leg first on the same socket, as 0.2.0/0.2.2 did.
# The default is a *separate* invocation (`GAME=$CONTROL_GAME`), because a control
# launch in the same HORIZON_HOME leaves the main game without its log file and the
# long hand-break phase then stops landing breaks after ~100 (measured 2026-11-21).
CONTROL_LEG=${CONTROL_LEG:-0}
KEEP=${KEEP:-0}
WORLD=OccultismE2E
CONTROL_WORLD=OccultismControl
# Where the arena stands: the End's void, far from the central island.
X0=200
Z_CONVERT=0
Z_CONTROL=4
Z_PAINT=-4
Z_FAR=8
Z_SPEC=-8
Z_SPEC_LIGHT=-12
Z_SEED=14
# The crop family's stations (see the header).
CROP_X0=221
CROP_Z=0
Z_HARVEST=6
Z_BREAK=10
CROP=occultism:datura
SEEDS=occultism:datura_seeds
STATE_RIPE="${CROP}[age=7]"
STATE_YOUNG="${CROP}[age=0]"
# The loot-table row (phase O): solid blocks at x=232/234 on the stone floor,
# the flower and the sapling at x=236/238 on their own dirt.
BLOCK_Z=30
# The crafting station (phase P): a table on the stone floor, south of the crops.
CRAFT_X=222
CRAFT_Z=38
# The building family (phase Q): six blocks in a row north of the crop stations,
# the wall pair three blocks north of them. The six recipes craft at the phase-P
# station, whose table the driver lays itself.
BUILDING_Z=33

if [ -z "$GAME" ]; then
    echo "GAME is unset: pass a game built by 'horizon-launcher native-build .'" >&2
    echo "for this package (no launcher is installed in the campaign environment," >&2
    echo "so this leg is a gap in notes.md, not a 0.2.0 run)" >&2
    exit 2
fi
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "$TMP/occultism-home.XXXXXX")
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch GAME WORLD
    local game=$1 world=$2
    rm -f "$SOCK"
    cd "$SCRATCH"
    # A native SDK package carries its code and its content in the one `.hmod`;
    # the launcher-built game already has it, so there is no `--dev-mod`.
    # `--offscreen`, not `--headless`: the crafting phase (P) opens a
    # crafting-table menu and clicks its grid slots, and a slot click needs the
    # screen's rendered layout (`horizon-ctl click-slot` answers "no renderer
    # yet", and `screen` times out, under `--headless` — measured 2026-11-21 in
    # this port's first phase-P run). Nothing else in the scenario depends on
    # the renderer.
    "$CTL" --socket "$SOCK" launch --game "$game" --world "$world" --seed 20261105 \
        --creative --offscreen --wait --resources "$RES" >>"$SCRATCH/launch-$world.log" 2>&1
    ctl wait-ready 120
    ctl wait 40
}

block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // empty'; }
# `ctl block` reports the whole state (`occultism:spirit_fire[color=white]`).
block_colour() { ctl block "$1" "$2" "$3" | jq -r '.state // empty' | sed -nE 's/.*color=([a-z_]+).*/\1/p'; }

# How many item entities of `ITEM` are in `RADIUS`, and how many items they hold.
count_items() { # count_items ITEM [RADIUS]
    ctl entities "${2:-8}" item \
        | jq -r --arg i "$1" \
            '[.entities[] | select(.kind == "item" and .item.item == $i) | .item.count] | add // 0'
}

summon_item() { # summon_item ITEM COUNT X Y Z
    ctl command "execute in minecraft:the_end run summon minecraft:item $3 $4 $5 {Item:{id:\"$1\",count:$2}}"
    ctl wait 3 >/dev/null
}

arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 104 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    # Big enough for the fire stations (z -12..8), the crop family
    # (x 214..233, z 0..40), the loot row (x 232..238, z 30), the bricks row
    # (x 214..236, z 46..50, phases R..U, with phase U's furnace row at z=44 and
    # its press/click stations at z=48), with clear air above all of them.
    ctl command "execute in minecraft:the_end run fill 196 100 -16 242 100 52 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill 196 101 -16 242 108 52 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl wait 20 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 6 >/dev/null
    [ "$(block_name "$X0" 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_name "$X0" 101 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# Helpers for the crop family: a block write in the End, a state read-back, the
# whole inventory's count of one item (a broken block's drop is picked up if the
# player is close), and one `ctl break` reply's drop count.
end() { ctl command "execute in minecraft:the_end run $*" >/dev/null; }
state_at() { ctl block "$1" "$2" "$3" | jq -r '.state // "minecraft:air"'; }
inv_count() { ctl inventory | jq -r --arg i "$1" '.counts[$i] // 0'; }
drops_of() { # drops_of REPLY ITEM -> that item's count in the break reply
    local reply=${1:-}
    [ -n "$reply" ] || reply='{}'
    jq -r --arg i "$2" '(.drops // {})[$i] // 0' <<<"$reply"
}
age_at() { state_at "$1" "$2" "$3" | sed -nE 's/.*age=([0-9]+).*/\1/p'; }

# Mine one block from one block away (a crop's outline is a cross: from two
# blocks the client's ray can pass beside it), with one retry aimed at the block.
mine() { # mine X Y Z -> the ctl break reply
    ctl tp "$(($1 + 1)).5" 101 "$(($3)).5" >/dev/null
    ctl wait 3 >/dev/null
    ctl look-at-block "$1" "$2" "$3" up >/dev/null
    ctl wait 2 >/dev/null
    ctl break "$1" "$2" "$3" 2>/dev/null && return 0
    ctl look-at-block "$1" "$2" "$3" up >/dev/null
    ctl wait 3 >/dev/null
    ctl break "$1" "$2" "$3"
}

# Middle ground: mine a block from two blocks away, so the drop lands outside the
# player's pickup box (vanilla's is inflated by one block) before the reply
# collects it, and retry once — the client occasionally misses the first swing on
# a fresh target.
break_at() { # break_at X Y Z -> the ctl break reply (the last attempt's, only)
    local x=$1 y=$2 z=$3 reply
    ctl command "execute in minecraft:the_end run tp @s $((x)).5 101 $((z + 2)).5" >/dev/null
    ctl wait 4 >/dev/null
    ctl look-at-block "$x" "$y" "$z" up >/dev/null
    ctl wait 2 >/dev/null
    reply=$(ctl break "$x" "$y" "$z" 2>/dev/null) && { echo "$reply"; return 0; }
    ctl look-at-block "$x" "$y" "$z" up >/dev/null
    ctl wait 3 >/dev/null
    ctl break "$x" "$y" "$z"
}

# Right-click a block with whatever the hand holds, from three blocks away: within
# reach (4.5), and outside the item pickup box, so a harvest's drops stay on the
# ground for the count (`ctl entities`) instead of moving into the inventory
# between two snapshots (that race read `fruit=0` once, 2026-11-21).
click_block() { # click_block X Z
    local x=$1 z=$2
    ctl tp "$((x)).5" 101 "$((z + 3)).5" >/dev/null
    ctl wait 5 >/dev/null
    ctl look-at-block "$x" 101 "$z" up >/dev/null
    ctl wait 2 >/dev/null
    ctl interact "$x" 101 "$z" up >/dev/null
    ctl wait 5 >/dev/null
}

# `/occultism info X Y Z`: the line, then the parsed fields in BLOCK, COLOUR,
# LAST, RULES, FILES.
info() { # info X Y Z
    say "occultism info $1 $2 $3"
    ctl wait 12 >/dev/null
    INFO_LINE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Spirit Fire at' || true)
    [ -n "$INFO_LINE" ] || fail "no /occultism info reply"
    eval "$(python3 "$PORT/tools/parse_info.py" "$INFO_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the info line did not parse: $INFO_LINE"
    echo "  $INFO_LINE" >&2
}

# Hold ITEM in hotbar slot SLOT (1-9) and select it: `give` may fill a main
# inventory slot that `select ITEM` cannot reach.
hold() { # hold SLOT ITEM
    ctl command "item replace entity @s hotbar.$(( $1 - 1 )) with $2 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select "$1" >/dev/null
    ctl wait 2 >/dev/null
}

# Right-click the top face of the floor block at (X,100,Z) from three blocks
# south: within reach, and far enough that the player never picks the summoned
# item entity up (vanilla's pickup box is inflated by one block).
click_floor() { # click_floor X Z
    local x=$1 z=$2
    ctl command "execute in minecraft:the_end run tp @s $((x)).5 101 $((z + 3)).5" >/dev/null
    ctl wait 6 >/dev/null
    ctl look-at-block "$x" 100 "$z" up >/dev/null
    ctl wait 2 >/dev/null
    ctl use-item >/dev/null
    ctl wait 4 >/dev/null
}

# CONTROL_GAME: the same scenario's ids on a game **without** the package. Both
# must be unknown; if they resolve, the main run proves nothing about the package.
control_leg() {
    echo "== control: a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    ctl command "item replace entity @s hotbar.0 with occultism:datura 1" >/dev/null || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "occultism:datura")] | length == 0' >/dev/null \
        || fail "the control game has the datura item, so it carries this package"
    ctl command "execute in minecraft:the_end run setblock $X0 101 0 occultism:spirit_fire" >/dev/null || true
    ctl wait 4 >/dev/null
    local name
    name=$(block_name "$X0" 101 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    # The crop family must be unknown too, or the crop phases prove nothing.
    ctl command "clear @s" >/dev/null || true
    ctl command "item replace entity @s hotbar.0 with occultism:datura_seeds 1" >/dev/null || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "occultism:datura_seeds")] | length == 0' >/dev/null \
        || fail "the control game has the datura seeds, so it carries this package"
    ctl command "execute in minecraft:the_end run setblock $X0 101 0 occultism:datura[age=7]" >/dev/null || true
    ctl wait 4 >/dev/null
    name=$(block_name "$X0" 101 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the crop id was known"
    echo "  without the package: no datura item, no datura seeds, neither the fire nor the crop block"
    # The four loot-table blocks (0.2.4) must be unknown too.
    for id in occultism:otherstone occultism:otherrock occultism:otherflower \
              occultism:otherworld_sapling_natural; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
    done
    echo "  without the package: none of the four loot-table blocks either"
    # The six building blocks (0.2.6) must be unknown too.
    for id in occultism:otherstone_slab occultism:otherstone_stairs \
              occultism:otherstone_wall occultism:otherrock_slab \
              occultism:otherrock_stairs occultism:otherrock_wall; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
    done
    echo "  without the package: none of the six building blocks either"
    # The bricks tier (0.2.7) and the polished tier (0.2.8) must be unknown too.
    for id in occultism:otherstone_bricks occultism:chiseled_otherstone_bricks \
              occultism:cracked_otherrock_bricks occultism:polished_otherstone \
              occultism:polished_otherrock occultism:polished_otherstone_slab \
              occultism:polished_otherstone_stairs occultism:polished_otherstone_wall \
              occultism:polished_otherrock_slab occultism:polished_otherrock_stairs \
              occultism:polished_otherrock_wall; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
    done
    echo "  without the package: none of the twelve bricks-tier or eight polished-tier blocks either"
    # The cobblestone tier (0.2.9) must be unknown too.
    for id in occultism:othercobblestone occultism:othercobblestone_slab \
              occultism:othercobblestone_stairs occultism:othercobblestone_wall \
              occultism:othercobblerock occultism:othercobblerock_slab \
              occultism:othercobblerock_stairs occultism:othercobblerock_wall; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
    done
    echo "  without the package: none of the eight cobblestone-tier blocks either"
    # The redstone pair (0.2.10) must be unknown too.
    for id in occultism:otherstone_button occultism:otherrock_button \
              occultism:otherstone_pressure_plate occultism:otherrock_pressure_plate; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
    done
    echo "  without the package: none of the four redstone-pair blocks either"
    # The frames and pedestals (0.2.11) must be unknown too.
    for id in occultism:otherstone_pedestal occultism:otherstone_pedestal_silver; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
    done
    for id in occultism:otherstone_frame occultism:otherrock_frame; do
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: neither the two pedestal blocks nor the two frame items"
    ctl quit >/dev/null 2>&1 || true
}

# The redstone pair's own control probe: `CONTROL_LEG=1 REDSTONE_ONLY=1` launches
# the game **without** the package, asserts the four ids are unknown as blocks and
# as items, and exits 0. The standard control leg (`GAME=$CONTROL_GAME`) stops at
# phase A's datura assertion, so this is the leg that proves *this* family's ids
# are the package's and not the engine's.
control_redstone() {
    echo "== control: the four redstone ids on a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    local id name
    for id in occultism:otherstone_button occultism:otherrock_button \
              occultism:otherstone_pressure_plate occultism:otherrock_pressure_plate; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: none of the four redstone ids exist as a block or an item"
    ctl quit >/dev/null 2>&1 || true
}

# The four carried block loot tables (0.2.4). Each block is placed, its state read
# back, broken in survival and the `ctl break` reply's `.drops` asserted:
#   * otherstone and otherrock are `requires_tool: true` (stone template), so the
#     fail-capable pair is: a hand break breaks the block and drops NOTHING, a
#     wooden pickaxe break drops exactly one of them. Without the `requires_tool`
#     gate both would drop, and the hand assertion would fail. Since 0.2.9 the
#     pickaxe drops the **cobble** tier (the jar's alternatives table) and a
#     silk-touch pickaxe keeps the stone — the same pair for that table;
#   * otherflower and otherworld_sapling_natural are `requires_tool: false` and
#     drop exactly one of themselves to a bare hand.
# LOOT_NEGATIVE=1 flips the pickaxe assertion to "drops no othercobblestone", so
# the run must FAIL: the counterfactual for this family.
phase_loot() {
    echo "== O: the four carried block loot tables =="
    end "gamerule random_tick_speed 0"
    # The End has no sky light, so hostile mobs spawn on the arena floor; an
    # Enderman killed the player 4 s into the first run of this phase (measured,
    # 2026-11-21). Peaceful removes them and keeps them away without touching what
    # a break rolls.
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    local player
    player=$(ctl status | jq -r '.server.name // empty')
    [ -n "$player" ] || fail "cannot name the local player"
    ctl command "gamemode survival @s" >/dev/null
    for _ in $(seq 1 40); do
        [ "$(ctl status | jq -r '.server.game_mode // ""')" = 0 ] && break
        ctl wait 1 >/dev/null
    done
    [ "$(ctl status | jq -r '.server.game_mode // ""')" = 0 ] \
        || fail "the loot phase needs a survival player, game_mode is $(ctl status | jq -r '.server.game_mode // ""')"
    # Place BLOCK at (X,101,BLOCK_Z), optionally laying GROUND at (X,100,BLOCK_Z).
    place() { # place X BLOCK [GROUND]
        local x=$1 block=$2 ground=${3:-}
        [ -z "$ground" ] || end "setblock $x 100 $BLOCK_Z $ground"
        end "setblock $x 101 $BLOCK_Z $block"
        ctl wait 4 >/dev/null
    }
    # The total of a break reply's `.drops` (every entry, whatever its id). A
    # plain variable, not `${1:-{\}}` as a here-string: bash parses that default
    # inside a nested function as `$1` followed by a stray `}`, so jq saw
    # `<json>}` and exited 5 with "Unmatched '}'" (measured, 2026-11-21).
    drops_total() {
        local reply=${1:-}
        [ -n "$reply" ] || reply='{}'
        jq -r '[(.drops // {})[]] | add // 0' <<<"$reply"
    }

    # The stone pair: hand breaks nothing; a wooden pickaxe rolls the cobble
    # tier (the jar's `otherstone`/`otherrock` table, whose alternatives branch
    # is a non-silk-touch break -> othercobblestone/othercobblerock), and a
    # silk-touch pickaxe keeps the stone itself. The two are the fail-capable pair
    # for the table 0.2.4 shipped simplified and 0.2.9 carries from the jar.
    silk_hold() {
        ctl command "clear @s" >/dev/null
        ctl command "xp add @s 30 levels" >/dev/null
        hold 1 minecraft:diamond_pickaxe
        ctl command "enchant @s minecraft:silk_touch" >/dev/null
        ctl wait 4 >/dev/null
    }
    for spec in "occultism:otherstone occultism:othercobblestone 232" \
                "occultism:otherrock occultism:othercobblerock 234"; do
        read -r id cobble x <<<"$spec"
        place "$x" "$id"
        got=$(state_at "$x" 101 "$BLOCK_Z")
        case "$got" in
            "$id"*) ;;
            *) fail "$id did not place: '$got'" ;;
        esac
        echo "  placed $got at x=$x"
        # 1. A bare hand: the block breaks and the table must not roll.
        ctl command "clear @s" >/dev/null
        ctl wait 4 >/dev/null
        REPLY=$(break_at "$x" 101 "$BLOCK_Z") || fail "cannot hand-break $id"
        echo "  $id by hand: $(jq -c '{block_before,block,drops}' <<<"$REPLY")"
        [ "$(jq -r '.block' <<<"$REPLY")" = "minecraft:air" ] \
            || fail "the hand break left $(jq -r '.block' <<<"$REPLY") at x=$x"
        HAND=$(drops_of "$REPLY" "$id")
        HAND_TOTAL=$(drops_total "$REPLY")
        [ "$HAND" = 0 ] || fail "a hand break of $id dropped $HAND of it (requires_tool must gate the table)"
        [ "$HAND_TOTAL" = 0 ] || fail "a hand break of $id dropped $HAND_TOTAL items (requires_tool must gate the table)"
        # 2. A wooden pickaxe: exactly one cobble block, never the stone.
        hold 1 minecraft:wooden_pickaxe
        place "$x" "$id"
        got=$(state_at "$x" 101 "$BLOCK_Z")
        case "$got" in
            "$id"*) ;;
            *) fail "$id did not place again: '$got'" ;;
        esac
        REPLY=$(break_at "$x" 101 "$BLOCK_Z") || fail "cannot mine $id with a wooden pickaxe"
        echo "  $id with a wooden pickaxe: $(jq -c '{block_before,block,drops}' <<<"$REPLY")"
        [ "$(jq -r '.block' <<<"$REPLY")" = "minecraft:air" ] \
            || fail "the pickaxe break left $(jq -r '.block' <<<"$REPLY") at x=$x"
        COBBLE_DROPS=$(drops_of "$REPLY" "$cobble")
        STONE_DROPS=$(drops_of "$REPLY" "$id")
        TOOL_TOTAL=$(drops_total "$REPLY")
        if [ "$LOOT_NEGATIVE" = 1 ]; then
            echo "(negative: asserting the pickaxe break of $id drops no $cobble)"
            [ "$COBBLE_DROPS" = 0 ] || fail "negative: $id dropped $COBBLE_DROPS $cobble with a wooden pickaxe (this run must fail)"
        else
            [ "$COBBLE_DROPS" = 1 ] || fail "a wooden pickaxe dropped $COBBLE_DROPS $cobble from $id, expected exactly 1 ($(jq -c '.drops' <<<"$REPLY"))"
            [ "$TOOL_TOTAL" = 1 ] || fail "the pickaxe break dropped $TOOL_TOTAL items in total, expected exactly 1"
            [ "$STONE_DROPS" = 0 ] || fail "$id dropped itself without silk touch ($STONE_DROPS)"
        fi
        # 3. A silk-touch pickaxe: exactly the stone, never the cobble block.
        silk_hold
        place "$x" "$id"
        got=$(state_at "$x" 101 "$BLOCK_Z")
        case "$got" in
            "$id"*) ;;
            *) fail "$id did not place again (silk touch): '$got'" ;;
        esac
        REPLY=$(break_at "$x" 101 "$BLOCK_Z") || fail "cannot mine $id with a silk-touch pickaxe"
        echo "  $id with a silk-touch pickaxe: $(jq -c '{block_before,block,drops}' <<<"$REPLY")"
        [ "$(jq -r '.block' <<<"$REPLY")" = "minecraft:air" ] \
            || fail "the silk-touch break left $(jq -r '.block' <<<"$REPLY") at x=$x"
        SILK_STONE=$(drops_of "$REPLY" "$id")
        SILK_COBBLE=$(drops_of "$REPLY" "$cobble")
        SILK_TOTAL=$(drops_total "$REPLY")
        [ "$SILK_STONE" = 1 ] || fail "$id under silk touch dropped $SILK_STONE of itself, expected 1 ($(jq -c '.drops' <<<"$REPLY"))"
        [ "$SILK_TOTAL" = 1 ] || fail "the silk-touch break dropped $SILK_TOTAL items in total, expected 1"
        [ "$SILK_COBBLE" = 0 ] || fail "$id under silk touch also dropped $SILK_COBBLE $cobble"
    done

    # The two plants: a bare hand drops exactly one of itself.
    for spec in "occultism:otherflower 236" "occultism:otherworld_sapling_natural 238"; do
        read -r id x <<<"$spec"
        place "$x" "$id" minecraft:dirt
        got=$(state_at "$x" 101 "$BLOCK_Z")
        case "$got" in
            "$id"*) ;;
            *) fail "$id did not place on dirt: '$got'" ;;
        esac
        echo "  placed $got at x=$x (on dirt)"
        ctl command "clear @s" >/dev/null
        ctl wait 4 >/dev/null
        REPLY=$(break_at "$x" 101 "$BLOCK_Z") || fail "cannot hand-break $id"
        echo "  $id by hand: $(jq -c '{block_before,block,drops}' <<<"$REPLY")"
        [ "$(jq -r '.block' <<<"$REPLY")" = "minecraft:air" ] \
            || fail "the hand break left $(jq -r '.block' <<<"$REPLY") at x=$x"
        GOT=$(drops_of "$REPLY" "$id")
        GOT_TOTAL=$(drops_total "$REPLY")
        [ "$GOT" = 1 ] || fail "a hand break of $id dropped $GOT of it, expected exactly 1 ($(jq -c '.drops' <<<"$REPLY"))"
        [ "$GOT_TOTAL" = 1 ] || fail "a hand break of $id dropped $GOT_TOTAL items in total, expected exactly 1"
    done
    echo "  otherstone/otherrock: hand 0; a wooden pickaxe drops exactly one othercobblestone/othercobblerock and no stone; a silk-touch pickaxe keeps the stone and drops no cobble; otherflower and the natural sapling: hand 1 each"
}

# The two carried crafting-table recipes (0.2.5). The driver opens a real table,
# fills the 3x3 grid one item per cell, reads the result slot, takes it and reads
# the inventory back; the recipe files are the jar's own shapes translated to the
# 26.3 dialect (a bare id / `#tag`, and an alternative list of those strings).
# CRAFT_NEGATIVE=1 inverts the fruit assertion so the run must FAIL.
phase_craft() {
    echo "== P: the two carried crafting recipes in a real crafting table =="
    end "gamerule random_tick_speed 0"
    # Standing still over the dark End floor: peaceful keeps hostile spawns away.
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    CRAFT_NEGATIVE=$CRAFT_NEGATIVE python3 "$PORT/tools/craft_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the crafting recipe check failed (see the CRAFT FAIL line above)"
    echo "  the 3x3 fruit grid, the 3x3 seed grid and the mixed grid each made one occultism:demons_dream_essence;"
    echo "  nine dirt and eight fruit + one dirt made nothing; the shapeless book recipe made one occultism:book_of_binding_empty"
}

# The six `otherstone`/`otherrock` building blocks (0.2.6): the slab, stair and
# wall of each stone, in a real crafting table (three stones in the middle row
# make six slabs, the stair step makes four, two rows make six), then each block
# placed by `setblock`, its default state read back, a bare hand break (no drop:
# `requires_tool`) and a wooden pickaxe break (exactly one). Two walls side by
# side must connect, and a double slab drops the jar table's two.
# BUILDING_NEGATIVE=1 inverts the slab assertion so the run must FAIL.
phase_building() {
    echo "== Q: the six otherstone/otherrock building blocks =="
    end "gamerule random_tick_speed 0"
    # Standing still over the dark End floor: peaceful keeps hostile spawns away.
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    BUILDING_NEGATIVE=$BUILDING_NEGATIVE python3 "$PORT/tools/building_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the building block check failed (see the BUILDING FAIL line above)"
    echo "  the six building recipes craft their declared counts with every ingredient consumed;"
    echo "  each block reads its template's default state, drops nothing to a hand and exactly one to a wooden pickaxe;"
    echo "  two walls connect, and a double slab drops the jar table's two"
}

# The bricks tier (0.2.7): the ten bricks-tier grids in a real table (a 2x2 of
# the stone makes four bricks, the brick slab/stair/wall recipes work from those
# bricks, two brick slabs make the chiseled block), one opened table's chain
# (twelve stones -> bricks -> stairs -> slabs -> chiseled, each step consuming
# the previous output), the cracked pair in a real furnace with a no-fuel
# differential, then each of the twelve blocks placed, its default state read
# back, a bare hand break (no drop: `requires_tool`) and a wooden pickaxe break
# (exactly one). A brick wall pair must connect, and a double brick slab drops
# the jar table's two. BRICKS_NEGATIVE=1 inverts the bricks-grid assertion so the
# run must FAIL.
phase_bricks() {
    echo "== R: the twelve bricks-tier otherstone/otherrock blocks =="
    end "gamerule random_tick_speed 0"
    # Standing still over the dark End floor: peaceful keeps hostile spawns away.
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    BRICKS_NEGATIVE=$BRICKS_NEGATIVE python3 "$PORT/tools/bricks_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the bricks check failed (see the BRICKS FAIL line above)"
    echo "  the ten bricks-tier recipes craft their declared counts with every ingredient consumed;"
    echo "  four dirt in the same 2x2 and three stones one cell short made nothing;"
    echo "  the chain stones -> bricks -> stairs -> slabs -> chiseled consumed each previous output in one table;"
    echo "  the cracked pair smelted in a real furnace while the fuel-less one stayed idle;"
    echo "  each block reads its template's default state, drops nothing to a hand and exactly one to a wooden pickaxe;"
    echo "  a brick wall pair connects, and a double brick slab drops two"
}

# The polished tier (0.2.8): the six polished-tier grids in a real table (the
# polished slab/stair/wall recipes, a 2x2 of polished blocks as the differential),
# the plain pair in a real furnace with a no-fuel differential,
# then each of the eight blocks placed, its default state read back, a bare hand
# break (no drop: `requires_tool`) and a wooden pickaxe break (exactly one). A
# polished wall pair must connect, a double slab drops the jar table's two, and
# polished_otherrock_slab drops one although the jar ships no table for it.
# POLISHED_NEGATIVE=1 inverts the polished slab grid assertion so the run must FAIL.
phase_polished() {
    echo "== S: the eight polished-tier otherstone/otherrock blocks =="
    end "gamerule random_tick_speed 0"
    # Standing still over the dark End floor: peaceful keeps hostile spawns away.
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    POLISHED_NEGATIVE=$POLISHED_NEGATIVE python3 "$PORT/tools/polished_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the polished block check failed (see the POLISHED FAIL line above)"
    echo "  the six polished-tier recipes craft their declared counts with every ingredient consumed;"
    echo "  four polished blocks in the 2x2 and four dirt in the same cells made nothing;"
    echo "  the polished pair smelted in a real furnace while the fuel-less one stayed idle;"
    echo "  each block reads its template's default state, drops nothing to a hand and exactly one to a wooden pickaxe;"
    echo "  a polished wall pair connects, a double polished slab drops two, and polished_otherrock_slab drops one"
}

# The cobblestone tier (0.2.9): the six cobble-tier grids in a real table (the
# cobble slab/stair/wall recipes, a 2x2 of the cobble as the differential), then
# each of the eight blocks placed, its default state read back, a bare hand break
# (no drop: `requires_tool`) and a wooden pickaxe break (exactly one). A cobble
# wall pair must connect, a double cobble slab drops the jar table's two, and
# othercobblerock_slab drops one although the jar ships no table for it. The plain
# pair's obtainability: the raw stones drop cobble without silk touch and
# themselves with it. COBBLE_NEGATIVE=1 inverts the cobble slab grid assertion so
# the run must FAIL.
phase_cobble() {
    echo "== T: the eight othercobblestone/othercobblerock blocks =="
    end "gamerule random_tick_speed 0"
    # Standing still over the dark End floor: peaceful keeps hostile spawns away.
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    COBBLE_NEGATIVE=$COBBLE_NEGATIVE python3 "$PORT/tools/cobble_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the cobblestone block check failed (see the COBBLE FAIL line above)"
    echo "  the six cobble-tier recipes craft their declared counts with every ingredient consumed;"
    echo "  four cobble blocks in the 2x2 and four dirt in the same cells made nothing;"
    echo "  each block reads its stone template's default state, drops nothing to a hand and exactly one to a wooden pickaxe;"
    echo "  a cobble wall pair connects, a double cobble slab drops two, and othercobblerock_slab drops one;"
    echo "  the raw stones drop one cobble block without silk touch and keep themselves with it"
}

# The redstone pair (0.2.10): the four grids (one stone -> one button, two
# stones in one row -> one plate) with the wrong-ingredient differentials, the
# jar's reverse smelt pair in a real furnace, each of the four blocks placed,
# read back, broken by a hand and by a wooden pickaxe (one each — these
# templates are `requires_tool: false`, with otherstone's hand break dropping
# nothing as the differential), a plate pressed by the player and released, and a
# button clicked (powered, held, released). PLATE_SELFTEST=1 and
# BUTTON_SELFTEST=1 each invert their own leg so those runs must FAIL.
phase_redstone() {
    echo "== U: the four otherstone/otherrock redstone blocks =="
    end "gamerule random_tick_speed 0"
    # Standing still over the dark End floor: peaceful keeps hostile spawns away
    # (a mob on the plate would keep it pressed after the player steps off).
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    PLATE_SELFTEST=$PLATE_SELFTEST BUTTON_SELFTEST=$BUTTON_SELFTEST \
        python3 "$PORT/tools/redstone_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the redstone block check failed (see the REDSTONE FAIL line above)"
    echo "  the four redstone grids craft their declared counts with every ingredient consumed;"
    echo "  a cobble button grid and a cobble plate grid made nothing, and one stone in the plate grid made the button, not the plate;"
    echo "  the cobble block smelted back into the stone in a real furnace while a fuel-less one stayed idle;"
    echo "  each block reads its template's default state, drops exactly one to a hand and one to a wooden pickaxe;"
    echo "  otherstone's bare-hand break dropped nothing (the differential);"
    echo "  each plate reads powered=true under the player and false after stepping off;"
    echo "  each button reads powered=true after a click, still true ten ticks later and false after its press duration"
}

# The frames and pedestals (0.2.11): the four grids (a stone ring -> one frame,
# five slabs + one stone -> one pedestal) with the wrong-shape differentials, the
# frames as **items** (given, and a `setblock` of the id refused), each pedestal
# placed, read back, broken by a hand (nothing — the `minecraft:stone` template is
# `requires_tool: true`) and by a wooden pickaxe (exactly one), with otherstone's
# hand break (nothing) as the differential. PEDESTAL_NEGATIVE=1 flips the frame
# grid and PEDESTAL_SELFTEST=1 flips the item leg; both runs must FAIL.
phase_pedestal() {
    echo "== V: the frames and pedestals =="
    end "gamerule random_tick_speed 0"
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    PEDESTAL_NEGATIVE=$PEDESTAL_NEGATIVE PEDESTAL_SELFTEST=$PEDESTAL_SELFTEST \
        python3 "$PORT/tools/pedestal_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the frames-and-pedestals check failed (see the PEDESTAL FAIL line above)"
    echo "  the four frame/pedestal grids craft their declared counts with every ingredient consumed;"
    echo "  a ninth stone in the frame's hole and four slabs + one stone made nothing;"
    echo "  both frames are items: given into the inventory and refused by /setblock (air stays);"
    echo "  each pedestal reads its own state, drops nothing to a bare hand and exactly one to a wooden pickaxe;"
    echo "  otherstone's bare-hand break dropped nothing (the differential)"
}

# The sacrificial bowls (0.2.12): the six grids, the six blocks placed and read
# back, the storage round trip on a plain and a golden bowl (a damaged sword in,
# read back through the block entity, returned with its component), the jar's slot
# limit, the spill on break, the conversion over spirit fire with the golden
# differential, the sneaking no-op, and the tool pair (hand 1 / pickaxe 1, with
# otherstone's hand break at 0 as the differential). BOWL_NEGATIVE=1 flips the
# first grid and BOWL_SELFTEST=1 flips the take-out leg; both runs must FAIL.
phase_bowl() {
    echo "== W: the six sacrificial bowls =="
    end "gamerule random_tick_speed 0"
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    BOWL_NEGATIVE=$BOWL_NEGATIVE BOWL_SELFTEST=$BOWL_SELFTEST \
        python3 "$PORT/tools/bowl_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the sacrificial-bowls check failed (see the BOWL FAIL line above)"
    echo "  the six bowl grids craft their declared counts with every ingredient consumed;"
    echo "  four stones and seven gold + one bowl made nothing;"
    echo "  each of the six bowls places and reads back;"
    echo "  a damaged diamond sword goes into a plain and a golden bowl, is read back through the block entity and comes back to an empty hand with its component;"
    echo "  five dirt put one in the bowl and left four in the hand;"
    echo "  breaking a full bowl dropped the bowl and spilled its stack;"
    echo "  a feather in a plain bowl over spirit fire became an awakened feather and stayed a feather in a golden one;"
    echo "  a sneaking click changed nothing;"
    echo "  every bowl dropped exactly one to a bare hand and one to a wooden pickaxe, and otherstone's bare-hand break dropped nothing (the differential)"
}

# The impure chalks (0.2.13): the 17 grids (15 shapeless colour grids + the two
# mirrored shaped white grids over the nested chalk-base tag), each filled exactly
# as its recipe declares, the result read back, taken, the grid emptied and every
# ingredient consumed; the wrong-ingredient pink grid and the chalk-base-only
# shaped grid make nothing; then the game log must carry no `invalid ingredient`
# (a tag-keyed cell that did not resolve). CHALK_SELFTEST=1 flips the first grid.
phase_chalk() {
    echo "== X: the 17 impure-chalk craftings =="
    end "gamerule random_tick_speed 0"
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    CHALK_SELFTEST=$CHALK_SELFTEST \
        python3 "$PORT/tools/chalk_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the impure-chalk check failed (see the CHALK FAIL line above)"
    local logs
    logs=$(ls "$SCRATCH"/launch-*.log 2>/dev/null || true)
    [ -n "$logs" ] || fail "no game log to grep for invalid ingredients"
    if grep -qi 'invalid ingredient' $logs; then
        grep -in 'invalid ingredient' $logs >&2
        fail "a tag-keyed ingredient did not resolve (invalid ingredient in the game log)"
    fi
    echo "  the game log has no 'invalid ingredient' line (every tag-keyed cell resolved)"
    echo "  the 17 grids craft their declared chalk with every ingredient consumed and the grid emptied;"
    echo "  the nested chalk base was filled with both burnt stones, the other tags with their own members;"
    echo "  a pink grid with dirt for one demonic meat and a two-column chalk-base grid with no otherworld wood made nothing;"
    echo "  the polished pair smelted into burnt otherstone/otherrock in a real furnace (200 ticks, 0.15 xp) while a fuel-less one stayed idle"
}

# The impure chalks' own control leg: the game without the package, the 17 chalk
# ids and the ingredient ids.
control_chalks() {
    echo "== control: the impure-chalk ids on a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    local id name
    for name in chalk_white_impure chalk_light_gray_impure chalk_gray_impure \
                chalk_black_impure chalk_brown_impure chalk_red_impure \
                chalk_orange_impure chalk_yellow_impure chalk_lime_impure \
                chalk_green_impure chalk_cyan_impure chalk_light_blue_impure \
                chalk_blue_impure chalk_purple_impure chalk_magenta_impure \
                chalk_pink_impure burnt_otherstone burnt_otherrock demonic_meat \
                cruelty_essence gray_paste; do
        id=occultism:$name
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: none of the 21 impure-chalk and ingredient ids exist as an item"
    ctl quit >/dev/null 2>&1 || true
}

# The otherstone tablet (0.2.14): the two shaped grids, each filled with four
# ashes, four gold and one of the two raw stones, the result read from slot 200
# of the real table screen, taken, and every ingredient consumed with the grid
# emptied; a three-ashes grid makes nothing; any `invalid ingredient` line in
# the game log fails the phase. TABLET_SELFTEST=1 flips the first grid's count.
phase_tablet() {
    echo "== Y: the two otherstone_tablet craftings =="
    end "gamerule random_tick_speed 0"
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    end "setblock $CRAFT_X 100 $CRAFT_Z minecraft:stone"
    ctl wait 4 >/dev/null
    TABLET_SELFTEST=$TABLET_SELFTEST \
        python3 "$PORT/tools/tablet_recipes.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the tablet check failed (see the TABLET FAIL line above)"
    local logs
    logs=$(ls "$SCRATCH"/launch-*.log 2>/dev/null || true)
    [ -n "$logs" ] || fail "no game log to grep for invalid ingredients"
    if grep -qi 'invalid ingredient' $logs; then
        grep -in 'invalid ingredient' $logs >&2
        fail "a tag-keyed ingredient did not resolve (invalid ingredient in the game log)"
    fi
    echo "  the game log has no 'invalid ingredient' line (both tag-keyed cells resolved)"
    echo "  both grids made one otherstone_tablet with every ingredient consumed and the grid emptied;"
    echo "  a three-ashes grid made nothing"
}

# The stonecutting family (0.2.16): a real stonecutter at (CRAFT_X, 101, CRAFT_Z),
# all 32 carried recipes enumerated from the ported data files, each declared
# input fed into slot 600, its jar recipe selected by a GUI cell click, the result
# read from slot 650 and taken (id and count asserted, input consumed).  A
# different selected cell yields a different result and vanilla stone offers
# nothing from this package. STONECUT_SELFTEST=stonecut-all flips the bricks cut's
# count from one to two (the run must FAIL).
phase_stonecut() {
    echo "== Z: the 32 carried stonecutting recipes at a real stonecutter =="
    end "gamerule random_tick_speed 0"
    end "difficulty peaceful"
    end "kill @e[type=!player]"
    ctl wait 20 >/dev/null
    STONECUT_SELFTEST=$STONECUT_SELFTEST \
        python3 "$PORT/tools/stonecutter_check.py" \
        "$SOCK" "$CTL" "$CRAFT_X" 101 "$CRAFT_Z" \
        || fail "the stonecutter check failed (see the STONECUT FAIL line above)"
    local logs
    logs=$(ls "$SCRATCH"/launch-*.log 2>/dev/null || true)
    [ -n "$logs" ] || fail "no game log to grep for invalid ingredients"
    if grep -qi 'invalid ingredient' $logs; then
        grep -in 'invalid ingredient' $logs >&2
        fail "a stonecutting ingredient did not resolve (invalid ingredient in the game log)"
    fi
    echo "  the game log has no 'invalid ingredient' line (every carried input resolved)"
}

# The stonecutting family's control: the ids a game without the package has not
# got. The real differential (vanilla stone makes no occultism slab) is in the
# phase itself.
control_stonecut() {
    echo "== control: the stonecutting ids on a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    local id
    for id in occultism:otherstone_slab occultism:otherrock_slab \
              occultism:otherstone_slab_from_otherstone; do
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: the slab ids and the recipe id are unknown as items"
    ctl quit >/dev/null 2>&1 || true
}

if [ "$CONTROL_LEG" = 1 ] && [ "$STONECUT_ONLY" = 1 ] && [ -n "$CONTROL_GAME" ]; then
    control_stonecut
    echo "PASS: a game without the package refuses the stonecutting ids as items"
    exit 0
fi

# The tablet's own control leg: the game without the package cannot fill a grid
# with an ingredient it does not know.
control_tablet() {
    echo "== control: the tablet ids on a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    local id
    for id in occultism:otherstone_tablet occultism:otherworld_ashes; do
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: the tablet and the ashes are unknown as items"
    ctl quit >/dev/null 2>&1 || true
}

if [ "$CONTROL_LEG" = 1 ] && [ "$TABLET_ONLY" = 1 ] && [ -n "$CONTROL_GAME" ]; then
    control_tablet
    echo "PASS: a game without the package refuses the tablet and the ashes as items"
    exit 0
fi

# The bowls' own control leg: the game without the package, six ids.
control_bowls() {
    echo "== control: the six bowl ids on a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    local id name
    for id in occultism:sacrificial_bowl occultism:dark_sacrificial_bowl \
              occultism:copper_sacrificial_bowl occultism:dark_copper_sacrificial_bowl \
              occultism:golden_sacrificial_bowl occultism:dark_golden_sacrificial_bowl; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: none of the six bowl ids exist as a block or an item"
    ctl quit >/dev/null 2>&1 || true
}

if [ "$CONTROL_LEG" = 1 ] && [ "$CHALK_ONLY" = 1 ] && [ -n "$CONTROL_GAME" ]; then
    control_chalks
    echo "PASS: a game without the package refuses all 16 impure-chalk ids and the five sampled ingredient ids as items"
    exit 0
fi

# The bowls' own control leg: the game without the package, six ids.
if [ "$CONTROL_LEG" = 1 ] && [ "$BOWL_ONLY" = 1 ] && [ -n "$CONTROL_GAME" ]; then
    control_bowls
    echo "PASS: a game without the package refuses all six bowl ids as a block and as an item"
    exit 0
fi

# The frames-and-pedestals control leg: the game without the package, four ids.
control_frames() {
    echo "== control: the four frame and pedestal ids on a game without the package =="
    launch "$CONTROL_GAME" "$CONTROL_WORLD"
    arena
    local id name
    for id in occultism:otherstone_pedestal occultism:otherstone_pedestal_silver \
              occultism:otherstone_frame occultism:otherrock_frame; do
        ctl command "execute in minecraft:the_end run setblock $X0 101 0 $id" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        name=$(block_name "$X0" 101 0)
        [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so $id was known"
        ctl command "clear @s" >/dev/null || true
        ctl command "item replace entity @s hotbar.0 with $id 1" >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl inventory | jq -e --arg i "$id" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
            || fail "the control game has the $id item, so it carries this package"
    done
    echo "  without the package: none of the four frame/pedestal ids exist as a block or an item"
    ctl quit >/dev/null 2>&1 || true
}

# The frames/pedestals' own control leg: the game without the package, four ids.
if [ "$CONTROL_LEG" = 1 ] && [ "$PEDESTAL_ONLY" = 1 ] && [ -n "$CONTROL_GAME" ]; then
    control_frames
    echo "PASS: a game without the package refuses all four frame/pedestal ids as a block and as an item"
    exit 0
fi

# The redstone pair's own control leg: the game without the package, four ids.
if [ "$CONTROL_LEG" = 1 ] && [ "$REDSTONE_ONLY" = 1 ] && [ -n "$CONTROL_GAME" ]; then
    control_redstone
    echo "PASS: a game without the package refuses all four redstone ids as a block and as an item"
    exit 0
fi

echo "== launch (The End arena, launcher-built game with the package) =="
if [ "$CONTROL_LEG" = 1 ] && [ -n "$CONTROL_GAME" ]; then control_leg; fi
launch "$GAME" "$WORLD"
arena

# The red/green leg for the loot family: launch, arena, phase O only.
if [ "$LOOT_ONLY" = 1 ]; then
    phase_loot
    echo "PASS: the four carried block loot tables: otherstone and otherrock drop nothing to a bare hand, exactly one othercobblestone/othercobblerock to a wooden pickaxe and themselves to a silk-touch pickaxe, otherflower and otherworld_sapling_natural drop exactly one of themselves to a bare hand, each after being placed and read back in survival"
    exit 0
fi

# The red/green leg for the crafting family: launch, arena, phase P only.
if [ "$CRAFT_ONLY" = 1 ]; then
    phase_craft
    echo "PASS: the two carried crafting recipes craft at a real crafting table: a 3x3 grid of the datura fruit, of the datura seed and a mixed fruit/seed grid each make exactly one occultism:demons_dream_essence with every ingredient consumed, nine dirt and eight fruit with one dirt make nothing, and the shapeless book recipe makes one occultism:book_of_binding_empty with its three items consumed"
    exit 0
fi

# The red/green leg for the building family: launch, arena, phase Q only.
if [ "$BUILDING_ONLY" = 1 ]; then
    phase_building
    echo "PASS: the six otherstone/otherrock building recipes craft at a real table (three stones in the middle row make six slabs, the stair step four stairs, two rows six walls, every ingredient consumed; three dirt make nothing), each block reads its stone template's default state, drops nothing to a bare hand and exactly one of itself to a wooden pickaxe, two walls connect, and a double slab drops two"
    exit 0
fi

# The red/green leg for the bricks tier: launch, arena, phase R only.
if [ "$BRICKS_ONLY" = 1 ]; then
    phase_bricks
    echo "PASS: the ten bricks-tier otherstone/otherrock recipes craft at a real table with every ingredient consumed (a 2x2 of the stone makes four bricks, the brick slab/stair/wall recipes work from those bricks, two brick slabs make one chiseled block; four dirt in the same 2x2 and three stones one cell short make nothing), a chain in one table turns twelve stones into bricks, six bricks into four stairs, three bricks into six slabs and two slabs into one chiseled block, the cracked pair smelts in a real furnace while a fuel-less furnace stays idle, each of the twelve blocks reads its stone template's default state, drops nothing to a bare hand and exactly one of itself to a wooden pickaxe, a brick wall pair connects, and a double brick slab drops two"
    exit 0
fi

# The red/green leg for the polished tier: launch, arena, phase S only.
if [ "$POLISHED_ONLY" = 1 ]; then
    phase_polished
    echo "PASS: the six polished-tier otherstone/otherrock recipes craft at a real table with every ingredient consumed (three polished blocks in the middle row make six slabs, the stair step four stairs, two rows six walls; four polished blocks in the 2x2 and four dirt make nothing), the polished pair smelts in a real furnace (one raw stone + one coal makes exactly one polished block and consumes the input, while a fuel-less furnace stays idle), each of the eight blocks reads its stone template's default state, drops nothing to a bare hand and exactly one of itself to a wooden pickaxe, a polished wall pair connects, a double polished slab drops two, and polished_otherrock_slab — the table the jar omits — drops one"
    exit 0
fi

# The red/green leg for the cobblestone tier: launch, arena, phase T only.
if [ "$COBBLE_ONLY" = 1 ]; then
    phase_cobble
    echo "PASS: the six cobble-tier othercobblestone/othercobblerock recipes craft at a real table with every ingredient consumed (three cobble blocks in the middle row make six slabs, the stair step four stairs, two rows six walls; four cobble blocks in the 2x2 and four dirt make nothing), each of the eight blocks reads its stone template's default state, drops nothing to a bare hand and exactly one of itself to a wooden pickaxe, a cobble wall pair connects, a double cobble slab drops two, othercobblerock_slab — the table the jar omits — drops one, and the raw stones drop one othercobblestone/othercobblerock with a wooden pickaxe and keep themselves under silk touch"
    exit 0
fi

# The red/green leg for the redstone pair: launch, arena, phase U only.
if [ "$REDSTONE_ONLY" = 1 ]; then
    phase_redstone
    echo "PASS: the four otherstone/otherrock redstone blocks work end to end: one stone in a real table makes one button and two stones in one row make one pressure plate (each of the four grids with every ingredient consumed, while a cobble button grid and a cobble plate grid make nothing and one stone in the plate grid makes the button rather than the plate), the cobble block smelts back into the stone in a real furnace at 200 ticks and 0.5 experience while a fuel-less furnace stays idle, each of the four blocks reads its template's default state and drops exactly one of itself to a bare hand and exactly one to a wooden pickaxe (with otherstone's hand break dropping nothing as the differential), each pressure plate reads powered=true under the player and powered=false after stepping off, and each button reads powered=true after a right click, is still pressed ten ticks later and reads powered=false after its press duration"
    exit 0
fi

# The red/green leg for the frames and pedestals: launch, arena, phase V only.
if [ "$PEDESTAL_ONLY" = 1 ]; then
    phase_pedestal
    echo "PASS: the frames and pedestals work end to end: eight stones in a ring make one otherstone or otherrock frame (an item — given into the inventory, and refused by /setblock, which leaves air), five slabs + one stone make one otherstone_pedestal or the jar's otherstone_pedestal_silver (whose file is otherrock_pedestal.json), every ingredient is consumed, a ninth stone in the frame's hole and one slab short of the pedestal make nothing, and each pedestal reads its state, drops nothing to a bare hand and exactly one of itself to a wooden pickaxe (with otherstone's hand break dropping nothing as the differential)"
    exit 0
fi

# The red/green leg for the sacrificial bowls: launch, arena, phase W only.
if [ "$BOWL_ONLY" = 1 ]; then
    phase_bowl
    echo "PASS: the sacrificial bowls work end to end: five stones in a ring make one sacrificial or dark bowl, eight gold ingots around that bowl make the golden pair and a bowl + a copper ingot makes the copper pair (every ingredient consumed; four stones and seven gold make nothing), each of the six places and reads back, a damaged diamond sword goes into a plain and a golden bowl and comes back to an empty hand with its component after being read through the block entity, five dirt put exactly one in the bowl and leave four in the hand, breaking a full bowl drops the bowl and spills its stack, a feather in a plain bowl over spirit fire becomes an awakened feather while a golden bowl leaves it a feather, a sneaking click changes nothing, and every bowl drops one to a bare hand and one to a wooden pickaxe with otherstone's hand break at nothing"
    exit 0
fi

# The red/green leg for the impure chalks: launch, arena, phase X only.
if [ "$CHALK_ONLY" = 1 ]; then
    phase_chalk
    echo "PASS: the 17 impure-chalk recipes craft at a real table: each grid makes one impure chalk with every ingredient consumed and the grid emptied, the nested chalk base resolves through both burnt stones and the other tag-keyed cells through their own members (no 'invalid ingredient' in the game log), a pink grid with dirt for one demonic meat and a two-column chalk-base grid with no otherworld wood make nothing, and the polished pair smelts into burnt otherstone/otherrock in a real furnace (200 ticks, 0.15 experience) while a fuel-less one stays idle"
    exit 0
fi

# The red/green leg for the otherstone tablet: launch, arena, phase Y only.
if [ "$TABLET_ONLY" = 1 ]; then
    phase_tablet
    echo "PASS: the two otherstone_tablet recipes craft at a real table: the aga/gsg/aga grids of four otherworld ashes, four gold and one otherstone or otherrock each make exactly one occultism:otherstone_tablet, taken from slot 200 of the table screen with every ingredient consumed and the grid emptied; three ashes make nothing and no tag-keyed cell left an 'invalid ingredient' line in the game log"
    exit 0
fi

# The red/green leg for the stonecutting family: launch, arena, phase Z only.
if [ "$STONECUT_ONLY" = 1 ]; then
    phase_stonecut
    echo "PASS: all 32 carried stonecutting recipes cut at a real stonecutter: each ingredient (otherstone, otherrock, their bricks and polished forms, and the othercobblestone/othercobblerock pair) fed into slot 600, its occultism:stonecutting/<stem> offer selected by a GUI cell click, and slot 650's output id and count asserted with the input consumed; a different selected cell yields a different result and vanilla stone offers no occultism recipe; no ingredient left an 'invalid ingredient' line in the game log"
    exit 0
fi

echo "== A: the carried ids register (datura item and spirit_fire block) =="
ctl command "item replace entity @s hotbar.0 with occultism:datura 1" >/dev/null
ctl wait 4 >/dev/null
ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "occultism:datura")] | length >= 1' >/dev/null \
    || fail "the datura item did not register: $(ctl inventory | jq -c '{hotbar,main}')"
echo "  occultism:datura is in the inventory"

echo "== B: the recipe table loads from the carried data =="
say "occultism recipes"
ctl wait 12 >/dev/null
RECIPES=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Spirit fire recipes' || true)
[ -n "$RECIPES" ] || fail "no /occultism recipes reply"
echo "  $RECIPES"
RECIPE_RULES=$(sed -nE 's/.*: ([0-9]+) rules.*/\1/p' <<<"$RECIPES")
RECIPE_FILES=$(sed -nE 's/.* ([0-9]+) files.*/\1/p' <<<"$RECIPES")
RECIPE_SKIPPED=$(sed -nE 's/.* ([0-9]+) skipped.*/\1/p' <<<"$RECIPES")
[ "$RECIPE_FILES" = 27 ] || fail "expected 27 spirit-fire recipe files, read $RECIPE_FILES"
[ "$RECIPE_RULES" = 26 ] || fail "expected 26 loadable rules, read $RECIPE_RULES"
[ "$RECIPE_SKIPPED" = 1 ] || fail "expected 1 skipped recipe (otherworld_ashes), read $RECIPE_SKIPPED"

echo "== C: flint and steel on datura lights spirit fire =="
hold 1 minecraft:flint_and_steel
# Stand back before summoning: an item entity at the player's feet is picked up
# on its next tick (the ignition box reaches two blocks from the click, so the
# click can still see it).
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_CONVERT + 3)).5" >/dev/null
ctl wait 6 >/dev/null
summon_item occultism:datura 1 "$X0.5" 101.0 "$Z_CONVERT.5"
[ "$(count_items occultism:datura 6)" = 1 ] || fail "the datura did not summon"
click_floor "$X0" "$Z_CONVERT"
NAME=$(block_name "$X0" 101 "$Z_CONVERT")
[ "$NAME" = "occultism:spirit_fire" ] || fail "expected spirit fire at the clicked face, got '$NAME'"
COLOUR=$(block_colour "$X0" 101 "$Z_CONVERT")
[ "$COLOUR" = "white" ] || fail "expected a white fire, got colour '$COLOUR'"
[ "$(count_items occultism:datura 6)" = 0 ] || fail "the datura was not consumed by the ignition"
echo "  $NAME at the clicked face, colour=$COLOUR, the datura is gone"

echo "== D: a stack of feathers becomes the same count of awakened feathers =="
# The conversion runs off the item entity's own tick, so the stack is already
# converted by the time the summon command returns: assert the outcome, not the
# intermediate. The summon line in the log is the "three arrived" evidence.
summon_item minecraft:feather 3 "$X0.5" 101.0 "$Z_CONVERT.5"
ctl wait 6 >/dev/null
[ "$(count_items minecraft:feather 6)" = 0 ] || fail "the feather stack was not converted"
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting 2 awakened feathers instead of 3)"
    EXPECT=2
else
    EXPECT=3
fi
GOT=$(count_items occultism:awakened_feather 6)
echo "  awakened feathers: $GOT (expected $EXPECT)"
[ "$GOT" = "$EXPECT" ] || fail "expected $EXPECT awakened feathers, got $GOT (the count rule is upstream's assemble)"
info "$X0" 101 "$Z_CONVERT"
[ "$BLOCK" = "occultism:spirit_fire" ] || fail "the fire is gone: $BLOCK"
[ "$LAST" = "occultism:awakened_feather x3" ] || fail "last conversion reads '$LAST'"
[ "$RULES" = 26 ] || fail "the readout says $RULES rules"

echo "== E: control — a feather on plain stone stays a feather =="
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_CONTROL + 3)).5" >/dev/null
ctl wait 6 >/dev/null
summon_item minecraft:feather 1 "$X0.5" 101.0 "$Z_CONTROL.5"
ctl wait 20 >/dev/null
[ "$(count_items minecraft:feather 6)" = 1 ] || fail "the control feather left the ground without a fire"
# The converted stack is 7 blocks behind the player: count the whole arena.
[ "$(count_items occultism:awakened_feather 14)" = "$EXPECT" ] \
    || fail "the control feather converted without a fire"
[ "$(block_name "$X0" 101 "$Z_CONTROL")" = "minecraft:air" ] || fail "the control spot is not air"
echo "  the feather on stone is still a feather, no awakened feather appeared"

echo "== F: an item no recipe accepts stays as it is, in the fire =="
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_CONVERT + 3)).5" >/dev/null
ctl wait 6 >/dev/null
summon_item minecraft:dirt 1 "$X0.5" 101.0 "$Z_CONVERT.5"
ctl wait 20 >/dev/null
[ "$(count_items minecraft:dirt 6)" = 1 ] || fail "the dirt was consumed"
info "$X0" 101 "$Z_CONVERT"
[ "$LAST" = "occultism:awakened_feather x3" ] || fail "a dirt conversion was recorded: $LAST"
echo "  the dirt is still dirt and the last conversion is still $LAST"

echo "== G: a gold chalk paints the fire yellow =="
hold 1 minecraft:flint_and_steel
summon_item occultism:datura 1 "$X0.5" 101.0 "$Z_PAINT.5"
click_floor "$X0" "$Z_PAINT"
[ "$(block_name "$X0" 101 "$Z_PAINT")" = "occultism:spirit_fire" ] \
    || fail "the second fire did not light at z=$Z_PAINT"
hold 2 occultism:chalk_gold
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_PAINT + 3)).5" >/dev/null
ctl wait 6 >/dev/null
ctl look-at-block "$X0" 101 "$Z_PAINT" up >/dev/null
ctl wait 2 >/dev/null
ctl use-item >/dev/null
ctl wait 6 >/dev/null
COLOUR=$(block_colour "$X0" 101 "$Z_PAINT")
[ "$COLOUR" = "yellow" ] || fail "chalk_gold left the fire colour '$COLOUR', expected yellow"
info "$X0" 101 "$Z_PAINT"
[ "$COLOUR" = "yellow" ] || fail "the readout says colour '$COLOUR'"
echo "  the fire at z=$Z_PAINT reads colour=$COLOUR (chalk_gold is the yellow chalk)"

echo "== H: datura two blocks away is neither consumed nor lit =="
hold 1 minecraft:flint_and_steel
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_FAR + 3)).5" >/dev/null
ctl wait 6 >/dev/null
summon_item occultism:datura 1 "$((X0 + 2)).5" 101.0 "$Z_FAR.5"
[ "$(count_items occultism:datura 8)" = 1 ] || fail "the far datura did not summon"
click_floor "$X0" "$Z_FAR"
FAR=$(block_name "$X0" 101 "$Z_FAR")
[ "$FAR" != "occultism:spirit_fire" ] || fail "spirit fire lit from datura two blocks away"
[ "$(count_items occultism:datura 8)" = 1 ] || fail "the far datura was consumed"
echo "  no spirit fire at z=$Z_FAR (the block is '$FAR'), the datura is untouched"

echo "== I: a spectator's click changes nothing and consumes nothing =="
# `ctl state` is the client's view (it has no player name); `ctl status` reports
# the server's player name and numeric GameType (0 survival, 1 creative, 2
# adventure, 3 spectator).
PLAYER=$(ctl status | jq -r '.server.name // empty')
[ -n "$PLAYER" ] || fail "cannot name the local player"
mode_now() { ctl status | jq -r '.server.game_mode // ""'; }
# A fresh white fire for the chalk probe.
hold 1 minecraft:flint_and_steel
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_SPEC + 3)).5" >/dev/null
ctl wait 6 >/dev/null
summon_item occultism:datura 1 "$X0.5" 101.0 "$Z_SPEC.5"
click_floor "$X0" "$Z_SPEC"
[ "$(block_name "$X0" 101 "$Z_SPEC")" = "occultism:spirit_fire" ] \
    || fail "the spectator-probe fire did not light at z=$Z_SPEC"
[ "$(block_colour "$X0" 101 "$Z_SPEC")" = "white" ] \
    || fail "the spectator-probe fire is not white to begin with"
ctl command "gamemode spectator $PLAYER" >/dev/null
ctl wait 10 >/dev/null
[ "$(mode_now)" = 3 ] || fail "the player is not a spectator: game_mode=$(mode_now)"
echo "  the player is a spectator (game_mode=$(mode_now))"
# 1. A spectator's chalk must not recolour the fire.
hold 2 occultism:chalk_red
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_SPEC + 3)).5" >/dev/null
ctl wait 6 >/dev/null
ctl look-at-block "$X0" 101 "$Z_SPEC" up >/dev/null
ctl wait 2 >/dev/null
ctl use-item >/dev/null || true
ctl wait 6 >/dev/null
SPEC_COLOUR=$(block_colour "$X0" 101 "$Z_SPEC")
[ "$SPEC_COLOUR" = "white" ] \
    || fail "a spectator recoloured the fire to '$SPEC_COLOUR' (the click must change nothing)"
echo "  a spectator's red chalk left the fire $SPEC_COLOUR"
# 2. A spectator's lighter must not light a fire nor consume the datura.
hold 1 minecraft:flint_and_steel
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_SPEC_LIGHT + 3)).5" >/dev/null
ctl wait 6 >/dev/null
summon_item occultism:datura 1 "$X0.5" 101.0 "$Z_SPEC_LIGHT.5"
[ "$(count_items occultism:datura 8)" = 1 ] || fail "the spectator-probe datura did not summon"
click_floor "$X0" "$Z_SPEC_LIGHT"
SPEC_BLOCK=$(block_name "$X0" 101 "$Z_SPEC_LIGHT")
[ "$SPEC_BLOCK" != "occultism:spirit_fire" ] \
    || fail "a spectator lit spirit fire at z=$Z_SPEC_LIGHT (the click must change nothing)"
[ "$(count_items occultism:datura 8)" = 1 ] \
    || fail "a spectator's click consumed the datura (it must lose nothing)"
echo "  a spectator's lighter left '$SPEC_BLOCK' and the datura untouched"
# 3. The differential: a survival player does the same two clicks and both work,
#    so the guard refuses spectators, not everyone.
ctl command "gamemode survival $PLAYER" >/dev/null
ctl wait 10 >/dev/null
[ "$(mode_now)" = 0 ] || fail "the player did not return to survival: game_mode=$(mode_now)"
hold 2 occultism:chalk_red
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_SPEC + 3)).5" >/dev/null
ctl wait 6 >/dev/null
ctl look-at-block "$X0" 101 "$Z_SPEC" up >/dev/null
ctl wait 2 >/dev/null
ctl use-item >/dev/null
ctl wait 6 >/dev/null
SURV_COLOUR=$(block_colour "$X0" 101 "$Z_SPEC")
[ "$SURV_COLOUR" = "red" ] \
    || fail "a survival player's red chalk left the fire '$SURV_COLOUR' (the guard blocks everyone)"
echo "  a survival player's red chalk made the fire $SURV_COLOUR"
hold 1 minecraft:flint_and_steel
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 $((Z_SPEC_LIGHT + 3)).5" >/dev/null
ctl wait 6 >/dev/null
click_floor "$X0" "$Z_SPEC_LIGHT"
[ "$(block_name "$X0" 101 "$Z_SPEC_LIGHT")" = "occultism:spirit_fire" ] \
    || fail "a survival player's lighter did not light spirit fire (the guard blocks everyone)"
[ "$(count_items occultism:datura 8)" = 0 ] \
    || fail "a survival player's ignition did not consume the datura"
echo "  a survival player's lighter lit the fire and consumed the datura"

echo "== J: /occultism info is operator-only and names a block only for a fire =="
# The host of a cheats-enabled world is an operator (level 4), so the real
# non-operator is a LAN guest: it joins at permission level 0 (the pattern
# solcarrot's e2e uses for the same check).
ctl command "op $PLAYER" >/dev/null
ctl wait 6 >/dev/null
info "$X0" 101 "$Z_CONVERT"
[ "$BLOCK" = "occultism:spirit_fire" ] || fail "an operator cannot read the fire: $INFO_LINE"
echo "  as an operator: $INFO_LINE"
# A position holding no spirit fire must not echo the block it found: that would
# turn the readout into a world probe even for an operator.
say "occultism info $X0 101 $Z_CONTROL"
ctl wait 12 >/dev/null
NOT_FIRE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 -E '^(not a spirit fire|Spirit Fire at)' || true)
[ "$NOT_FIRE" = "not a spirit fire" ] || fail "a non-fire position answered '$NOT_FIRE'; it must name no block"
case "$NOT_FIRE" in
    *:*) fail "the non-fire answer carries a block id: $NOT_FIRE" ;;
esac
echo "  a position holding no fire: $NOT_FIRE"
LAN_PORT=$(ctl lan open 0 | jq -r '.port')
[ "$LAN_PORT" != "null" ] || fail "the host did not open its world to LAN"
echo "  LAN port $LAN_PORT (guest commands off: a guest joins at level 0)"
rm -f "$GSOCK"
HORIZON_HOME=$SCRATCH/guest "$CTL" --socket "$GSOCK" launch \
    --game "$GAME" --headless --resources "$RES" --no-pause >/dev/null
sleep 3
gctl() { "$CTL" --socket "$GSOCK" "$@"; }
CONNECT=$(gctl connect "127.0.0.1:$LAN_PORT" Guest 2>&1 || true)
echo "  guest connect: $CONNECT"
JOINED=false
for _ in $(seq 1 60); do
    if [ "$(gctl status 2>/dev/null | jq -r '.server.joined // false')" = true ]; then
        JOINED=true
        break
    fi
    sleep 2
done
[ "$JOINED" = true ] || fail "the guest never joined the LAN world"
# The guest's own command path is alive at level 0: `/occultism recipes` is a
# level-0 usage of this mod and answers. Without this line the refusal below
# could be "commands are off for guests" rather than the level gate.
gctl command "/occultism recipes" >/dev/null
RECIPE_REPLY=""
for _ in $(seq 1 20); do
    RECIPE_REPLY=$(gctl messages 4 | jq -r '.chat[]' 2>/dev/null \
        | grep -m1 'Spirit fire recipes' || true)
    [ -n "$RECIPE_REPLY" ] && break
    sleep 1
done
echo "  the level-0 guest runs /occultism recipes: ${RECIPE_REPLY:-<none>}"
[ -n "$RECIPE_REPLY" ] \
    || fail "the level-0 guest cannot run the level-0 usage, so the refusal below proves nothing"
# The level-2 usage is refused, and the refusal names no block and no readout.
gctl command "/occultism info $X0 101 $Z_CONVERT" >/dev/null
GUEST_WINDOW=""
for _ in $(seq 1 20); do
    GUEST_WINDOW=$(gctl messages 4 | jq -r '.chat[]' 2>/dev/null || true)
    grep -qiE 'unknown or incomplete command|incorrect argument|permission|Spirit Fire at' <<<"$GUEST_WINDOW" && break
    sleep 1
done
echo "  as a level-0 guest: $(tr '\n' ' ' <<<"$GUEST_WINDOW")"
grep -qiE 'unknown or incomplete command|incorrect argument|permission' <<<"$GUEST_WINDOW" \
    || fail "a non-operator was not refused: $GUEST_WINDOW"
if grep -qE 'Spirit Fire at|block=' <<<"$GUEST_WINDOW"; then
    fail "a non-operator received the readout: $GUEST_WINDOW"
fi
gctl quit >/dev/null 2>&1 || true
sleep 1

# ------------------------------------------------------------ the datura crop
# The mod's first step in survival: demon's dream seeds from grass, a crop on
# farmland that ripens by random tick, harvested by a right click (or mined) into
# the fruit the spirit-fire loop is lit with.
echo "== K: the datura crop binds ages 0..7, its seed plants it, and it grows by random tick =="
# Random ticks off while the ages are written and read back: at the default speed
# a planted crop can advance to age 1 between the click and the read (measured).
end "gamerule random_tick_speed 0"
ctl wait 4 >/dev/null
say "the crop's farmland strip"
end "setblock $((CROP_X0 - 1)) 100 $CROP_Z minecraft:water"
end "setblock $((CROP_X0 + 10)) 100 $CROP_Z minecraft:water"
end "fill $CROP_X0 100 $CROP_Z $((CROP_X0 + 9)) 100 $CROP_Z minecraft:farmland[moisture=7]"
ctl wait 6 >/dev/null
for age in 0 1 2 3 4 5 6 7; do
    end "setblock $((CROP_X0 + age)) 101 $CROP_Z ${CROP}[age=${age}]"
done
ctl wait 6 >/dev/null
for age in 0 1 2 3 4 5 6 7; do
    got=$(state_at $((CROP_X0 + age)) 101 "$CROP_Z")
    case "$got" in
        "$CROP"*"age=$age"*) ;;
        *) fail "age $age reads '$got', expected ${CROP}[age=${age}]" ;;
    esac
done
echo "  ages 0..7 read back on $CROP (the wheat template's crop state)"

say "occultism:datura_seeds plant an age-0 crop on farmland; on stone they plant nothing"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl give "$SEEDS" 4 >/dev/null || fail "cannot give $SEEDS"
ctl wait 4 >/dev/null
ctl select "$SEEDS" >/dev/null || fail "cannot select $SEEDS"
PLANT_X=$((CROP_X0 + 8))
# A click can be lost between the client's aim and the server's tick, and the
# renderer holds the aim for a frame or two under load: retry while the block
# still reads air (a planted crop cannot be planted twice).
PLANTED=""
for attempt in 1 2 3 4; do
    ctl tp "$PLANT_X.5" 101 "$((CROP_Z + 3)).5" >/dev/null
    ctl wait 4 >/dev/null
    ctl look-at-block "$PLANT_X" 100 "$CROP_Z" up >/dev/null
    ctl wait 3 >/dev/null
    ctl interact "$PLANT_X" 100 "$CROP_Z" up >/dev/null
    ctl wait 6 >/dev/null
    PLANTED=$(state_at "$PLANT_X" 101 "$CROP_Z")
    case "$PLANTED" in
        "$CROP"*"age=0"*) break ;;
    esac
    echo "    planting attempt $attempt left '$PLANTED'" >&2
done
case "$PLANTED" in
    "$CROP"*"age=0"*) ;;
    *) fail "the seeds planted '$PLANTED', expected $STATE_YOUNG" ;;
esac
echo "  planted $PLANTED on farmland"
# The differential: the same click on bare stone places nothing.
STONE_X=$((X0 + 17))
ctl tp "$STONE_X.5" 101 3.5 >/dev/null
ctl wait 5 >/dev/null
ctl look-at-block "$STONE_X" 100 0 up >/dev/null
ctl wait 2 >/dev/null
ctl interact "$STONE_X" 100 0 up >/dev/null
ctl wait 6 >/dev/null
[ "$(block_name "$STONE_X" 101 0)" = "minecraft:air" ] \
    || fail "a seed planted $(block_name "$STONE_X" 101 0) on stone"
echo "  the same click on stone left air"

say "the crop grows where the wheat template grows: random_tick_speed 0 keeps the age, 400 advances it"
ctl wait 100 >/dev/null
ctl wait 100 >/dev/null
STILL=$(age_at "$PLANT_X" 101 "$CROP_Z")
[ "$STILL" = 0 ] || fail "with random_tick_speed 0 the crop advanced to age '$STILL'"
echo "  control: the planted crop is still age $STILL after 200 ticks at random_tick_speed 0"
end "gamerule random_tick_speed 400"
GREW=0
for _ in $(seq 1 40); do
    ctl wait 20 >/dev/null
    # Keep the plot wet: a dry farmland turns to dirt on its own random tick and
    # takes the crop above it with it (the pin's farmland rule).
    end "fill $CROP_X0 100 $CROP_Z $((CROP_X0 + 9)) 100 $CROP_Z minecraft:farmland[moisture=7]"
    NOW=$(age_at "$PLANT_X" 101 "$CROP_Z")
    if [ -n "$NOW" ] && [ "$NOW" -gt 0 ]; then GREW=1; break; fi
done
[ "$GREW" = 1 ] || fail "the crop did not grow under random ticks (age $(age_at "$PLANT_X" 101 "$CROP_Z"))"
for _ in $(seq 1 40); do
    RIPE=$(age_at "$PLANT_X" 101 "$CROP_Z")
    if [ "$RIPE" = 7 ]; then break; fi
    ctl wait 20 >/dev/null
    end "fill $CROP_X0 100 $CROP_Z $((CROP_X0 + 9)) 100 $CROP_Z minecraft:farmland[moisture=7]"
done
RIPE=$(age_at "$PLANT_X" 101 "$CROP_Z")
[ "$RIPE" = 7 ] || fail "the crop ripened to age '$RIPE', expected 7"
echo "  the planted crop advanced from age 0 to age $RIPE under random ticks"
end "gamerule random_tick_speed 0"

echo "== L: a ripe crop is harvested by a right click, replanted at age 0 =="
# Upstream's ReplantableCropsBlock: a bare-hand right click on a ripe crop rolls
# the block's own loot table with the held stack, drops it at the block's centre
# and leaves the crop at age 0.
ctl command "clear @s" >/dev/null
# Upstream harvests in `useItemOn`, so the click carries a stack (a stick here);
# a bare hand is the same path with an empty stack.
hold 1 minecraft:stick
ctl wait 4 >/dev/null
HARVEST_X=$PLANT_X
# A click can be lost between the client's aim and the server's tick: retry while
# the crop still reads ripe (a harvest resets it, so a second click is harmless).
for attempt in 1 2 3; do
    click_block "$HARVEST_X" "$CROP_Z"
    HARVESTED=$(state_at "$HARVEST_X" 101 "$CROP_Z")
    case "$HARVESTED" in "$CROP"*"age=7"*) continue ;; esac
    break
done
case "$HARVESTED" in
    "$CROP"*"age=0"*) ;;
    *) fail "the right click left '$HARVESTED', expected $STATE_YOUNG" ;;
esac
# The harvest's drops may be picked up while the player moves to the next probe,
# so the count is the inventory's plus the world's (invariant under pickup).
# The harvest's drops may move into the inventory while a count runs, so the world
# is read *first* and the inventory second: an item picked up between the two
# snapshots is still counted (in the world), never in neither (measured: a run
# read fruit=0 with the seeds present, 2026-11-21).
fruit_total() {
    local world
    world=$(count_items "$CROP" 6)
    echo $(( world + $(inv_count "$CROP") ))
}
seed_total() {
    local world
    world=$(count_items "$SEEDS" 6)
    echo $(( world + $(inv_count "$SEEDS") ))
}
FRUIT=$(fruit_total)
SEED_DROPS=$(seed_total)
echo "  before the assertions: fruit=$FRUIT seeds=$SEED_DROPS (state $(state_at "$HARVEST_X" 101 "$CROP_Z"))"
echo "  inventory: $(ctl inventory | jq -c '.counts')"
echo "  item entities within 6: $(ctl entities 6 item | jq -c '[.entities[] | select(.kind == "item") | {item:.item.item,count:.item.count}]')"
if [ "$NEGATIVE" = 1 ]; then
    echo "(negative: asserting a ripe crop drops no fruit)"
    [ "$FRUIT" = 0 ] || fail "negative: the ripe crop dropped $FRUIT fruit (this run must fail)"
fi
if [ "$NEGATIVE" != 1 ]; then
    [ "$FRUIT" = 1 ] || fail "the ripe crop dropped $FRUIT fruit (and $SEED_DROPS seeds), expected exactly 1"
    # The seed pool's own count is the engine's `apply_bonus` read (measured 1..6
    # on a bare hand); the rule under test is that a ripe crop yields a seed.
    [ "$SEED_DROPS" -ge 1 ] || fail "the ripe crop dropped $SEED_DROPS seeds, expected at least 1"
fi
echo "  the harvest replanted $HARVESTED and dropped $FRUIT fruit + $SEED_DROPS seeds"
# A young crop is not harvested: the click changes nothing and drops nothing.
YOUNG_X=$((CROP_X0 + 9))
end "setblock $YOUNG_X 101 $CROP_Z ${CROP}[age=3]"
ctl wait 4 >/dev/null
click_block "$YOUNG_X" "$CROP_Z"
YOUNG=$(state_at "$YOUNG_X" 101 "$CROP_Z")
case "$YOUNG" in
    "$CROP"*"age=3"*) ;;
    *) fail "a right click on a young crop left '$YOUNG'" ;;
esac
[ "$(fruit_total)" = "$FRUIT" ] || fail "a young crop was harvested ($(fruit_total) fruit, was $FRUIT)"
echo "  a young crop ($YOUNG) stayed and dropped nothing"

echo "== M: the carried loot table: a ripe crop drops its fruit and seed, a young one a seed =="
ctl command "gamemode survival @s" >/dev/null
ctl wait 6 >/dev/null
# A crop only survives on farmland, so the break station gets its own strip.
end "setblock $((CROP_X0 - 1)) 100 $Z_BREAK minecraft:water"
end "setblock $((CROP_X0 + 10)) 100 $Z_BREAK minecraft:water"
end "fill $CROP_X0 100 $Z_BREAK $((CROP_X0 + 9)) 100 $Z_BREAK minecraft:farmland[moisture=7]"
ctl wait 6 >/dev/null
RIPE_X=$((CROP_X0 + 5))
YOUNG_X=$((CROP_X0 + 6))
end "setblock $RIPE_X 101 $Z_BREAK $STATE_RIPE"
end "setblock $YOUNG_X 101 $Z_BREAK $STATE_YOUNG"
ctl wait 6 >/dev/null
[ "$(state_at "$RIPE_X" 101 "$Z_BREAK")" = "$STATE_RIPE" ] || fail "the ripe crop did not place"
[ "$(state_at "$YOUNG_X" 101 "$Z_BREAK")" = "$STATE_YOUNG" ] || fail "the young crop did not place"
ctl command "clear @s" >/dev/null
RIPE_REPLY=$(mine "$RIPE_X" 101 "$Z_BREAK") || fail "cannot mine the ripe crop"
echo "  ripe crop drops: $(jq -c '.drops' <<<"$RIPE_REPLY")"
[ "$(state_at "$RIPE_X" 101 "$Z_BREAK")" = "minecraft:air" ] || fail "the ripe crop did not break"
RIPE_FRUIT=$(drops_of "$RIPE_REPLY" "$CROP")
RIPE_SEEDS=$(drops_of "$RIPE_REPLY" "$SEEDS")
[ "$RIPE_FRUIT" = 1 ] || fail "a mined ripe crop dropped $RIPE_FRUIT fruit, expected exactly 1"
[ "$RIPE_SEEDS" -ge 1 ] || fail "a mined ripe crop dropped $RIPE_SEEDS seeds"
YOUNG_REPLY=$(mine "$YOUNG_X" 101 "$Z_BREAK") || fail "cannot mine the young crop"
echo "  young crop drops: $(jq -c '.drops' <<<"$YOUNG_REPLY")"
YOUNG_FRUIT=$(drops_of "$YOUNG_REPLY" "$CROP")
YOUNG_SEEDS=$(drops_of "$YOUNG_REPLY" "$SEEDS")
[ "$YOUNG_FRUIT" = 0 ] || fail "a mined age-0 crop dropped $YOUNG_FRUIT fruit"
[ "$YOUNG_SEEDS" = 1 ] || fail "a mined age-0 crop dropped $YOUNG_SEEDS seeds, expected exactly 1"
echo "  the table's two branches: age 7 -> 1 fruit + $RIPE_SEEDS seeds, age 0 -> 1 seed and no fruit"

echo "== N: short grass: a hand break can yield the jar's 2% seed; shears never do =="
# The jar's two loot modifiers: a 0.02 chance of one seed from short_grass (and
# tall_grass), inverted on shears. The client lands a fresh patch of aimed breaks
# and drops them as the session goes on (measured: 33 of 400 over 16 rounds), so
# a 200-roll statistical assertion is not reproducible here and each round takes
# up to three *fresh* patches. The *rule* is pinned by runtime::tests (red then
# green) and the deterministic in-game proof is the shears differential below:
# the sheared breaks roll the grass's own loot and the seed is suppressed. The
# hand round reports its seeds; a seed it observes is asserted.
SEED_X=$((X0 + 16))
SEED_Z=$((Z_SEED))
SHEAR_Z=$((Z_SEED + 8))
SEED_W=5
# The End's arena floor has no sky light, so hostile mobs spawn on it: two rounds
# of 25 aimed breaks take minutes, long enough for an Enderman to kill the
# survival player and leave the death screen open, which makes every later
# `ctl break` answer "the dead screen is open" (measured: 9 and 11 of 25 landed,
# 2026-11-21). Peaceful keeps them away; the rounds still kill whatever is there.
end "difficulty peaceful"
end "kill @e[type=!player]"
ctl wait 20 >/dev/null
# Mine one block of the grass patch, standing two rows south of it in the End:
# outside the patch, or on a row this loop has already broken, so the player is
# never teleported *into* a solid block (the old `mine` teleported to x+1 in the
# same row, inside the next grass block).
#
# The teleport is confirmed from the client before the aim: under load the client
# learns `tp @s` a few ticks late, and a break aimed from the old position fails
# fast ("no line of sight") — that is what turned a 25-of-25 round into 3 of 25
# in a full run (measured, 2026-11-21). Each attempt re-teleports and re-aims.
mine_patch() { # mine_patch X Z -> the last ctl break reply
    local x=$1 z=$2 reply attempt
    for attempt in 1 2 3 4; do
        ctl command "execute in minecraft:the_end run tp @s $((x)).5 101 $((z - 2)).5" >/dev/null
        wait_near "$((x)).5" 101 "$((z - 2)).5"
        ctl look-at-block "$x" 101 "$z" up >/dev/null
        ctl wait 3 >/dev/null
        reply=$(ctl break "$x" 101 "$z" 2>/dev/null) && { echo "$reply"; return 0; }
        echo "    break attempt $attempt at $x,101,$z failed: ${reply:-<no reply>}" >&2
    done
    return 1
}

# Wait until the client reports the position a `tp @s` put it at (the client learns
# a server teleport a tick or two late), up to TICKS polls. Not fatal on its own:
# the aim and the break below are what the phase asserts.
wait_near() { # wait_near X Y Z [TICKS]
    local x=$1 y=$2 z=$3 polls=${4:-20} pos
    for _ in $(seq 1 "$polls"); do
        pos=$(ctl state 2>/dev/null | jq -r '.player.position | @csv' 2>/dev/null || true)
        if [ -n "$pos" ] && awk -F, -v x="$x" -v y="$y" -v z="$z" \
            'BEGIN { exit !((($1-x)^2 + ($2-y)^2 + ($3-z)^2) < 0.36) }' <<<"$pos"; then
            return 0
        fi
        ctl wait 2 >/dev/null
    done
    return 0
}

hand_round() { # hand_round X Z ITEM -> "<landed> <seeds> <grass drops>"
    local x=$1 z=$2 item=$3 landed=0 seeds=0 grass=0 reply patch pz
    ctl command "gamemode survival @s" >/dev/null
    # A death screen left from a previous round blocks every ctl action.
    ctl close >/dev/null 2>&1 || true
    ctl command "clear @s" >/dev/null
    if [ -n "$item" ]; then hold 1 "$item"; fi
    # Up to three *fresh* patches, each used once: the client lands the first
    # patch of aimed breaks and stops landing them as the session goes on under
    # load (measured 25 of 25 on the first round and 9 of 25 on the second at
    # load average 13, 2026-11-21; 33 of 400 over 16 rounds on 2026-11-21 too).
    # The round still has to land 20 of its 25 aims, so this adapts to the load
    # without weakening what the round proves.
    for patch in 0 1 2; do
        pz=$((z + patch * 6))
        end "kill @e[type=!player]"
        ctl wait 10 >/dev/null
        end "fill $x 100 $pz $((x + SEED_W - 1)) 100 $((pz + SEED_W - 1)) minecraft:dirt"
        end "fill $x 101 $pz $((x + SEED_W - 1)) 101 $((pz + SEED_W - 1)) minecraft:short_grass"
        ctl wait 6 >/dev/null
        if [ "$(block_name $((x + 1)) 101 $((pz + 1)))" != "minecraft:short_grass" ]; then
            continue
        fi
        for dx in 0 1 2 3 4; do
            for dz in 0 1 2 3 4; do
                # stderr passes through: a failed attempt names its reply in the
                # log (and keeps the e2e guard's idle detector fed during a slow
                # round), while stdout is the break reply itself.
                reply=$(mine_patch "$((x + dx))" "$((pz + dz))") || reply=""
                case "$reply" in
                    *'"block_before"'*)
                        landed=$((landed + 1))
                        [ "$(drops_of "$reply" minecraft:short_grass)" -ge 1 ] && grass=$((grass + 1)) ;;
                esac
            done
        done
        [ "$landed" -ge 20 ] && break
    done
    ctl wait 10 >/dev/null
    seeds=$(( $(inv_count "$SEEDS") + $(count_items "$SEEDS" 8) ))
    echo "$landed $seeds $grass"
}
read -r HAND_LANDED HAND_SEEDS HAND_GRASS <<<"$(hand_round "$SEED_X" "$SEED_Z" minecraft:stick)"
echo "  hand breaks: $HAND_LANDED of 25 landed, $HAND_SEEDS datura seed(s), $HAND_GRASS dropped grass"
[ "$HAND_LANDED" -ge 20 ] \
    || fail "only $HAND_LANDED of 25 aimed hand breaks landed, so the harness proves nothing"
if [ "$HAND_SEEDS" -ge 1 ]; then
    echo "  the jar's 2% seed rolled in this run"
else
    echo "  no seed in this run's $HAND_LANDED rolls (2%: about 40% of 25-roll runs yield none; the rule is pinned by runtime's tests)"
fi
read -r SHEAR_LANDED SHEAR_SEEDS SHEAR_GRASS <<<"$(hand_round "$SEED_X" "$SHEAR_Z" minecraft:shears)"
echo "  shears breaks: $SHEAR_LANDED of 25 landed, $SHEAR_GRASS dropped grass, $SHEAR_SEEDS datura seed(s)"
[ "$SHEAR_LANDED" -ge 20 ] \
    || fail "only $SHEAR_LANDED of 25 aimed shears breaks landed, so the control proves nothing"
[ "$SHEAR_SEEDS" = 0 ] \
    || fail "shears yielded $SHEAR_SEEDS datura seeds; the modifier is inverted on shears"
[ "$SHEAR_GRASS" -ge 1 ] \
    || fail "the shears breaks dropped no short_grass, so no loot rolled and the control proves nothing"

phase_loot

phase_craft

phase_building

phase_bricks

phase_polished

phase_cobble

phase_redstone

phase_pedestal

phase_bowl

phase_chalk

phase_tablet

phase_stonecut

echo "PASS: spirit fire lights from datura and flint and steel, converts a 3-feather stack into exactly 3 awakened feathers with the recipe table read from the carried data (26 of 27 rules), leaves an unmatched item alone, keeps a feather on plain stone, takes its colour from a gold chalk, and ignores datura two blocks away; /occultism info is operator-only, answers a non-fire position without naming a block and is refused to a level-0 LAN guest while /occultism recipes still answers it; a spectator's chalk and lighter change nothing and consume no datura, while a survival player's do; the datura crop binds ages 0..7, occultism:datura_seeds plant it on farmland (and not on stone), it ripens by random tick, a right click harvests a ripe crop into exactly 1 fruit and at least 1 seed and replants it while a young one is left alone, mining the crop gives the carried loot table's two branches, and short grass broken with shears rolls its own loot and never the jar's 2% seed; and the four carried block loot tables give otherstone and otherrock nothing to a bare hand, exactly one othercobblestone/othercobblerock to a wooden pickaxe and themselves to a silk-touch pickaxe, and one otherflower and one natural sapling to a bare hand; and the two carried crafting recipes craft at a real table: a 3x3 grid of datura fruit, of datura seed and a mixed fruit/seed grid each make exactly one occultism:demons_dream_essence with every ingredient consumed, nine dirt and eight fruit with one dirt make nothing, and the shapeless awakened feather + purified ink + taboo book make one occultism:book_of_binding_empty; and the six carried otherstone/otherrock building recipes craft at a real table (six slabs from three stones in the middle row, four stairs, six walls, every ingredient consumed, three dirt making nothing) and each block reads its stone template's default state, drops nothing to a bare hand and exactly one of itself to a wooden pickaxe, two walls connect, and a double slab drops two; and the twelve bricks-tier blocks work the same way: their ten grids craft at a real table with every ingredient consumed (four dirt in the same 2x2 and three stones one cell short make nothing), a chain in one table turns twelve stones into bricks, six bricks into four stairs, three bricks into six slabs and two slabs into one chiseled block, the cracked pair smelts in a real furnace while a fuel-less furnace stays idle, all twelve read their stone template's default state, drop nothing to a bare hand and exactly one to a wooden pickaxe, a brick wall pair connects, and a double brick slab drops two; and the eight polished-tier blocks work the same way: their six grids craft at a real table with every ingredient consumed (four polished blocks in the 2x2 and four dirt make nothing), the polished pair smelts in a real furnace (one raw stone + one coal makes exactly one polished block and consumes the input, while a fuel-less furnace stays idle), all eight read their stone template's default state, drop nothing to a bare hand and exactly one to a wooden pickaxe, a polished wall pair connects, a double polished slab drops two, and polished_otherrock_slab (the table the jar omits) drops one; and the eight cobblestone-tier blocks work the same way: their six grids craft at a real table with every ingredient consumed (four cobble blocks in the 2x2 and four dirt make nothing), all eight read their stone template's default state, drop nothing to a bare hand and exactly one to a wooden pickaxe, a cobble wall pair connects, a double cobble slab drops two, othercobblerock_slab (the table the jar omits) drops one, and the raw stones drop one othercobblestone/othercobblerock without silk touch and keep themselves with it; and the four redstone-pair blocks work the same way: one stone in a real table makes one button and two stones in one row make one pressure plate (every ingredient consumed, while a cobble button grid and a cobble plate grid make nothing and one stone in the plate grid makes the button rather than the plate), the cobble block smelts back into the stone in a real furnace at 200 ticks and 0.5 experience while a fuel-less furnace stays idle, each block reads its template's default state and drops exactly one of itself to a bare hand and one to a wooden pickaxe (otherstone's hand break dropping nothing, the differential), each plate reads powered=true under the player and false after stepping off, and each button reads powered=true after a click, is still pressed ten ticks later and false after its press duration; and the frames and pedestals work the same way: eight stones in a ring make one occultism:otherstone_frame or occultism:otherrock_frame (items, given and refused by /setblock), five slabs + one stone make one occultism:otherstone_pedestal or occultism:otherstone_pedestal_silver (whose file is otherrock_pedestal.json), every ingredient is consumed, a ninth stone in the frame's hole and one slab short of the pedestal make nothing, and each pedestal reads its state, drops nothing to a bare hand and exactly one to a wooden pickaxe (otherstone's hand break dropping nothing, the differential); and the six sacrificial bowls work end to end: their grids make one bowl each with every ingredient consumed (four stones and seven gold making nothing), each of the six places and reads back, a damaged diamond sword goes into a plain and a golden bowl and comes back to an empty hand with its component after being read through the block entity, five dirt put exactly one in the bowl and leave four in the hand, breaking a full bowl drops the bowl and spills its stack, a feather in a plain bowl over spirit fire becomes an awakened feather while a golden bowl leaves it a feather, a sneaking click changes nothing, and every bowl drops one to a bare hand and one to a wooden pickaxe with otherstone's hand break at nothing; and the 17 impure-chalk recipes craft at a real table: each grid makes its impure chalk with every ingredient consumed and the grid emptied, the nested chalk base resolves through both burnt stones and every tag-keyed cell through its member (no 'invalid ingredient' in the game log), a pink grid with dirt for one demonic meat and a two-column chalk-base grid with no otherworld wood make nothing, and the polished pair smelts into burnt otherstone/otherrock in a real furnace (200 ticks, 0.15 experience) while a fuel-less one stays idle; and all 32 carried stonecutting recipes cut at a real stonecutter: every ingredient fed into slot 600, its occultism:stonecutting/<stem> offer selected by a GUI cell click, slot 650's output id and count asserted with the input consumed, a different selected cell yields a different result, and vanilla stone offers no occultism recipe"
