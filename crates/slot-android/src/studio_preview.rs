//! Studio uses the same SNES mould, tint and title-label renderer as the shelf.
use slot_store::{Cart, Platform, ShellChoice};

pub(crate) fn snes_png(stem: &str, shell: &str) -> Option<Vec<u8>> {
    if stem.len() > 4096 || shell.len() > 100 { return None; }
    let choice = if shell == "auto" { None } else { Some(ShellChoice::parse(shell)?) };
    let cart = Cart { platform:Platform::Snes, stem:stem.into(), title:stem.into(),
        code:String::new(), rom:Default::default(), label:None, shell:choice };
    let face = slot_ui::density::with_scale(2.0, || slot_ui::cart_face_with(&cart,None));
    let mut bytes=Vec::new();
    let mut encoder=png::Encoder::new(&mut bytes,720,480);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer=encoder.write_header().ok()?;
    writer.write_image_data(&face.rgba).ok()?;
    writer.finish().ok()?;
    Some(bytes)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_fyi_slot_konkr_SnesCartRenderer_render(
    mut env:jni::JNIEnv<'_>, _this:jni::objects::JObject<'_>,
    stem:jni::objects::JString<'_>, shell:jni::objects::JString<'_>,
) -> jni::sys::jbyteArray {
    let Ok(stem)=env.get_string(&stem).map(|s|s.to_string_lossy().into_owned()) else {return std::ptr::null_mut()};
    let Ok(shell)=env.get_string(&shell).map(|s|s.to_string_lossy().into_owned()) else {return std::ptr::null_mut()};
    let Some(bytes)=snes_png(&stem,&shell) else {return std::ptr::null_mut()};
    env.byte_array_from_slice(&bytes).map(|a|a.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_decodes_to_the_native_dense_shelf_face() {
        let bytes=snes_png("A very long SNES title (USA)","auto 6655aa solid").unwrap();
        let mut reader=png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut rgba=vec![0;reader.output_buffer_size()];
        let info=reader.next_frame(&mut rgba).unwrap();
        assert_eq!((info.width,info.height),(720,480));
        let cart=Cart { platform:Platform::Snes,stem:"A very long SNES title (USA)".into(),
            title:String::new(),code:String::new(),rom:Default::default(),label:None,
            shell:ShellChoice::parse("auto 6655aa solid") };
        let face=slot_ui::density::with_scale(2.0,||slot_ui::cart_face_with(&cart,None));
        assert_eq!(rgba,face.rgba);
        assert_eq!(rgba[3],0);
        assert!(snes_png("Title","invalid").is_none());
    }
}
