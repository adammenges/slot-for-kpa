//! Real Snes9x contract test. No commercial ROM or BIOS is required.
//! Generate the probe with tools/emulation-lab/make_snes_probe.py, then set
//! SLOT_SNES9X_CORE and SLOT_SNES_PROBE and run this test with --ignored.
use slot_retro::{ButtonMask, LibretroCore, RetroCore};
use std::path::PathBuf;

#[test]
#[ignore = "requires a built Snes9x core and generated probe cartridge"]
fn real_snes_video_buttons_audio_sram_and_state_roundtrip() {
    let core_file = PathBuf::from(std::env::var("SLOT_SNES9X_CORE").expect("core path"));
    let rom = PathBuf::from(std::env::var("SLOT_SNES_PROBE").expect("probe path"));
    let temp = tempfile::tempdir().unwrap();
    let mut core = LibretroCore::open_with(&core_file, temp.path(), temp.path()).unwrap();
    core.set_option("snes9x_overscan", "auto");
    core.load(&rom).unwrap();
    let bytes = std::fs::read(&rom).unwrap();
    let expected_fps = if bytes[0x7fd9] == 2 { 50.00698 } else { 60.09881 };
    assert!((core.av_info().fps - expected_fps).abs() < 0.01);
    assert!((32000.0..32100.0).contains(&core.av_info().sample_rate));
    for _ in 0..8 { core.run_frame(ButtonMask(0)); }
    let (w, h) = if std::env::var_os("SLOT_SNES_HIGH_RES").is_some() { (512, 478) } else { (256, 224) };
    assert_eq!(core.video_size(), (w, h));
    let video = core.video_xrgb8888();
    assert_eq!(video.len(), (w * h * 4) as usize);
    assert!(video[((h * w - 1) * 4 + 2) as usize] > 200, "bottom right red pixel must survive");
    assert!(!core.take_audio().is_empty(), "core must deliver its PCM clock");
    let mut ram = vec![0; 0xa0800];
    let mut coverage = vec![0; 0x17000];
    for (button, expected) in [(ButtonMask::B, 0x8000u16), (ButtonMask::Y, 0x4000),
        (ButtonMask::A, 0x0080), (ButtonMask::X, 0x0040), (ButtonMask::L, 0x0020), (ButtonMask::R, 0x0010)] {
        for _ in 0..4 { core.run_frame(ButtonMask(button)); }
        let valid = core.achievement_memory_for_console(&mut ram, &mut coverage, 3);
        assert_eq!(valid[0], 0x20000);
        assert_eq!(valid[1], 8192);
        assert_eq!(u16::from_le_bytes([ram[0], ram[1]]), expected, "button {button}");
    }
    let saved = core.serialize().unwrap();
    let battery = core.save_ram().unwrap();
    for _ in 0..4 { core.run_frame(ButtonMask(0)); }
    assert_ne!(core.save_ram().unwrap(), battery);
    core.unserialize(&saved).unwrap();
    assert_eq!(core.save_ram().unwrap(), battery);
    let mut imported = battery.clone(); imported[7] = 0x42;
    core.load_save_ram(&imported).unwrap();
    assert_eq!(core.save_ram().unwrap(), imported);
}
