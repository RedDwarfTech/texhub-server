use std::env;
use std::path::Path;

fn main() {
    // If a prebuilt native library exists under `src/so`, add that to the link search path
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let so_dir = Path::new(&manifest).join("src/so");
    if so_dir.exists() {
        let so_dir_str = so_dir.display().to_string();
        println!("cargo:rustc-link-search=native={}", so_dir_str);
        
        // On macOS, also set rpath so the library can be found at runtime
        #[cfg(target_os = "macos")]
        {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", so_dir_str);
        }
    }

    // In case some environments need an explicit link-lib directive
    println!("cargo:rustc-link-lib=synctex_parser");
}
