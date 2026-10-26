# SNES roadmap

Status as of 2026-10-26: SNES gameplay is implemented through Snes9x. The textured
cartridge PNGs have not been generated or integrated. This roadmap tracks the
remaining work to make SNES a first-class console in the Android/KONKR app.
Unchecked items are planned work, not supported features.

The target is a complete everyday experience: attractive cartridges, reliable
artwork matching and editing, appropriate controls and display settings, and
verified play/save/resume behavior on the actual handheld. P0 and P1 below are
the completion criteria; P2 contains optional extensions.

## Already implemented

| Area | Current implementation |
| --- | --- |
| ROMs and core | Pinned, packaged Snes9x; `.sfc`, headered `.smc`, and one-game ZIP discovery and launch |
| Library | Separate SNES shelf, favorites, recents, generated title labels, and a basic SVG shell |
| Controls | One controller with native A/B/X/Y, L/R, D-pad, Start and Select; X/Y are not consumed by Game Boy turbo mapping |
| Video | Dynamic frame dimensions, NTSC/PAL timing, high-resolution/interlaced output, and per-game Integer/4:3/Stretch modes |
| Sessions | SRAM, save states, suspend/resume, rewind, fast-forward, run-ahead, and screenshots |
| Artwork editing | Custom PNG labels through a separate SNES Cart Studio page; the X shortcut opens the selected cartridge |
| Achievements | SNES console identification, header-normalized hashes, WRAM/SRAM and optional SA-1 memory integration |
| Verification | Generated real-core probes, runtime/JVM tests, and Android emulator exercises |

These describe implemented paths, not a claim that every game or feature has
been verified on hardware. See [SNES support and validation](snes-support.md)
for test evidence and current limits.

## P0 — Finish cartridge artwork

### Files and references

All paths below are relative to the repository root.

| Output | Purpose | Format |
| --- | --- | --- |
| `assets/snes_neutral.png` | Textured gray shell with a blank label panel | 720 × 480 RGBA PNG |
| `assets/snes_cart.png` | White silhouette derived from the shell, retaining identical alpha | 720 × 480 RGBA PNG |

Use these existing images as references:

- [gb_neutral.png](../assets/gb_neutral.png): molded plastic material and texture.
- [gba_neutral.png](../assets/gba_neutral.png): shading and rendering style.
- [snes_cart.svg](../crates/slot-ui/assets/snes_cart.svg): SNES proportions and
  label placement. Rasterize a copy if the image generator cannot accept SVG.

Keep the original generated image separately from the production exports and
record the generator/model, prompt, and any subsequent editing in this document
when the artwork is delivered.

### Shell generation prompt

Save the finished result to **`assets/snes_neutral.png`**:

```text
Create a reusable SNES cartridge shell sprite for a game library UI.

Use the supplied GB and GBA PNGs as references for material, texture,
lighting, and rendering style. Use the SNES SVG as the layout reference.

Show one horizontal SNES cartridge from directly in front: orthographic,
centered, without perspective or rotation. Match the reference assets’
tactile molded plastic, fine surface grain, recessed grooves, subtle
beveled edges, and soft highlights. Use neutral light gray plastic with
slightly darker recesses.

Final canvas: 720 × 480 pixels.
Blank label interior: x=132, y=108, width=456, height=276 pixels.
Preserve this label placement exactly. Keep all molded details outside
the label rectangle. The app will overlay game artwork there.

The shell and label panel must be fully opaque. Leave a narrow genuinely
transparent margin around the cartridge, with clean antialiased edges.

No lettering, logos, game artwork, stickers, watermark, surrounding
shadow, ground plane, background color, checkerboard, or additional views.
Output a transparent RGBA PNG.
```

### Silhouette conversion instructions

Create **`assets/snes_cart.png`** from the finished neutral shell with an image
editor or script. Do not generate a second independent shell: its edges must
match exactly.

```text
Keep the exact 720 × 480 dimensions and every pixel’s alpha value.
Set every pixel’s RGB channels to white (255, 255, 255).
Do not change the silhouette or antialiased edges.
Save as an RGBA PNG.
```

