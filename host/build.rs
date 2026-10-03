fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/nrap.ico");
        let _ = res.compile();
    }
}
