//! Embeds the application icon into the Windows executable. The icon is drawn by the same glyph data
//! and rasterizer the window uses (`src/ui/icons`), so there is a single source.

#[path = "src/ui/icons/app_icon.rs"]
mod app_icon;
// The build script uses one glyph of the set.
#[allow(dead_code)]
#[path = "src/ui/icons/glyphs.rs"]
mod glyphs;
#[allow(dead_code)]
#[path = "src/ui/icons/path.rs"]
mod path;
#[path = "src/ui/icons/raster.rs"]
mod raster;

use std::error::Error;
use std::path::PathBuf;

const SIZES: [u32; 6] = [16, 24, 32, 48, 64, 256];
const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const MOVEABLE_PURE_DISCARDABLE: u16 = 0x1030;
const LANG_EN_US: u16 = 0x0409;

/// `RESOURCEHEADER` with ordinal type and name: always 32 bytes.
fn header(data: usize, kind: u16, id: u16, flags: u16) -> Vec<u8> {
    let mut h = Vec::with_capacity(32);
    h.extend_from_slice(&(data as u32).to_le_bytes());
    h.extend_from_slice(&32u32.to_le_bytes());
    for value in [0xFFFF, kind, 0xFFFF, id] {
        h.extend_from_slice(&value.to_le_bytes());
    }
    h.extend_from_slice(&0u32.to_le_bytes());
    h.extend_from_slice(&flags.to_le_bytes());
    h.extend_from_slice(&LANG_EN_US.to_le_bytes());
    h.extend_from_slice(&[0; 8]);
    h
}

/// 32-bit DIB: `BITMAPINFOHEADER` (double height), bottom-up BGRA, an empty AND mask.
fn dib(size: u32) -> Vec<u8> {
    let rgba = app_icon::rgba(size);
    let n = size as usize;
    let mask_row = n.div_ceil(32) * 4;
    let mut d = Vec::with_capacity(40 + n * n * 4 + mask_row * n);
    for value in [40u32, size, size * 2] {
        d.extend_from_slice(&value.to_le_bytes());
    }
    d.extend_from_slice(&1u16.to_le_bytes());
    d.extend_from_slice(&32u16.to_le_bytes());
    d.extend_from_slice(&[0; 24]);
    for y in (0..n).rev() {
        for p in rgba[y * n * 4..(y + 1) * n * 4].as_chunks::<4>().0 {
            d.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
        }
    }
    d.resize(d.len() + mask_row * n, 0);
    d
}

fn push(res: &mut Vec<u8>, kind: u16, id: u16, data: &[u8]) {
    res.extend(header(data.len(), kind, id, MOVEABLE_PURE_DISCARDABLE));
    res.extend_from_slice(data);
    res.resize(res.len().next_multiple_of(4), 0);
}

fn resource() -> Vec<u8> {
    // A .res file starts with an empty entry.
    let mut res = header(0, 0, 0, 0);
    let mut group = vec![0, 0, 1, 0];
    group.extend_from_slice(&(SIZES.len() as u16).to_le_bytes());
    for (i, size) in SIZES.into_iter().enumerate() {
        let id = i as u16 + 1;
        let image = dib(size);
        // 256 is written as 0 in the directory.
        let side = if size >= 256 { 0 } else { size as u8 };
        group.extend_from_slice(&[side, side, 0, 0]);
        group.extend_from_slice(&1u16.to_le_bytes());
        group.extend_from_slice(&32u16.to_le_bytes());
        group.extend_from_slice(&(image.len() as u32).to_le_bytes());
        group.extend_from_slice(&id.to_le_bytes());
        push(&mut res, RT_ICON, id, &image);
    }
    push(&mut res, RT_GROUP_ICON, 1, &group);
    res
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    for file in ["app_icon.rs", "glyphs.rs", "path.rs", "raster.rs"] {
        println!("cargo:rerun-if-changed=src/ui/icons/{file}");
    }
    // The MSVC linker takes a .res file as an input; other toolchains would need windres.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return Ok(());
    }
    let out = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?);
    let res = out.join("app-icon.res");
    std::fs::write(&res, resource())?;
    println!("cargo:rustc-link-arg-bins={}", res.display());
    Ok(())
}
