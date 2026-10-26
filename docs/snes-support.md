# SNES support

The Android frontend now discovers and launches Super Nintendo cartridges through
Snes9x. The core revision is pinned in `cores/snes9x/revision`; Android packages
its license and revision metadata with the library.

See the [SNES roadmap](snes-roadmap.md) for remaining artwork, UI, validation and
release work. This document records implemented behavior and test evidence.

## Supported flow

- `.sfc`, `.smc` (including 512-byte copier headers), and one-game ZIP archives.
- A SNES shelf, cartridge artwork, favorites, recents, and custom PNG labels in
  Cart Studio. The official automatic art catalog still covers GB/GBC/GBA only.
- A/B/X/Y, D-pad, Start, Select, L/R on the first controller. X/Y bypass the
  Game Boy turbo mapping; L/R are game buttons.
- Dynamic video sizes, including 256×224, extended-height and interlaced 512×478
  output. The default 4:3 view preserves the entire active picture; Integer and
  Stretch are available in the paused game's Screen settings.
- SRAM and state import/export through the existing Snes9x save folders,
  automatic suspend/resume, rewind, fast-forward, run-ahead, and screenshots at
  the actual frame dimensions.
- RetroAchievements console 3, copier-header-normalized ROM hashes, WRAM/SRAM
  snapshots, and optional SA-1 internal RAM. The small core patch exposes SA-1
  memory to the frontend without changing state formats.

One controller is supported. Slot Link multiplayer, SNES mouse/light guns, and
MSU-1 sidecar files are outside this implementation. No BIOS or commercial ROM
is included. Enhanced-chip game compatibility, audible audio quality, and
performance on a physical KONKR still require representative game testing.

## Reproduce the tests

Use Rust 1.96. The host core test requires Git, Python 3, make and a C++ compiler
on macOS or Linux:

```sh
bash scripts/test-snes9x.sh
# or: task test:snes
```

This builds the pinned, patched core and generates original homebrew cartridges.
NTSC, PAL, and high-resolution/interlaced runs assert video dimensions and the
last visible pixel, PCM delivery and timing, joypad bits, achievement RAM,
SRAM import, and save-state rollback. No downloaded game is needed.

The ordinary regression commands are:

```sh
cargo test --locked -p slot-android --features runtime-tests --lib
cargo test --locked -p slot-achievements -p slot-retro -p slot-store -p slot-power
cargo test --locked -p slot-gfx --no-default-features --test konkr
cargo test --locked -p slot-ui --test cart_material --test mixed_shelf_parity --test native_density
gradle --no-daemon -p android :app:testDebugUnitTest :app:testReleaseUnitTest
```

For Android, `scripts/build-snes9x-android.sh` uses NDK 27.2.12479018 and builds
only arm64-v8a. `scripts/build-konkr-android.sh` includes the new core. Package
Cart Studio using the existing pinned WASM workflow before assembling a usable
APK. CI builds and checks the SNES library, license, metadata and editor assets;
the Checks workflow runs the generated-cartridge host tests.

## Local validation, 2026-10-26

- 297 Android runtime/session tests passed.
- The achievements, libretro, store and power regression suites passed, as did
  the KONKR viewport tests and 11 focused cartridge/density tests.
- Both Android JVM suites passed (119 debug tests, 118 release tests); the
  optimized arm64 native library and signed debug APK built successfully.
- All three real-core probe variants passed.
- Android 35 arm64 emulator at 960×640: SAF discovery, plain, ZIP and headered
  SMC launches, PAL timing, full-frame rendering, screen settings,
  run-ahead, saving/ejecting, and custom label selection/save/reload were exercised.

The broader UI suite has 12 pre-existing failures: one toast-size test, five
legacy cartridge-rendering tests, and six shelf-rendering tests. They were
reproduced from an untouched checkout of the starting revision with a separate
build directory; the SNES change does not introduce them. No physical KONKR or
RetroAchievements account was available for live hardware/service validation.
