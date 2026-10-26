#!/usr/bin/env bash
# Exercise the pinned production core with our original homebrew probe.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIN="$(cat "$ROOT/cores/snes9x/revision")"
SRC="$ROOT/.build/snes9x-host/source"
mkdir -p "$(dirname "$SRC")"
if [ ! -d "$SRC/.git" ]; then
  git init -q "$SRC"
  git -C "$SRC" remote add origin https://github.com/libretro/snes9x.git
fi
if [ "$(git -C "$SRC" rev-parse HEAD 2>/dev/null || true)" != "$PIN" ]; then
  git -C "$SRC" fetch --depth=1 origin "$PIN"
  git -C "$SRC" checkout --detach -q FETCH_HEAD
fi
PATCH="$ROOT/cores/snes9x/achievement-memory.patch"
if git -C "$SRC" apply --check "$PATCH" 2>/dev/null; then
  git -C "$SRC" apply "$PATCH"
else
  git -C "$SRC" apply --reverse --check "$PATCH"
fi
case "$(uname -s)" in
  Darwin) PLATFORM=osx; EXT=dylib ;;
  Linux) PLATFORM=unix; EXT=so ;;
  *) echo 'Host probe supports macOS and Linux.' >&2; exit 1 ;;
esac
make -C "$SRC/libretro" -j4 platform="$PLATFORM" ARCH="$(uname -m)"
export SLOT_SNES9X_CORE="$SRC/libretro/snes9x_libretro.$EXT"
cd "$ROOT"
for region in ntsc pal hires; do
  unset SLOT_SNES_HIGH_RES
  export SLOT_SNES_PROBE="$ROOT/.build/snes9x-host/probe-$region.sfc"
  if [ "$region" = pal ]; then
    python3 tools/emulation-lab/make_snes_probe.py "$SLOT_SNES_PROBE" --pal
  elif [ "$region" = hires ]; then
    export SLOT_SNES_HIGH_RES=1
    python3 tools/emulation-lab/make_snes_probe.py "$SLOT_SNES_PROBE" --hires
  else
    python3 tools/emulation-lab/make_snes_probe.py "$SLOT_SNES_PROBE"
  fi
  cargo test --locked -p slot-retro --test snes9x -- --ignored --nocapture
done
