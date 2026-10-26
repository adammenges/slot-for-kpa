use std::path::Path;

use slot_gfx::{ScreenEffect, SRC_H, SRC_W, WHOLE_TEXTURE};
use slot_store::{ini, Platform};

use crate::settings::{Bezel, Shader};

pub const VIDEO_MODE_FILE: &str = "Config/video_mode.ini";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoMode {
    #[default]
    Actual,
    /// Non-integer: the full panel height at the platform's own aspect ratio.
    Aspect,
    Stretch,
}

impl VideoMode {
    pub fn step(self, right: bool) -> Self {
        let modes = [Self::Actual, Self::Aspect, Self::Stretch];
        let at = modes.iter().position(|mode| *mode == self).unwrap();
        modes[(at + if right { 1 } else { 2 }) % modes.len()]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            VideoMode::Actual => "actual",
            VideoMode::Aspect => "aspect",
            VideoMode::Stretch => "stretch",
        }
    }

    pub fn parse(s: &str) -> Option<VideoMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "actual" => Some(VideoMode::Actual),
            "aspect" => Some(VideoMode::Aspect),
            "stretch" => Some(VideoMode::Stretch),
            _ => None,
        }
    }

    /// L1: one size up, Actual -> Aspect -> Stretch.
    pub fn bigger(self) -> VideoMode {
        match self {
            VideoMode::Actual => VideoMode::Aspect,
            VideoMode::Aspect | VideoMode::Stretch => VideoMode::Stretch,
        }
    }

    /// R1: one size down, Stretch -> Aspect -> Actual.
    pub fn smaller(self) -> VideoMode {
        match self {
            VideoMode::Stretch => VideoMode::Aspect,
            VideoMode::Aspect | VideoMode::Actual => VideoMode::Actual,
        }
    }
}

/// The official Slot bezel's window is cut for integer scale only, the
/// Watomsk overlays' for the full-height picture (integer fits inside it);
/// nothing can frame a stretched picture.
pub fn bezel_fits(bezel: Bezel, mode: VideoMode) -> bool {
    match bezel {
        Bezel::None => true,
        Bezel::Default => mode == VideoMode::Actual,
        Bezel::Gb | Bezel::Pocket | Bezel::Color => mode != VideoMode::Stretch,
    }
}

/// Why `bezel` is hidden in `mode`, for the short Android message.
pub fn bezel_hidden_message(bezel: Bezel, mode: VideoMode) -> Option<&'static str> {
    if bezel_fits(bezel, mode) {
        return None;
    }
    Some(if mode == VideoMode::Stretch {
        "No bezel in Stretch."
    } else {
        "The slot. bezel needs Integer scale."
    })
}

/// Original Ratio is not a whole multiple of the Game Boy picture: plain
/// pixels would come out 4 or 5 screen pixels wide, so a sprite or a scroll
/// moving one pixel per frame would advance unevenly. With no shader picked,
/// draw it with RetroArch's sharp-bilinear instead, which keeps every pixel
/// the same size and the motion regular.
pub fn screen_effect(shader: Shader, platform: Platform, mode: VideoMode) -> ScreenEffect {
    if shader == Shader::Off && platform != Platform::Gba && mode == VideoMode::Aspect {
        ScreenEffect::SharpBilinear
    } else {
        shader.effect()
    }
}

#[cfg(test)]
pub fn video_mode_for(root: &Path, stem: &str) -> VideoMode {
    video_mode_for_platform(root, stem, Platform::Gba)
}

#[cfg(test)]
pub fn video_mode_for_platform(root: &Path, stem: &str, platform: Platform) -> VideoMode {
    video_mode_with_default(root, stem, platform, VideoMode::Aspect)
}

pub fn video_mode_with_default(root: &Path, stem: &str, platform: Platform, snes_default: VideoMode) -> VideoMode {
    ini::value(root, VIDEO_MODE_FILE, stem)
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or(if platform == Platform::Snes { snes_default } else { VideoMode::Actual })
}

pub fn write_video_mode(root: &Path, stem: &str, mode: VideoMode) -> std::io::Result<()> {
    ini::write(root, VIDEO_MODE_FILE, stem, mode.as_str())
}

pub fn source_rect(platform: Platform, mode: VideoMode) -> [f32; 4] {
    let (w, h) = platform.picture();
    if mode == VideoMode::Actual || (w, h) == (SRC_W, SRC_H) {
        return WHOLE_TEXTURE;
    }
    let y = SRC_H.saturating_sub(h) / 2;
    if mode == VideoMode::Aspect {
        // Widen the crop to the panel's shape so the picture keeps its ratio.
        let shown = SRC_W as f32 * h as f32 / SRC_H as f32;
        let x = (SRC_W as f32 - shown) / 2.0;
        return [
            x / SRC_W as f32,
            y as f32 / SRC_H as f32,
            shown / SRC_W as f32,
            h as f32 / SRC_H as f32,
        ];
    }
    let x = SRC_W.saturating_sub(w) / 2;
    [
        x as f32 / SRC_W as f32,
        y as f32 / SRC_H as f32,
        w as f32 / SRC_W as f32,
        h as f32 / SRC_H as f32,
    ]
}

