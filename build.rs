use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the manifest directory (project root)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Get the output directory (target/debug or target/release)
    let out_dir = env::var("OUT_DIR").unwrap();

    // Construct the target directory path
    // OUT_DIR is typically target/{profile}/build/{crate}/out
    // We need to go up to target/{profile}
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    // Copy icon files to the output directory
    let icons = ["spotlight-dimmer-icon.ico", "spotlight-dimmer-icon-paused.ico"];

    for icon in &icons {
        let src = Path::new(&manifest_dir).join(icon);
        let dest = target_dir.join(icon);

        if src.exists() {
            if let Err(e) = fs::copy(&src, &dest) {
                println!("cargo:warning=Failed to copy {}: {}", icon, e);
            }
            // Tell cargo to rerun if the icon files change
            println!("cargo:rerun-if-changed={}", icon);
        } else {
            println!("cargo:warning=Icon file not found: {}", src.display());
        }
    }
}
