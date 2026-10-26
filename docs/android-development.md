# Build and test Android locally

These steps work from a fresh clone. They do not require the original author's
ignored `.build/local-env.sh`, SDK installation, or prepared emulator. Build on
macOS or Linux; run on an ARM64 Android device, or an ARM64 AVD on an Apple
Silicon Mac. The APK contains ARM64 native libraries only.

## Prerequisites

Install Git, Python 3, Node.js 18 or newer, JDK 17, Gradle 8.7, CMake and make.
CMake 3.x is the configuration used for the local core builds. Select JDK 17
through `JAVA_HOME` and put Gradle and the other tools on `PATH`.

Install Rust and the Studio build tool:

```sh
rustup toolchain install 1.96
rustup target add aarch64-linux-android wasm32-unknown-unknown --toolchain 1.96
cargo +1.96 install wasm-pack --version 0.15.0 --locked
```

Install Android Studio or the Android command-line tools. Set the SDK location
for your machine; the usual macOS path is:

```sh
export ANDROID_SDK_ROOT="$HOME/Library/Android/sdk"
export PATH="$ANDROID_SDK_ROOT/platform-tools:$PATH"
```

On Linux, the usual SDK path is `$HOME/Android/Sdk`. Install the pinned SDK
components through Android Studio's SDK Manager, or:

```sh
"$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager" \
  "platform-tools" "platforms;android-35" "build-tools;35.0.0" \
  "ndk;27.2.12479018"
```

Complete any SDK license prompts, then check the setup from the repository root:

```sh
bash scripts/dev-android.sh doctor
```

## Build

```sh
bash scripts/dev-android.sh build
```

This builds the pinned upstream Studio with the committed
[Studio dependency lockfile](../android/studio/Cargo.lock), packages the Android
adapters, builds any missing emulator cores, compiles the Rust Android library,
and assembles the debug APK. The first build downloads upstream sources and
compiler dependencies; later builds reuse `.build/` and Cargo caches. No ROMs or
BIOS files are downloaded.

Output: `android/app/build/outputs/apk/debug/app-debug.apk`.

The debug package is `fyi.slot.konkr.diag`. It installs alongside the ordinary
app, uses separate app-private storage, and disables external shared save
folders. To rebuild only the Studio assets:

```sh
bash scripts/dev-android.sh studio
```

For native-only iteration after Studio and the cores have been built:

```sh
bash scripts/build-konkr-android.sh
```

## Run

Connect an ARM64 device with USB debugging enabled, or start an ARM64 AVD. On
Apple Silicon, create one in Android Studio's Device Manager using an Android
35 ARM64 system image and enable hardware keyboard input. Alternatively use
`avdmanager` after installing `emulator` and the
`system-images;android-35;google_apis;arm64-v8a` SDK package.

```sh
bash scripts/dev-android.sh emulator SlotSnesTest -port 5556
```

In another terminal, set the test display size and install the app:

```sh
adb -s emulator-5556 shell wm size 960x640
adb -s emulator-5556 shell wm density 160
bash scripts/dev-android.sh run emulator-5556
```

For a physical device, substitute its serial from `adb devices`. The `run`
command checks the target ABI before installing. See the
[README keyboard controls](../README.md#mac-keyboard-controls) for emulator
input, or use a connected controller.

## Automated SNES smoke test

Install Pillow in your preferred Python environment. For example:

```sh
python3 -m venv .build/smoke-python
.build/smoke-python/bin/pip install Pillow
```

Prepare the original test cartridges:

```sh
.build/smoke-python/bin/python tools/emulation-lab/snes_smoke.py \
  --serial emulator-5556 --prepare-only
```

In the debug app, select **START → Library → Choose ROM Folder → Download →
SlotSnesSmoke** and grant access. This uses the actual Android folder picker;
the test does not fabricate a storage grant. Then run:

```sh
.build/smoke-python/bin/python tools/emulation-lab/snes_smoke.py \
  --serial emulator-5556
```

The test refreshes the library, selects each generated cartridge, verifies the
red gameplay frame, saves/ejects, checks SRAM and state files, and resumes with
a successful Snes9x state-restoration log. It covers NTSC, PAL, high-resolution,
headered SMC and ZIP launches. It relaunches only the isolated debug app and
leaves the probe cartridges and saves available for inspection.

Use `--rom-dir /sdcard/Download/SNES` if that is your already-granted test folder.
The script refuses to overwrite a different file under one of its fixture
names. Results, screenshots and logs go to `.build/snes-smoke/` by default;
`--output` selects another location. The tests require Auto Save on Eject to be
on and are intended for the diagnostic app, not an active gameplay session.

This checks frontend integration; it does not establish commercial-game
compatibility or performance on physical KONKR hardware.

## Regression checks

```sh
node --test android/studio/*.test.mjs
python3 -m unittest discover -s tools/emulation-lab -p 'test_*.py' -v
bash scripts/test-snes9x.sh
cargo test --locked -p slot-android --features runtime-tests --lib
cargo test --locked -p slot-ui --test cart_material --test mixed_shelf_parity --test native_density
gradle --no-daemon -p android :app:testDebugUnitTest :app:testReleaseUnitTest
```

The [SNES support notes](snes-support.md) record broader coverage and baseline
UI test failures. The [roadmap](snes-roadmap.md) distinguishes completed work
from remaining generated artwork and hardware validation.