/// UV window including black margins, so SNES pixels are never cropped.
pub fn snes_source_rect((w, h): (u32, u32), (panel_w, panel_h): (u32, u32), mode: VideoMode) -> [f32; 4] {
    if w == 0 || h == 0 || panel_w == 0 || panel_h == 0 { return WHOLE_TEXTURE; }
    let (pw, ph) = (panel_w as f32, panel_h as f32);
    let (shown_w, shown_h) = match mode {
        VideoMode::Stretch => (pw, ph),
        VideoMode::Aspect => { let sh = ph.min(pw * 3.0 / 4.0); (sh * 4.0 / 3.0, sh) },
        VideoMode::Actual => {
            let scale = (pw / w as f32).min(ph / h as f32).floor().max(1.0);
            (w as f32 * scale, h as f32 * scale)
        }
    };
    let (x, y) = ((pw / shown_w).max(1.0), (ph / shown_h).max(1.0));
    [(1.0 - x) / 2.0, (1.0 - y) / 2.0, x, y]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snes_aspect_and_integer_scaling_preserve_the_full_picture() {
        let rect = snes_source_rect((256,224), (960,640), VideoMode::Aspect);
        assert_eq!(rect, [-0.0625,0.0,1.125,1.0]);
        let integer = snes_source_rect((256,224), (960,640), VideoMode::Actual);
        assert_eq!(960.0 / integer[2] / 256.0, 2.0);
        assert_eq!(640.0 / integer[3] / 224.0, 2.0);
        assert_eq!(snes_source_rect((512,478),(960,640),VideoMode::Stretch), WHOLE_TEXTURE);
        let root = tempfile::tempdir().unwrap();
        assert_eq!(video_mode_for_platform(root.path(), "Mario", Platform::Snes), VideoMode::Aspect);
        assert_eq!(video_mode_for_platform(root.path(), "Mario", Platform::Gba), VideoMode::Actual);
    }

    #[test]
    fn shoulders_step_through_the_three_sizes() {
        assert_eq!(VideoMode::Actual.bigger(), VideoMode::Aspect);
        assert_eq!(VideoMode::Aspect.bigger(), VideoMode::Stretch);
        assert_eq!(VideoMode::Stretch.bigger(), VideoMode::Stretch);
        assert_eq!(VideoMode::Stretch.smaller(), VideoMode::Aspect);
        assert_eq!(VideoMode::Aspect.smaller(), VideoMode::Actual);
        assert_eq!(VideoMode::Actual.smaller(), VideoMode::Actual);
        assert_eq!(VideoMode::parse("aspect"), Some(VideoMode::Aspect));
    }

    #[test]
    fn aspect_keeps_the_game_boy_ratio_on_a_3_2_panel() {
        let [x, y, w, h] = source_rect(Platform::Gb, VideoMode::Aspect);
        let (tw, th) = (w * SRC_W as f32, h * SRC_H as f32);
        assert!((th - 144.0).abs() < 1e-3);
        assert!((tw / th - SRC_W as f32 / SRC_H as f32).abs() < 1e-4);
        assert!((x + w / 2.0 - 0.5).abs() < 1e-6 && (y + h / 2.0 - 0.5).abs() < 1e-6);
        // Shown on the whole panel, the 160 game columns keep 10:9.
        let game_w = 160.0 / tw * 3.0;
        assert!((game_w / 2.0 - 10.0 / 9.0).abs() < 1e-3);
    }

    #[test]
    fn original_ratio_without_shader_keeps_pixels_even() {
        assert_eq!(screen_effect(Shader::Off, Platform::Gb, VideoMode::Aspect),
            ScreenEffect::SharpBilinear);
        assert_eq!(screen_effect(Shader::Off, Platform::Gbc, VideoMode::Aspect),
            ScreenEffect::SharpBilinear);
        assert_eq!(screen_effect(Shader::Off, Platform::Gb, VideoMode::Actual),
            ScreenEffect::None);
        assert_eq!(screen_effect(Shader::Grid, Platform::Gb, VideoMode::Aspect),
            ScreenEffect::Grid);
        assert_eq!(screen_effect(Shader::Off, Platform::Gba, VideoMode::Aspect),
            ScreenEffect::None);
    }

    #[test]
    fn each_bezel_fits_only_its_sizes() {
        use VideoMode::*;
        for mode in [Actual, Aspect, Stretch] {
            assert!(bezel_fits(Bezel::None, mode));
        }
        assert!(bezel_fits(Bezel::Default, Actual));
        assert!(!bezel_fits(Bezel::Default, Aspect));
        for watomsk in [Bezel::Gb, Bezel::Pocket, Bezel::Color] {
            assert!(bezel_fits(watomsk, Actual));
            assert!(bezel_fits(watomsk, Aspect));
            assert!(!bezel_fits(watomsk, Stretch));
            assert_eq!(bezel_hidden_message(watomsk, Stretch), Some("No bezel in Stretch."));
        }
        assert_eq!(bezel_hidden_message(Bezel::Default, Stretch), Some("No bezel in Stretch."));
        assert!(bezel_hidden_message(Bezel::Default, Aspect).unwrap().contains("Integer"));
        assert_eq!(bezel_hidden_message(Bezel::Gb, Aspect), None);
    }
}
