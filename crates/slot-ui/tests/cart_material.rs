use std::path::PathBuf;
use slot_store::{Cart, Platform};
use slot_ui::{cart_face_with, cart_face_with_material, LABEL_X, LABEL_Y, LABEL_W, LABEL_H};

fn cartridge(code: &str, name: &str) -> Cart {
    Cart {
        platform: Platform::Gba,
        stem: name.into(),
        title: name.into(),
        code: code.into(),
        rom: PathBuf::new(),
        label: None,
        shell: None,
    }
}

#[test]
fn upstream_1_7_solid_shell_is_pixel_identical_in_android_carousel() {
    let cart = cartridge("AW2E", "Advance Wars 2 - Black Hole Rising (USA)");
    let official = cart_face_with(&cart, None);
    let android = cart_face_with_material(&cart, None);
    assert_eq!((official.w, official.h), (360, 212));
    assert_eq!(android.rgba, official.rgba,
        "no Android-only satin grain, gradients or recolouring on solid cartridges");
}

#[test]
fn upstream_1_7_transparent_shell_precomposes_real_board_without_changing_label() {
    // Boktai is an upstream Shell::Translucent GBA code in the real shell table.
    let cart = cartridge("U3IE", "Boktai - The Sun Is in Your Hand");
    let official_shell = cart_face_with(&cart, None);
    let android = cart_face_with_material(&cart, None);
    assert_eq!((official_shell.w, official_shell.h), (android.w, android.h));
    let changed = official_shell.rgba.chunks_exact(4).zip(android.rgba.chunks_exact(4))
        .filter(|(old, new)| old != new).count();
    assert!(changed > 100,
        "the board behind authentic translucent plastic must be visibly present");
    // The original opaque cartridge label must remain untouched.
    for y in LABEL_Y..LABEL_Y + LABEL_H {
        for x in LABEL_X..LABEL_X + LABEL_W {
            let at = ((y * official_shell.w + x) * 4) as usize;
            if official_shell.rgba[at + 3] == 255 {
                assert_eq!(&official_shell.rgba[at..at+4], &android.rgba[at..at+4]);
            }
        }
    }
    assert_eq!(android.rgba, cart_face_with_material(&cart, None).rgba,
        "no material flicker from frame-to-frame randomness");
}

#[test]
fn u1_clear_material_is_drawn_in_exact_original_slot_depth_order() {
    use slot_gfx::TexId;
    use slot_ui::{SlotChrome,Draw};
    let cart=cartridge("U3IE","Boktai - The Sun Is in Your Hand");
    let labels=[
        TexId::from_raw(7), // precomposited shelf cache, not drawn in insert
        TexId::from_raw(8), // back coloured plastic
        TexId::from_raw(9), // PCB
        TexId::from_raw(10), // translucent front and label
    ];
    let chrome=SlotChrome {
        cart:&cart,face:Some(labels[0]),rest:170.0,scale:1.0,
        seat:0.55,alert:None,dim:0.0,screen:0.0,game:false,over:true,
        slot_faces:None,
    };
    let mut out=Vec::new();
    chrome.draw_with_material(&mut out,
        [Some(labels[1]),Some(labels[2])],Some(labels[3]));
    let visible:Vec<_>=out.iter().filter_map(|draw|match draw {
        Draw::Tex{tex,..}=>Some(*tex),_=>None,
    }).collect();
    assert_eq!(visible,&labels[1..],
        "official back -> PCB -> front, never double-draw cached solid face");
    let mut fallback=Vec::new();
    chrome.draw(&mut fallback);
    assert_eq!(fallback.iter().filter_map(|draw|match draw {
        Draw::Tex{tex,..}=>Some(*tex),_=>None,
    }).collect::<Vec<_>>(),vec![labels[0]]);
}

#[test]
fn snes_cartridge_has_its_own_shape_label_and_dense_pixels() {
    let mut cart = cartridge("", "SNES Probe");
    cart.platform = Platform::Snes;
    for scale in [1.0, 4.0 / 3.0] {
        slot_ui::density::with_scale(scale, || {
            let face = cart_face_with_material(&cart, None);
            assert_eq!((face.w, face.h), (360, 240));
            assert_eq!(slot_ui::label_size(&cart), (228, 138));
            let (w, h) = slot_ui::density::texels(face.w, face.h, face.rgba.len());
            assert_eq!((w, h), ((360.0 * scale).round() as u32, (240.0 * scale).round() as u32));
            let shadow = slot_ui::snes_cart_shadow();
            assert_eq!(shadow.rgba.len(), face.rgba.len());
            assert_eq!(face.rgba[3], 0, "corners outside the shell stay transparent");
            assert!(face.rgba.chunks_exact(4).any(|p| p[3] == 255));
        });
    }
}

#[test]
fn snes_solid_shell_color_preserves_labels_and_silhouette_at_each_density() {
    let mut cart=cartridge("", "Custom SNES (USA)");
    cart.platform=Platform::Snes;
    for scale in [1.0,4.0/3.0] {
        slot_ui::density::with_scale(scale,|| {
            cart.shell=None;
            let neutral=cart_face_with_material(&cart,None);
            cart.shell=slot_store::ShellChoice::parse("auto 365d9d solid");
            let colored=cart_face_with_material(&cart,None);
            let width=slot_ui::density::px(360);
            let mut changed=0;
            for (i,(old,new)) in neutral.rgba.chunks_exact(4).zip(colored.rgba.chunks_exact(4)).enumerate() {
                assert_eq!(old[3],new[3]);
                let x=i as u32%width; let y=i as u32/width;
                if x>=slot_ui::density::px(66) && x<slot_ui::density::px(294)
                    && y>=slot_ui::density::px(54) && y<slot_ui::density::px(192) {
                    assert_eq!(old,new,"shell tint must not recolor labels");
                } else if old[3]>0 && old!=new { changed+=1; }
            }
            assert!(changed>1000);
        });
    }
}
