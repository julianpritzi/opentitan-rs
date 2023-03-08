use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it.
    fs::write(out_dir.join("layout.ld"), include_bytes!("layout.ld")).unwrap();
    fs::write(out_dir.join("memory.ld"), include_bytes!("memory.ld")).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=layout.x");
    println!("cargo:rerun-if-changed=build.rs");
}
