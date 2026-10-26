#!/usr/bin/env bash
# Reproducible local entry point; uses a normal SDK/JDK, never .build/local-env.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
COMMAND="${1:-help}"
if [ "$#" -gt 0 ]; then shift; fi
if [ "$COMMAND" = help ] || [ "$COMMAND" = --help ]; then
  cat <<'HELP'
Usage: bash scripts/dev-android.sh <command> [arguments]
  doctor                 Check SDK 35, NDK r27c, JDK 17 and build tools
  studio                 Build and package the pinned upstream Cart Studio
  build                  Build Studio, native cores, and the debug APK
  run [adb-serial]        Install and launch the debug APK on an ARM64 device
  emulator <avd> [args]   Start an existing ARM64 Android Virtual Device
  smoke [args]           Run tools/emulation-lab/snes_smoke.py (Pillow required)

Prerequisites and fresh-clone instructions: docs/android-development.md
HELP
  exit 0
fi
if [ -z "${ANDROID_SDK_ROOT:-}" ]; then
  if [ -n "${ANDROID_HOME:-}" ]; then export ANDROID_SDK_ROOT="$ANDROID_HOME"
  elif [ "$(uname -s)" = Darwin ]; then export ANDROID_SDK_ROOT="$HOME/Library/Android/sdk"
  else export ANDROID_SDK_ROOT="$HOME/Android/Sdk"; fi
fi
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_SDK_ROOT/ndk/27.2.12479018}"
export PATH="$ANDROID_SDK_ROOT/platform-tools:$PATH"
if [ -n "${JAVA_HOME:-}" ]; then export PATH="$JAVA_HOME/bin:$PATH"; fi
fail() { echo "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null || fail "Missing $1. See docs/android-development.md"; }
use_rust() {
  need rustup
  local cargo_path
  cargo_path="$(rustup which --toolchain 1.96 cargo)" || fail 'Install Rust with: rustup toolchain install 1.96'
  export PATH="$(dirname "$cargo_path"):$PATH"
  export RUSTUP_TOOLCHAIN=1.96
}
doctor() {
  for tool in git python3 node java gradle cmake make; do need "$tool"; done
  use_rust
  java -version 2>&1 | head -n 1 | grep -Eq 'version "17\.' || fail 'Select JDK 17 using JAVA_HOME.'
  gradle --version | grep -q '^Gradle 8.7$' || fail 'Use Gradle 8.7, matching CI.'
  test -s "$ANDROID_SDK_ROOT/platforms/android-35/android.jar" || fail 'Install platforms;android-35 in the Android SDK.'
  test -d "$ANDROID_SDK_ROOT/build-tools/35.0.0" || fail 'Install build-tools;35.0.0 in the Android SDK.'
  test -s "$ANDROID_NDK_HOME/source.properties" || fail 'Install ndk;27.2.12479018 in the Android SDK.'
  grep -q '27.2.12479018' "$ANDROID_NDK_HOME/source.properties" || fail 'Use NDK 27.2.12479018.'
  need wasm-pack
  echo 'Android build prerequisites are ready.'
}
studio() {
  use_rust; need git; need python3; need node; need wasm-pack
  local stage="$ROOT/.build/studio-upstream" pin
  pin="$(tr -d '\r\n' < third_party/slot-cart-studio/slot.ref)"
  mkdir -p "$stage"
  if [ ! -d "$stage/slot/.git" ]; then
    git init -q "$stage/slot"
    git -C "$stage/slot" remote add origin https://github.com/BrandonKowalski/slot.git
  fi
  if [ "$(git -C "$stage/slot" rev-parse HEAD 2>/dev/null || true)" != "$pin" ]; then
    git -C "$stage/slot" fetch --depth 1 origin "$pin"
    git -C "$stage/slot" checkout --detach "$pin"
  fi
  python3 - "$stage" <<'PY'
from pathlib import Path
import shutil, sys
stage = Path(sys.argv[1])
shutil.copytree('third_party/slot-cart-studio',stage/'slot-cart-studio',dirs_exist_ok=True)
manifest=stage/'slot-cart-studio/Cargo.toml'
manifest.write_text(manifest.read_text()+'\n[workspace]\n')
shutil.copy2('android/studio/Cargo.lock',stage/'slot-cart-studio/Cargo.lock')
PY
  rustup target add wasm32-unknown-unknown --toolchain 1.96
  node --test third_party/slot-cart-studio/web/*.test.mjs android/studio/*.test.mjs
  (cd "$stage/slot-cart-studio" && wasm-pack build --release --target web --no-typescript --out-dir web/pkg -- --locked)
  python3 scripts/package-konkr-cart-studio.py --built "$stage/slot-cart-studio/web"
}
case "$COMMAND" in
  doctor) doctor ;;
  studio) studio ;;
  build) doctor; studio; bash scripts/build-konkr-android.sh ;;
  emulator)
    [ "$#" -ge 1 ] || fail 'Specify an ARM64 AVD name.'
    avd="$1"; shift
    exec "$ANDROID_SDK_ROOT/emulator/emulator" -avd "$avd" -gpu auto -no-snapshot "$@"
    ;;
  run)
    need adb
    adb_args=(); if [ "$#" -gt 0 ]; then adb_args=(-s "$1"); fi
    [ "$(adb "${adb_args[@]}" shell getprop ro.product.cpu.abi | tr -d '\r')" = arm64-v8a ] || fail 'This APK needs an ARM64 device or AVD.'
    apk=android/app/build/outputs/apk/debug/app-debug.apk
    test -s "$apk" || fail 'Build the APK first: bash scripts/dev-android.sh build'
    adb "${adb_args[@]}" install -r "$apk"
    adb "${adb_args[@]}" shell am start -W -n fyi.slot.konkr.diag/fyi.slot.konkr.MainActivity
    ;;
  smoke) need python3; exec python3 tools/emulation-lab/snes_smoke.py "$@" ;;
  *) fail "Unknown command: $COMMAND (use --help)" ;;
esac
