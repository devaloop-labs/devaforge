pub fn get_version() -> &'static str {
    let version: &str = env!("CARGO_PKG_VERSION");
    version
}
