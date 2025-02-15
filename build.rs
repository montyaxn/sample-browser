use std::{env, fs};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-link-search=native=Everything/lib");
    println!("cargo:rustc-link-lib=dylib=Everything64");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = Path::new(&manifest_dir).join("target").join("debug");
    let dll_dest = Path::new(&target_dir).join("Everything64.dll");

    let dll_source = Path::new(&manifest_dir).join("Everything").join("dll").join("Everything64.dll");

    if dll_source.exists() {
        fs::copy(dll_source, dll_dest).expect("Failed to copy DLL");
    } else {
        eprintln!("DLL not found: {}", dll_source.display());
    }
}