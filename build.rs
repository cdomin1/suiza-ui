fn main() {
    println!("cargo:rerun-if-changed=assets/suiza.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/suiza.ico")
            .set("ProductName", "SUIZA Canvas")
            .set("FileDescription", "SUIZA Canvas")
            .set("OriginalFilename", "Suiza.exe")
            .compile()
            .expect("compile Suiza Windows icon resource");
    }
}
