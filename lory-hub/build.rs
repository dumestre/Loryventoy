use std::env;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let icon_ico = Path::new(&manifest_dir).join("app.ico");
    let icon_png = Path::new(&manifest_dir).join("app.png");

    #[cfg(target_os = "windows")]
    {
        if icon_ico.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("app.ico");
            res.compile().unwrap();
        }
    }

    if icon_ico.exists() && !icon_png.exists() {
        if let Ok(img) = image::open(&icon_ico) {
            let _ = img.save(&icon_png);
        }
    }

    slint_build::compile("ui/app.slint").unwrap();

    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=build.rs");
}