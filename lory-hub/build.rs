use std::env;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let icon_ico = Path::new(&manifest_dir).join("app.ico");
    let icon_png = Path::new(&manifest_dir).join("app.png");
    let icon_icns = Path::new(&manifest_dir).join("app.icns");

    // Gera .png a partir do .ico em todas as plataformas
    if icon_ico.exists() && !icon_png.exists() {
        if let Ok(img) = image::open(&icon_ico) {
            let _ = img.to_rgba8().save(&icon_png);
            println!("cargo:warning=Gerado app.png a partir do app.ico");
        }
    }

    // Windows: embute .ico no .exe
    #[cfg(target_os = "windows")]
    {
        if icon_ico.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("app.ico");
            res.compile().unwrap();
        }
    }

    // macOS: aviso sobre .icns
    if target_os == "macos" && !icon_icns.exists() {
        println!("cargo:warning=app.icns não encontrado. Gere com: iconutil -c icns app.iconset");
    }

    slint_build::compile("ui/app.slint").unwrap();

    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=app.png");
    println!("cargo:rerun-if-changed=build.rs");
}