### Integration checklist

- [ ] Generate and inspect the shell alongside the GB/GBA references. Correct
  the label geometry before exporting; the prompt alone does not guarantee
  exact pixel alignment.
- [ ] Export both PNGs. Check genuine transparency, clean edges, an opaque label
  interior, and identical shell/mask alpha.
- [ ] Load/cache the PNGs in [bitmaps.rs](../crates/slot-ui/src/bitmaps.rs) and
  replace the SVG rendering path in [cart.rs](../crates/slot-ui/src/cart.rs).
  Preserve the 360 × 240 logical size, with a label at (66, 54, 228, 138), and
  resample the 2× source at the actual display density. Derive the black shadow
  from the matching silhouette alpha.
- [ ] Use the same shell in [SNES Cart Studio](../android/studio/snes.html) and
  package it through [package-konkr-cart-studio.py](../scripts/package-konkr-cart-studio.py).
  Preserve Studio's 456 × 276 label export and identical crop placement.
- [ ] Invalidate old rendered faces in
  [cart_disk_cache.rs](../crates/slot-android/src/cart_disk_cache.rs) so upgrading
  the app replaces the cached SVG cartridges.
- [ ] Extend the focused cartridge/density checks for the new assets. Include
  light/dark shelf previews, the small seated cartridge, insertion/ejection,
  long generated titles, and custom labels at 1× and 4/3× display density.
- [ ] Add `assets/**` to the relevant CI path filters and verify the SNES PNG is
  packaged in the APK's Studio assets. The current checks cover the SNES HTML
  and core; artwork-only changes must also trigger verification.

Done when the actual shelf and Studio display matching, textured cartridges
without clipped labels, edge halos, stale cached art, or changes to GB/GBC/GBA.
Retain the existing SVG reference until the PNG integration is verified.

## P1 — Complete artwork discovery and Cart Studio

The automatic art path currently excludes SNES in
[AutoCartLabels.kt](../android/app/src/main/java/fyi/slot/konkr/AutoCartLabels.kt).
The full Studio exposes SNES through a separate link and a compact editor in
[snes.html](../android/studio/snes.html).

- [ ] Choose and document a SNES artwork/catalog source, its coverage, and
  attribution requirements. Keep this separate from the existing Game Boy
  catalog, which does not provide SNES art.
- [ ] Add matching for ROM content and region/revision, handling copier headers
  and ZIP-contained games consistently. Keep a generated-title fallback for
  homebrew, hacks and unmatched ROMs; never silently apply an uncertain match.
- [ ] Add automatic filling of missing SNES labels with caching, bounded
  downloads, retry/offline behavior, and preservation of user-selected labels.
- [ ] Integrate SNES into the normal Studio system navigation and browse/search
  flow. Preserve the direct X shortcut and mixed-platform libraries; verify
  physical-controller focus, save/cancel, and returning to the shelf.
- [ ] Make generated fallback labels and imported-art crops consistent between
  Studio and the native renderer. Cover tall/wide artwork, long titles, and
  changing cartridges with unsaved edits.
- [ ] Add solid shell-color editing with persisted per-game overrides while
  preserving material shading and label colors. Keep transparent shells out of
  the initial scope until their interior artwork is available.

Done when a fresh SNES library gets useful labels automatically where matching
art exists, and customizations survive save, reload, restart and offline use
through the same discoverable editing flow as the other consoles.

## P1 — Finish SNES settings and presentation

SNES already has its own shader setting, but
[settings.rs](../crates/slot-android/src/settings.rs) cycles the Game Boy shader
list. Screen size is stored per game in
[video_mode.rs](../crates/slot-android/src/video_mode.rs), defaulting to 4:3.

- [ ] Define an explicit SNES display preset list with suitable clean scaling
  choices; evaluate a lightweight CRT/scanline option on KONKR. Keep Off as the
  default and verify presets at normal, high-resolution and interlaced sizes.
- [ ] Add a system-wide SNES screen-size default while retaining per-game
  overrides and migrating existing choices without changing them.
