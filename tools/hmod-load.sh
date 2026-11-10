#!/usr/bin/env bash
# Proves the built content package (content/occultism.hmod) loads on a real game:
# the shared, unmodded 1d63e0a game with `--dev-mod <the .hmod>` must register
# the spirit-fire block and the datura item. A second launch WITHOUT `--dev-mod`
# is the negative control: the same ids must be unknown there, so the check
# cannot pass vacuously.
set -Eeuo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PIN_BIN/horizon-client}
HMOD=${HMOD:-$CLONE/ports/occultism/content/occultism.hmod}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/occultism-hmod.sock}

# This leg needs the *unmodded* shared game: a native build would carry the
# bundle's own content and the negative control would prove nothing. Where the
# published pair is absent (the far build box), skip: run this leg locally.
if [ ! -x "$GAME" ] || [ ! -x "$CTL" ]; then
    echo "SKIP: no shared unmodded game at $PIN_BIN; run tools/hmod-load.sh locally" >&2
    exit 77
fi
# The published pair was built on the campaign's main box: an older glibc
# (another machine, the far build box) cannot load it at all.
glibc_need=$(objdump -T "$GAME" 2>/dev/null | grep -o 'GLIBC_[0-9.]*' | sort -V | tail -1 | sed 's/GLIBC_//' || true)
glibc_have=$(ldd --version 2>/dev/null | head -1 | grep -o '[0-9.]*$' || true)
if [ -n "$glibc_need" ] && [ -n "$glibc_have" ] \
    && [ "$(printf '%s\n%s\n' "$glibc_need" "$glibc_have" | sort -V | tail -1)" != "$glibc_have" ]; then
    echo "SKIP: the shared game needs GLIBC_$glibc_need, this box has $glibc_have; run tools/hmod-load.sh locally" >&2
    exit 77
fi
[ -f "$HMOD" ] || { echo "missing package: $HMOD" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/occultism-hmod-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "${KEEP:-0}" = 1 ]; then echo "kept scratch: $SCRATCH" >&2; else rm -rf "$SCRATCH" 2>/dev/null || true; fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch WORLD [--dev-mod PATH]
    local world=$1; shift
    local out
    rm -f "$SOCK"
    cd "$SCRATCH"
    out=$("$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" "$@" 2>&1) || {
        echo "launch failed: $(tr -d '\n' <<<"$out" | cut -c1-400)" >&2
        return 1
    }
    ctl wait-ready 120
    ctl wait 20
}

# A stone floor in the End's void, so a fire block has something to stand on.
# The player goes there first: a chunk only loads around a player, and `setblock`
# in an unloaded chunk does nothing.
floor() {
    ctl command "execute in minecraft:the_end run tp @s 300.5 104 0.5" >/dev/null || true
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block 300 100 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill 298 100 -2 302 100 2 minecraft:stone" >/dev/null || true
    ctl wait 8
    ctl command "execute in minecraft:the_end run tp @s 300.5 101 0.5" >/dev/null || true
    ctl wait 6
}

echo "== launch WITH the package (--dev-mod $HMOD) =="
launch OccultismHmod --dev-mod "$HMOD"
floor
ctl command "item replace entity @s hotbar.0 with occultism:datura 1" >/dev/null
ctl wait 4
ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "occultism:datura")] | length >= 1' >/dev/null \
    || fail "the package did not register its datura item: $(ctl inventory | jq -c '{hotbar,main}')"
echo "item: occultism:datura"
ctl command "execute in minecraft:the_end run setblock 300 101 0 occultism:spirit_fire" >/dev/null
ctl wait 6
block=$(ctl block 300 101 0 | jq -r '.block // empty')
echo "block: $block"
[ "$block" = "occultism:spirit_fire" ] || fail "the package did not register its block (got '$block')"
colour=$(ctl block 300 101 0 | jq -r '.state // empty' | sed -nE 's/.*color=([a-z_]+).*/\1/p')
echo "block state: colour=$colour"
[ "$colour" = "white" ] || fail "the spirit fire's default colour is '$colour', expected white"
ctl quit >/dev/null 2>&1 || true
sleep 2

echo "== negative control: the SAME ids without --dev-mod =="
launch OccultismHmodControl
floor
ctl command "item replace entity @s hotbar.0 with occultism:datura 1" >/dev/null 2>&1 || true
ctl wait 4
ctl inventory | jq -e '[.hotbar[], .main[] | select(.item != null) | .item | select(startswith("occultism:"))] | length == 0' >/dev/null \
    || fail "the unmodded game gave an occultism item: $(ctl inventory | jq -c '{hotbar,main}')"
echo "item: none"
ctl command "execute in minecraft:the_end run setblock 300 101 0 occultism:spirit_fire" >/dev/null 2>&1 || true
ctl wait 6
block=$(ctl block 300 101 0 | jq -r '.block // empty')
echo "block: $block"
[ "$block" != "occultism:spirit_fire" ] || fail "the unmodded game placed spirit fire"
echo "PASS: occultism.hmod registered the spirit-fire block (colour=white) and the datura item on a real, unmodded game, and the same ids are unknown without the package"
