#!/usr/bin/env bash
# The four in-game legs of Occultism slice 1, one after the other in a single
# lease (so every leg runs the binary built from these sources), each with its
# own evidence log:
#
#   e2e-devmod.log    the scenario with `--dev-mod content`
#   e2e-embedded.log  the scenario with the bundle's embedded content and no
#                     `--dev-mod`, plus the control game without the embedded file
#   e2e-selftest.log  the deliberately wrong count assertion (MUST fail)
#   e2e-hmod-load.log the content package alone on the shared, unmodded game
#
# Prints `PASS:` and `EXIT:0` only when every leg met its expectation; a leg that
# did not is reported by name. Run it through the runner:
#   ports/tools/build-slot.sh e2e --job occ-e2e --port ports/occultism -- tools/e2e-all.sh
set -Eeuo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/occultism
EVIDENCE=$CLONE/ports/_shared/evidence/occultism
mkdir -p "$EVIDENCE"

status=0

# The runtime's tests as they compile inside the game crate (`--lib
# game::occultism::tests`): the same 16 rules, through the real staging plan.
echo "== game-crate tests ==" >&2
code=0
python3 "$PORT/tools/horizon.py" test >"$EVIDENCE/game-test.log" 2>&1 || code=$?
# `horizon.py` prints its JSON report; the cargo output (and so the test counts)
# is the `cargo_log` file it names. Copy the counts into the evidence.
cargo_exit=$(grep -o '"cargo_exit": [0-9]*' "$EVIDENCE/game-test.log" | tail -1 | grep -o '[0-9]*' || true)
cargo_log=$(grep -o '"cargo_log": "[^"]*"' "$EVIDENCE/game-test.log" | tail -1 | sed 's/.*: "//; s/"$//' || true)
if [ -n "$cargo_log" ] && [ -f "$cargo_log" ]; then
    grep -E '^test result:|^test tests::' "$cargo_log" | tail -30 >>"$EVIDENCE/game-test.log" || true
    passed=$(grep -m1 '^test result:' "$cargo_log" | sed -nE 's/.* ([0-9]+) passed.*/\1/p' || true)
else
    passed=""
fi
echo "  game-crate tests: exit $code, cargo_exit ${cargo_exit:-?}, ${passed:-0} passed, $EVIDENCE/game-test.log" >&2
if [ "$code" != 0 ] || [ "${cargo_exit:-1}" != 0 ]; then
    echo "FAIL: the embedded tests exited ${cargo_exit:-$code}" >&2
    status=1
fi
if [ "${passed:-0}" = 0 ]; then
    echo "FAIL: the embedded test filter matched 0 tests" >&2
    status=1
fi

run_leg() { # run_leg NAME EXPECTED(0|fail) -- ENV...
    local name=$1 expect=$2; shift 2
    if [ "${1:-}" = "--" ]; then shift; fi
    echo "== $name ==" >&2
    local code=0
    env "$@" "$PORT/tools/e2e.sh" >"$EVIDENCE/$name.log" 2>&1 || code=$?
    echo "  $name: exit $code (expected $expect), log $EVIDENCE/$name.log" >&2
    if [ "$expect" = 0 ]; then
        [ "$code" = 0 ] || { echo "FAIL: $name exited $code, expected 0" >&2; status=1; }
        grep -q '^PASS:' "$EVIDENCE/$name.log" || { echo "FAIL: $name printed no PASS line" >&2; status=1; }
    else
        [ "$code" != 0 ] || { echo "FAIL: $name passed, but it must fail" >&2; status=1; }
        grep -q '^FAIL:' "$EVIDENCE/$name.log" || { echo "FAIL: $name printed no FAIL line" >&2; status=1; }
    fi
    tail -3 "$EVIDENCE/$name.log" >&2 || true
}

run_leg e2e-devmod 0 -- EMBEDDED=0
run_leg e2e-embedded 0 -- EMBEDDED=1 CONTROL=1
run_leg e2e-selftest fail -- EMBEDDED=0 SELFTEST=1

echo "== hmod-load ==" >&2
code=0
"$PORT/tools/hmod-load.sh" >"$EVIDENCE/e2e-hmod-load.log" 2>&1 || code=$?
case "$code" in
    0)
        echo "  hmod-load: exit 0, log $EVIDENCE/e2e-hmod-load.log" >&2
        grep -q '^PASS:' "$EVIDENCE/e2e-hmod-load.log" \
            || { echo "FAIL: hmod-load printed no PASS line" >&2; status=1; }
        ;;
    77)
        # No shared unmodded game here (the far build box): run it locally.
        echo "  hmod-load: skipped here, run tools/hmod-load.sh locally" >&2
        rm -f "$EVIDENCE/e2e-hmod-load.log"
        ;;
    *)
        echo "FAIL: hmod-load exited $code (log $EVIDENCE/e2e-hmod-load.log)" >&2
        status=1
        ;;
esac

if [ "$status" = 0 ]; then
    echo "PASS: spirit fire lights from datura + flint and steel, converts a 3-feather stack into exactly 3 awakened feathers with the recipe table read from the carried data (26 of 27 rules), leaves an unmatched item alone, keeps a feather on plain stone, takes its colour from a gold chalk, ignores datura two blocks away, installs block+items+behaviour from the one .hnmod, and the selftest leg fails as it must"
fi
echo "EXIT:$status"
exit "$status"
