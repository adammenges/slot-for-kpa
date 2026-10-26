#!/usr/bin/env bash
# Reproducible libretro Snes9x build for Android arm64 (SNES). No BIOS/ROMs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NDK="${ANDROID_NDK_HOME:-${ANDROID_SDK_ROOT:?Set ANDROID_NDK_HOME or ANDROID_SDK_ROOT}/ndk/27.2.12479018}"
[ -x "$NDK/ndk-build" ] || { echo "Android NDK ndk-build not found in $NDK" >&2; exit 1; }
LLVM="$(echo "$NDK"/toolchains/llvm/prebuilt/*/bin)"
# Pinned upstream libretro snes9x-libretro commit (2026-10-26).
PIN="$(cat "$ROOT/cores/snes9x/revision")"
OUT="$ROOT/.build/snes9x"
SRC="$OUT/source"
mkdir -p "$OUT"
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
# Upstream's own Android recipe (libretro/jni/Android.mk): RGB565 video, static libc++,
# 16 KiB pages. Only the KONKR ABI is built.
"$NDK/ndk-build" -C "$SRC/libretro/jni" -j4 \
  APP_ABI=arm64-v8a APP_PLATFORM=android-31 APP_SUPPORT_FLEXIBLE_PAGE_SIZES=true \
  NDK_PROJECT_PATH="$SRC/libretro" \
  NDK_OUT="$OUT/obj" NDK_LIBS_OUT="$OUT/libs"
CORE="$OUT/libs/arm64-v8a/libretro.so"
[ -s "$CORE" ] || { echo "Missing Snes9x libretro library" >&2; exit 1; }
SYMBOLS="$("$LLVM/llvm-nm" -D --defined-only "$CORE")"
for symbol in retro_run retro_serialize retro_get_memory_data; do
  grep -q " $symbol$" <<< "$SYMBOLS" || {
    echo "Missing libretro symbol in Snes9x: $symbol" >&2
    exit 1
  }
done
if "$LLVM/llvm-readelf" -d "$CORE" | grep -F 'libc++_shared.so'; then
  echo "Snes9x must link libc++ statically" >&2
  exit 1
fi
DEST="$ROOT/android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$DEST"
cp "$CORE" "$DEST/libsnes9x_libretro.so"
mkdir -p "$ROOT/android/app/src/main/assets/licenses"
cp "$SRC/LICENSE" "$ROOT/android/app/src/main/assets/licenses/snes9x-LICENSE.txt"
cat > "$ROOT/android/app/src/main/assets/core-build-snes9x.json" <<JSON
{"core":"snes9x","revision":"$PIN","patches":["achievement-memory.patch"]}
JSON
echo "Snes9x arm64 core: $DEST/libsnes9x_libretro.so"
