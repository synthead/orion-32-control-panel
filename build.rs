const WIN_RESOURCE_NAME: &str = "Orion 32 Control Panel";
const WIN_RESOURCE_LANGUAGE_EN_US: u16 = 0x0409;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();

        res.set("FileDescription", WIN_RESOURCE_NAME);
        res.set("FileVersion", &version);

        res.set("ProductName", WIN_RESOURCE_NAME);
        res.set("ProductVersion", &version);

        res.set_language(WIN_RESOURCE_LANGUAGE_EN_US);
        res.set_icon("assets/icons/icon.ico");

        res.compile().unwrap();
    }
}
