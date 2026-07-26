fn main() {
    slint_build::compile("ui/app.slint").unwrap();

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.compile().unwrap();
    }
}