- [ ] Audit menu text, button legends and shortcuts for native SNES X/Y and L/R.
  Keep Game Boy palettes, color correction, LCD bezels and Slot Link actions out
  of SNES-specific menus. Show the fixed Snes9x core as informational rather
  than suggesting that a second core can be selected.
- [ ] Verify NTSC/PAL pacing, transitions between frame sizes, full-picture 4:3
  rendering, integer scaling and screenshots with the final shader choices.

Done when system defaults and per-game overrides persist, controls remain
available to games, and display options work across the supported video modes.
Game Boy hardware-specific features are not requirements for SNES parity.

## P1 — Validate real games and ship a reproducible build

Generated probes cover interfaces and timing, but do not establish game
compatibility, audible sound quality or sustained performance on a KONKR.
Record hardware, APK/core revisions, game region/revision, settings, duration,
results and remaining issues in [snes-support.md](snes-support.md). Use locally
provided games; keep ROMs out of the repository.

| Required check | Evidence to collect |
| --- | --- |
| ROM and game variety | Representative LoROM/HiROM, NTSC/PAL, headered/unheadered and ZIP cases, plus SA-1, Super FX, DSP, S-DD1 and SPC7110 titles where available; document any firmware/setup requirement or unsupported case |
| Real play on KONKR | Sustained play with frame pacing, audible music/effects, audio/video synchronization, input responsiveness and thermal/performance observations |
| State and battery saves | Save/eject/relaunch, suspend/wake, process recreation, SRAM and state round trips, external save folders, and RetroArch exchange with the documented matching Snes9x version |
| Convenience features | Rewind, fast-forward and run-ahead under ordinary and demanding games; measure their performance cost and choose documented defaults |
| Achievements | Live-account identification and unlocks, header/ZIP equivalence, standard and SA-1 memory, offline caching/reconnect, and supported mode restrictions |
| Complete UI flow | Mixed shelves, favorites/recents, Studio editing, screen settings, artwork refresh after update, and no regressions to GB/GBC/GBA |

- [ ] Complete the matrix above on a physical KONKR and publish results with
  explicit unsupported cases rather than a blanket compatibility claim.
- [ ] Turn the current manual Android emulator walkthrough into repeatable
  smoke coverage using the existing original probe ROMs. Retain the real-core
  host tests and add focused regressions for defects found in real games.
- [ ] Track the 12 pre-existing broader UI test failures separately; repair them
  or maintain explicit baseline accounting so new failures cannot be hidden.
- [ ] Document a clean-clone local build/run path that does not depend on this
  checkout's ignored `.build/` tools or prepared AVD. Keep pinned core, licenses,
  artwork packaging and the APK contents checks reproducible in CI.
- [ ] Update README feature descriptions, SNES settings, limitations and real
  screenshots when the corresponding work lands. Identify the SNES-capable
  build/release explicitly; the current installation link points to upstream
  releases and should not imply they contain this fork's SNES changes.

Use the [existing regression commands](snes-support.md#reproduce-the-tests)
throughout implementation. For the artwork change, start with:

```sh
cargo test --locked -p slot-ui --test cart_material --test mixed_shelf_parity --test native_density
```

The release is ready for the first-class claim when P0/P1 work is complete,
the documented regression checks pass with baseline exceptions accounted for,
the hardware results are recorded, and the install instructions lead to the
verified build.

## P2 — Optional extensions

These are separate follow-ups, not prerequisites for the standard one-controller
SNES experience:

- Regional Super Famicom/PAL cartridge shells, matching label layouts, special
  colors, and transparent shells with interior/PCB artwork.
- Additional local controllers and multitap support, including controller
  assignment and menu/input ownership.
- SNES mouse and light-gun input, MSU-1 sidecar loading, or other peripheral and
  content workflows, each with its own compatibility and storage tests.
- SNES netplay designed around the console's multiplayer model; the existing
  Game Boy Slot Link path does not provide it.
- Optional SNES-themed framing and more advanced CRT presets after performance
  and readability are verified on the handheld.
