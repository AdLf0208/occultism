#!/usr/bin/env bash
# Package both artifacts:
#  - the content mod `.hmod` via the pin's own cargo-horizon-mod (build + check)
#  - the embedded copy inside the bundle (`mixins/occultism/data/`)
#  - the source-only `.hnmod` (tools/package.py, deterministic bytes, port root)
# Run inside a build slot (it compiles the packaging tool):
#   ports/tools/build-slot.sh run --job occ-pack --port ports/occultism \
#       --step package -- env CARGO_INCREMENTAL=0 tools/package.sh
set -euo pipefail

PORT=$(cd "$(dirname "$0")/.." && pwd)
HORIZON=${HORIZON:-$PORT/../_shared/pins/horizon-1d63e0a}
VERSION=$(sed -nE 's/^version = "(.*)"$/\1/p' "$PORT/content/horizon-mod.toml")
LOGS=$PORT/target/ladder
mkdir -p "$LOGS"
cd "$PORT"

CARGO_TARGET_DIR=$PORT/target/carry-tools
export CARGO_TARGET_DIR
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod build) \
    2>&1 | tee "$LOGS/package-hmod.log"
cp content/target/horizon-mod/occultism.hmod content/occultism.hmod
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod check occultism.hmod) \
    2>&1 | tee -a "$LOGS/package-hmod.log"

mkdir -p mixins/occultism/data
cp content/occultism.hmod mixins/occultism/data/occultism-content.hmod

python3 tools/package.py | tee "$LOGS/package-hnmod.log"
cp "target/occultism-$VERSION.hnmod" occultism.hnmod

echo "== artifacts =="
for f in content/occultism.hmod occultism.hnmod; do
    [ -f "$f" ] || { echo "missing $f"; exit 1; }
    size=$(stat -c %s "$f")
    sha=$(sha256sum "$f" | cut -d' ' -f1)
    entries=$(python3 -c "import zipfile,sys; print(len(zipfile.ZipFile(sys.argv[1]).namelist()))" "$f")
    echo "$f size=$size sha256=$sha entries=$entries"
done
