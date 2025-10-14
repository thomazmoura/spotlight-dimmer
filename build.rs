use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Only embed icon on Windows builds
    #[cfg(windows)]
    {
        embed_icon_resource();
    }

    copy_icon_files();
}

#[cfg(windows)]
fn embed_icon_resource() {
    // Embed the icon into the executable using winres
    let mut res = winres::WindowsResource::new();
    res.set_icon("spotlight-dimmer-icon.ico");

    // Set application manifest for proper DPI awareness and Windows 10/11 compatibility
    res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.0.0.0"
    processorArchitecture="*"
    name="SpotlightDimmer"
    type="win32"
  />
  <description>Spotlight Dimmer - Focus by dimming inactive displays</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and Windows 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#);

    if let Err(e) = res.compile() {
        eprintln!("Failed to compile Windows resources: {}", e);
    }
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
    let icons = [
        "spotlight-dimmer-icon.ico",
        "spotlight-dimmer-icon-paused.ico",
    ];

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
        let icons = [
            "spotlight-dimmer-icon.ico",
            "spotlight-dimmer-icon-paused.ico",
        ];
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
