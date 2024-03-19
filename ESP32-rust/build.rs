use std::env;

use std::path::PathBuf;


fn main() {
    embuild::espidf::sysenv::output();

    let wifi_credentials_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/wifi_credentials.rs");

    if wifi_credentials_path.exists() {
        println!("cargo:rustc-cfg=wifi_credentials");
    }
}
