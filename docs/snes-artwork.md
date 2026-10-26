# SNES cartridge artwork

Status: artwork specification prepared; the new PNGs have not been generated or integrated yet.

The SNES shell should match the molded, textured plastic in `assets/gb_neutral.png`
and `assets/gba_neutral.png`. The existing `crates/slot-ui/assets/snes_cart.svg`
defines the current layout. Game labels remain separate images composited by the
app and Cart Studio.

## Assets

| File | Purpose | Dimensions |
| --- | --- | --- |
| `assets/snes_neutral.png` | Opaque gray plastic shell with a blank recessed label panel and a transparent exterior | 720 × 480 PNG, RGBA, representing a 360 × 240 logical canvas |
| `assets/snes_cart.png` | White silhouette with the exact same alpha as the neutral shell, derived from the final artwork | 720 × 480 PNG, RGBA |

Retain the original generated image separately when producing these assets. The
silhouette is a derivative of the approved shell, not a second generated image.
Shadows can use the same alpha with black RGB, as the current renderer does.

## Layout contract

- Front view, centered, without perspective or rotation.
- Logical canvas: 360 × 240; transparent margin around the entire cartridge.
- Label rectangle: x = 66, y = 54, width = 228, height = 138.
- At 2× resolution the label rectangle is (132, 108, 456, 276).
- Keep all ridges, embossing, bevels, and other molded details outside that label
  rectangle. The app replaces the entire rectangle with game artwork.
- The body must be fully opaque behind the label. Only the area outside the
  shell and its antialiased edge should be transparent.
- Use genuine alpha transparency, with no checkerboard, surrounding shadow,
  background color, or matte fringe baked into the image.
- Use neutral gray plastic with subtle grain, soft highlights, recessed grooves,
  and a restrained bevel treatment. Match the reference assets at their actual
  on-screen size, including the small seated-cartridge view.

## Generation prompt

Use the existing GB and GBA neutral PNGs as material/style references and a
rasterized copy of the current SNES SVG as the composition reference. The prompt
below is prepared for generation; it has not been executed.

> Create a production-ready reusable SNES cartridge shell sprite for a game
> library UI. Image 1, gb_neutral.png, and image 2, gba_neutral.png, are material
> and rendering-style references only. Image 3, the current SNES shell, is the
> layout reference. Produce one front-facing horizontal SNES cartridge, centered
> and orthographic, with no perspective or rotation. Preserve image 3's overall
> proportions and blank label placement. Match the reference PNGs' finely
> textured molded plastic, carefully shaded grooves, subtle edge highlights, and
> tactile hand-painted appearance. Use neutral light gray plastic with gently
> darker recesses. Keep the label panel completely blank and unobstructed; it
> will receive separate game artwork in the app. On a normalized 360 by 240
> canvas, the label's interior is exactly x=66, y=54, width=228, height=138.
> Keep molded details outside that rectangle. Preserve a narrow transparent
> margin around the full shell. The plastic and blank label panel are opaque.
> The exterior background must be genuinely transparent, including clean
> antialiased edges. No game art, lettering, sticker, watermark, decorative
> background, ground plane, cast shadow, extra objects, or additional views.

## Integration and verification

1. Inspect the generated shell beside both style references. Align the final
   asset with the label rectangle before exporting the two PNGs.
2. Decode and cache the neutral PNG in `crates/slot-ui/src/bitmaps.rs`; render it
   in the SNES path in `crates/slot-ui/src/cart.rs`, retaining 360 × 240 logical
   dimensions and density-aware resampling.
3. Use the same neutral PNG in `android/studio/snes.html` and include it through
   `scripts/package-konkr-cart-studio.py`. Keep the native and Studio label
   coordinates identical.
4. Revise the cartridge face cache key in
   `crates/slot-android/src/cart_disk_cache.rs` so existing installations display
   the new artwork after updating.
5. Check RGBA dimensions, transparent margins, an opaque label interior, and
   identical shell/mask alpha. Run the existing cartridge material and density
   tests after integration.
6. Inspect the actual Android shelf, seated cartridge, and Cart Studio with both
   generated title labels and a custom PNG label. Confirm the label does not
   overlap the molded frame and that edges look clean on light and dark surfaces.

An interior/PCB image is not needed for the current opaque SNES shell. Add one
only if transparent SNES shells or a corresponding exposed-board UI is added.
