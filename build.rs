use std::env;
use std::path::Path;

#[cfg(target_os = "windows")]
fn build_windows_resources(icon_ico: &Path) {
    if icon_ico.exists() {
        let mut res = winres::WindowsResource::new();
        if let Some(icon_str) = icon_ico.to_str() {
            res.set_icon(icon_str);
        }
        if let Err(e) = res.compile() {
            println!("cargo:warning=Erro ao compilar recurso Windows: {}", e);
        }
    }
}

fn convert_ico_to_png(icon_ico: &Path, icon_png: &Path) {
    if icon_ico.exists() && !icon_png.exists() {
        if let Ok(img) = image::open(icon_ico) {
            let _ = img.to_rgba8().save(icon_png);
            println!(
                "cargo:warning=Gerado app.png ({}x{}) a partir do app.ico",
                img.width(),
                img.height()
            );
        }
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let icon_ico = Path::new(&manifest_dir).join("app.ico");
    let icon_png = Path::new(&manifest_dir).join("app.png");
    let icon_icns = Path::new(&manifest_dir).join("app.icns");

    // Converte .ico → .png em todas as plataformas (Linux/macOS usam PNG,
    // Windows usa .ico embedding mas o .png também é gerado para compatibilidade)
    convert_ico_to_png(&icon_ico, &icon_png);

    // Windows: embute .ico no .exe via winres
    #[cfg(target_os = "windows")]
    build_windows_resources(&icon_ico);

    // Avisos específicos por plataforma
    match target_os.as_str() {
        "macos" => {
            if !icon_icns.exists() {
                println!("cargo:warning=app.icns não encontrado. Gere um .icns a partir do .ico para ícone nativo macOS:");
                println!("cargo:warning=  iconutil -c icns app.iconset (requer macOS)");
            }
        }
        "linux" => {
            if !icon_png.exists() {
                println!("cargo:warning=app.png não encontrado. Coloque um PNG 256x256 como app.png para ícone do Launcher.");
            }
        }
        _ => {}
    }

    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=app.png");
    println!("cargo:rerun-if-changed=build.rs");
}
