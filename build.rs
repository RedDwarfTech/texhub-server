use std::env;
use std::path::Path;

fn main() {
    // If a prebuilt native library exists under `src/so`, add that to the link search path
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let so_dir = Path::new(&manifest).join("src/so");
    if so_dir.exists() {
        println!("cargo:rustc-link-search=native={}", so_dir.display());
    }

    // In case some environments need an explicit link-lib directive
    println!("cargo:rustc-link-lib=synctex_parser");
}
