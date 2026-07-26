fn main() {
    slint_build::compile("ui/app.slint").unwrap();

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        if std::path::Path::new("app.ico").exists() {
            res.set_icon("app.ico");
        }
        res.compile().unwrap();
    }
}