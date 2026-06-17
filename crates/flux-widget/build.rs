//! Embeds the Flux logo as the Windows executable icon (shown in Explorer, the
//! taskbar, and Alt-Tab). Rust binaries carry no icon resource by default, so
//! without this the exe shows a blank/default icon.
//!
//! Best-effort: if the icon can't be generated or the resource compiler isn't
//! available, the build still succeeds — the exe just has no embedded icon.

fn main() {
    #[cfg(windows)]
    embed_windows_icon();
}

#[cfg(windows)]
fn embed_windows_icon() {
    println!("cargo:rerun-if-changed=assets/icon.png");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let ico_path = std::path::Path::new(&out_dir).join("Flux.ico");

    if let Err(e) = generate_ico("assets/icon.png", &ico_path) {
        println!("cargo:warning=could not build Flux.ico ({e}); exe will have no icon");
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(&ico_path.to_string_lossy());
    // Version-info so Windows (Explorer details, and especially the UAC elevation
    // dialog when the widget self-elevates to set up the CPU-temp service) shows
    // a proper "Flux" name + icon instead of a bare "flux.exe / Unknown publisher",
    // which reads as sketchy. (Still unsigned, but clearly identified as Flux.)
    let ver = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    res.set("ProductName", "Flux")
        .set("FileDescription", "Flux")
        .set("CompanyName", "DruidFluids")
        .set("LegalCopyright", "Copyright (c) 2026 Matt Hakes")
        .set("OriginalFilename", "flux.exe")
        .set("InternalName", "flux")
        .set("FileVersion", &ver)
        .set("ProductVersion", &ver);
    let mut parts = ver.split('.').map(|s| s.parse::<u64>().unwrap_or(0));
    let (maj, min, pat) = (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    );
    let packed = (maj << 48) | (min << 32) | (pat << 16);
    res.set_version_info(winresource::VersionInfo::FILEVERSION, packed);
    res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed);
    if let Err(e) = res.compile() {
        println!("cargo:warning=icon/version resource compile failed ({e}); exe will lack metadata");
    }
}

/// Build a multi-size `.ico` from the 256×256 source PNG.
#[cfg(windows)]
fn generate_ico(
    png: &str,
    ico_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = image::open(png)?;
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16u32, 32, 48, 64, 128, 256] {
        let rgba = src
            .resize_exact(size, size, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        let img = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
        dir.add_entry(ico::IconDirEntry::encode(&img)?);
    }
    dir.write(std::fs::File::create(ico_path)?)?;
    Ok(())
}
