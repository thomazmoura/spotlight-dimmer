use std::env;
use std::fs;
use std::path::Path;

fn main() {
    copy_icon_files();
}

fn copy_icon_files() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_ancestors() {
        // Test that we can navigate up the directory tree correctly
        let path = PathBuf::from("/a/b/c/d/e");
        let ancestor = path.ancestors().nth(3).unwrap();
        assert_eq!(ancestor, Path::new("/a/b"));
    }

    #[test]
    fn test_icon_file_names() {
        let icons = ["spotlight-dimmer-icon.ico", "spotlight-dimmer-icon-paused.ico"];
        assert_eq!(icons.len(), 2);
        assert!(icons[0].ends_with(".ico"));
        assert!(icons[1].ends_with(".ico"));
        assert!(icons[1].contains("paused"));
    }

    #[test]
    fn test_path_join() {
        let base = Path::new("/test");
        let joined = base.join("icon.ico");
        assert!(joined.to_str().unwrap().contains("icon.ico"));
    }
}
