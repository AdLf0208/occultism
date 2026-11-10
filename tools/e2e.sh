#!/usr/bin/env bash
# In-game check for Occultism slice 1: the spirit-fire conversion loop.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs the
# console path and bypasses the native `perform_command` hook. Item entities are
# summoned at exact positions (upstream's own flow is "throw it on the ground";
# the harness only needs the entity there, and a throw's landing spot is not
# deterministic).
#
# The End, in the void at x=200 (away from the central island and its ambient
# item entities), on a stone floor at y=100 with clear air above:
#   z=0   the conversion station: datura -> spirit fire -> feathers converted;
#   z=4   the control: the same feather on plain stone stays a feather;
#   z=-4  the recolour station: a gold chalk paints the fire yellow;
#   z=8   the far-datura case: datura two blocks out must not be consumed and
#         must not light spirit fire (vanilla still lights its own fire).
#
# Default run: the game loads `content/` as a dev mod.
# EMBEDDED=1: the game is launched with **only** the native bundle (no
# `--dev-mod`) and the content comes from the bundle's embedded copy
# (`mixins/occultism/data/occultism-content.hmod`, the AE2 pattern). The run also
# checks the file the helper wrote; CONTROL=1 (default with EMBEDDED) first
# launches the same binary with `.horizon-cache` unusable, where the ids must be
# unknown.
#
# SELFTEST=1 flips the count assertion (3 awakened feathers -> 2): it must FAIL,
# proving the scenario can fail.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${OCC_DEBUG:-}" ] || set -x

EMBEDDED=${EMBEDDED:-0}
CONTROL=${CONTROL:-$EMBEDDED}

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/occultism
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/occultism-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=OccultismE2E
CONTROL_WORLD=OccultismControl
# Where the arena stands: the End's void, far from the central island.
X0=200
Z_CONVERT=0
Z_CONTROL=4
Z_PAINT=-4
Z_FAR=8

# The published per-pin pair may be absent (a fresh tree, the far build box):
# build the control client for this pin instead, beside the game.
if [ ! -x "$CTL" ]; then
    echo "  no published horizon-ctl at $CTL; building one for this pin" >&2
    python3 "$PORT/tools/horizon.py" ctl >/dev/null 2>&1 || true
    CTL=${CTL_FALLBACK:-$PORT/target/horizon-cargo/fast/horizon-ctl}
fi
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/occultism-home.XXXXXX)
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

launch() { # launch [WORLD]
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --headless
                --wait --resources "$RES")
    if [ "$EMBEDDED" = 1 ]; then
        echo "  launch: no --dev-mod; the bundle carries its own content" >&2
    else
        args+=(--dev-mod "$PORT/content")
    fi
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
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
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 100 -14 $((X0 + 4)) 100 14 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 101 -14 $((X0 + 4)) 108 14 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl wait 20 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 6 >/dev/null
    [ "$(block_name "$X0" 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_name "$X0" 101 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# `/occultism info X Y Z`: the line, then the parsed fields in BLOCK, COLOUR,
# CONVERSIONS, LAST, RULES, FILES.
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

# The control for EMBEDDED=1: the same binary (every hook is in it) with the
# embedded content made unavailable — `.horizon-cache` is a plain file, so the
# helper can neither compare nor write its content file and falls back to
# `Continue`. Both ids must then be unknown; if they resolve, the run proves
# nothing about the embedded file.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    rm -rf "$SCRATCH/.horizon-cache"
    : > "$SCRATCH/.horizon-cache"
    launch "$CONTROL_WORLD"
    arena
    ctl command "item replace entity @s hotbar.0 with occultism:datura 1" >/dev/null || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "occultism:datura")] | length == 0' >/dev/null \
        || fail "the control game has the datura item, so the content did not come from the embedded file"
    ctl command "execute in minecraft:the_end run setblock $X0 101 0 occultism:spirit_fire" >/dev/null || true
    ctl wait 4 >/dev/null
    local name
    name=$(block_name "$X0" 101 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    [ -f "$SCRATCH/.horizon-cache" ] || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no datura item, the block stayed $name"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SCRATCH/.horizon-cache"
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for
# byte the delivered content mod.
embedded_content_check() {
    local cache=$SCRATCH/.horizon-cache/occultism-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/occultism.hmod" \
        || fail "the embedded content differs from content/occultism.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/occultism.hmod"
}

echo "== launch (The End arena, content + native bundle) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi
arena

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
[ "$CONVERSIONS" = 1 ] || fail "the block counted $CONVERSIONS conversions, expected 1"
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
[ "$CONVERSIONS" = 1 ] || fail "a dirt conversion was counted: $CONVERSIONS"
echo "  the dirt is still dirt and the counter is still $CONVERSIONS"

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

echo "PASS: spirit fire lights from datura and flint and steel, converts a 3-feather stack into exactly 3 awakened feathers with the recipe table read from the carried data (26 of 27 rules), leaves an unmatched item alone, keeps a feather on plain stone, takes its colour from a gold chalk, and ignores datura two blocks away"
