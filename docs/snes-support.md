# SNES support

The Android frontend now discovers and launches Super Nintendo cartridges through
Snes9x. The core revision is pinned in `cores/snes9x/revision`; Android packages
its license and revision metadata with the library.

See the [SNES roadmap](snes-roadmap.md) for remaining artwork, UI, validation and
release work. This document records implemented behavior and test evidence.

## Supported flow

- `.sfc`, `.smc` (including 512-byte copier headers), and one-game ZIP archives.
- A SNES shelf, beveled SVG cartridges, favorites, recents, automatic cover
  artwork, and custom PNG labels. Cart Studio offers shell colors, title search,
  independent drafts, and the same native preview renderer as the shelf.
- A/B/X/Y, D-pad, Start, Select, L/R on the first controller. X/Y bypass the
  Game Boy turbo mapping; L/R are game buttons.
- Dynamic video sizes, including 256×224, extended-height and interlaced 512×478
  output. The default 4:3 view preserves the entire active picture; Integer and
  Stretch are available in the paused game's Screen settings. System settings
  offer a default size plus Off, Sharp Bilinear and Scanlines presets; explicit
  per-game choices take precedence.
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

## Automatic artwork

SNES uses the [Libretro No-Intro database](https://github.com/libretro/libretro-database/blob/master/metadat/no-intro/Nintendo%20-%20Super%20Nintendo%20Entertainment%20System.dat)
for exact CRC32 and uncompressed-size identity, and
[Libretro SNES box art](https://github.com/libretro-thumbnails/Nintendo_-_Super_Nintendo_Entertainment_System/tree/master/Named_Boxarts)
for the image. The [thumbnail project](https://github.com/libretro-thumbnails/libretro-thumbnails)
provides the collection and naming rules. These adapted covers are distinct
from the original Slot catalog's physical Game Boy cartridge-label scans.

Copier headers are excluded from the fingerprint, including inside ZIP files.
Renamed dumps therefore still match; unknown hacks/homebrew and ambiguous
matches retain generated title labels. No title-only guesses are made. Covers
are fitted into the entire 456 × 276 label without cropping off vertical box
art. Missing images retain the existing fallback.

The existing foreground artwork worker handles at most four SNES attempts and
two new labels per batch, pauses during gameplay/Studio/background use, and
uses bounded retries. The DAT is cached for seven days with a stale offline
fallback; downloaded source images have a 64 MiB cache limit. Saved labels
remain available offline. User-selected labels take priority and automatic
writes do not replace them.

Each automatic label has a neighboring `.png.source.json` file with provider,
source URL, matched title, checksum and ROM size. Saving a custom replacement
removes that attribution record. Artwork is downloaded on demand; no commercial
game artwork or ROM is bundled in this repository.

## Local development and Android smoke test

See [Build and test Android locally](android-development.md) for a fresh-clone
build, emulator setup, and the automated five-format Android SNES smoke test.
The tool uses original generated cartridges and the actual folder-picker grant,
and checks launch, SRAM/state writes and successful state restoration.

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

## Follow-up validation: Studio, display and development workflow

- 300 Rust Android runtime/session tests passed, including settings migration,
  preservation of per-game screen overrides, and native/Studio pixel parity.
- 12 focused cartridge/density tests passed, including shell colors that retain
  the original label pixels and silhouette alpha at both tested densities.
- Android JVM suites passed with 122 debug and 121 release tests, including
  header/ZIP-normalized artwork identity, ambiguous-match rejection and URL
  naming rules. Two Studio draft/crop tests passed under Node.
- The local helper built Studio from a fresh staging checkout with the committed
  dependency lockfile, then built the ARM64 library and debug APK.
- Android 35 ARM64 emulator: all five generated NTSC, PAL, high-resolution,
  headered SMC and ZIP smoke cases launched, wrote 8192-byte SRAM and save states,
  and reported successful state restoration when resumed.
- Manual emulator checks verified a saved shell color on the shelf, native
  preview parity, and an unchanged draft preview after switching GB/SNES tabs.
  D-pad/A events move focus and select colors. Leaving with unsaved edits
  prompts to keep editing or discard them; B dismisses the prompt and retains
  the draft.
- Scanlines visibly modulated the rendered probe's rows, the system Stretch
  default filled the panel, and both settings persisted across app restarts.
  The three real-core probes, two viewport checks and 29 emulation-lab Python
  tests also passed.
- A live Libretro image download was exercised using an original probe ROM and
  a temporary synthetic DAT identity. Android decoded/fitted the cover, wrote
  its source metadata, and preserved a custom replacement after restart. The
  original DAT was restored and the fixture removed afterward. This validates
  the acquisition path; it is not evidence of commercial-game compatibility.

Generated PNG shell artwork, real-game/hardware performance, and live-account
achievement validation remain outstanding as recorded in the roadmap.
