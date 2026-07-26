use std::env;
use std::path::Path;

#[cfg(target_os = "windows")]
mod windows_build {
    use image;
    use winres;
    use std::path::Path;

    pub fn build_windows_resources(icon_ico: &Path) {
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

    pub fn convert_ico_to_png(icon_ico: &Path, icon_png: &Path) {
        if icon_ico.exists() && !icon_png.exists() {
            if let Ok(img) = image::open(icon_ico) {
                let rgba = img.to_rgba8();
                let _ = rgba.save(icon_png);
                println!("cargo:warning=Gerado app.png a partir do app.ico");
            }
        }
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let icon_ico = Path::new(&manifest_dir).join("app.ico");
    let icon_png = Path::new(&manifest_dir).join("app.png");
    let icon_icns = Path::new(&manifest_dir).join("app.icns");

    // Converte .ico para .png se não existir (Linux) - roda em todas as plataformas
    #[cfg(target_os = "windows")]
    {
        use windows_build::{convert_ico_to_png, build_windows_resources};
        convert_ico_to_png(&icon_ico, &icon_png);
        build_windows_resources(&icon_ico);
    }

    // Compila componentes Slint
    slint_build::compile("src/ui/library.slint").expect("Falha ao compilar library.slint");

    // Instruções específicas por plataforma
    match target_os.as_str() {
        "windows" => {
            // Windows usa .ico direto no executável via winres (já feito acima)
        }
        "macos" => {
            if !icon_icns.exists() {
                println!("cargo:warning=app.icns não encontrado. Gere um .icns a partir do .ico para macOS (use iconutil no macOS)");
            }
        }
        "linux" => {
            if !icon_png.exists() {
                println!("cargo:warning=app.png não encontrado. Gere um .png (256x256) para Linux");
            }
        }
        _ => {}
    }

    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ui/library.slint");
}