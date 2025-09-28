use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../dist/");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("embedded_assets.rs");

    let dist_dir = Path::new("../dist");

    let index_html = read_asset_file(&dist_dir.join("index.html"));
    let overlay_html = read_asset_file(&dist_dir.join("overlay.html"));
    let style_css = read_asset_file(&dist_dir.join("style.css"));
    let main_js = read_asset_file(&dist_dir.join("main.js"));

    let mut generated_code = String::new();
    generated_code.push_str("// Auto-generated embedded assets\n");
    generated_code.push_str(&format!("pub const INDEX_HTML: &str = r##\"{}\"##;\n", escape_rust_string(&index_html)));
    generated_code.push_str(&format!("pub const OVERLAY_HTML: &str = r##\"{}\"##;\n", escape_rust_string(&overlay_html)));
    generated_code.push_str(&format!("pub const STYLE_CSS: &str = r##\"{}\"##;\n", escape_rust_string(&style_css)));
    generated_code.push_str(&format!("pub const MAIN_JS: &str = r##\"{}\"##;\n", escape_rust_string(&main_js)));

    generated_code.push_str(r#"
pub fn get_asset(path: &str) -> Option<&'static str> {
    match path {
        "index.html" | "/index.html" => Some(INDEX_HTML),
        "overlay.html" | "/overlay.html" => Some(OVERLAY_HTML),
        "style.css" | "/style.css" => Some(STYLE_CSS),
        "main.js" | "/main.js" => Some(MAIN_JS),
        _ => None,
    }
}
"#);

    fs::write(&dest_path, generated_code).unwrap();
    tauri_build::build()
}

fn read_asset_file(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let index_name = "index.html";
            let overlay_name = "overlay.html";
            let style_name = "style.css";
            let js_name = "main.js";

            if filename == index_name {
                String::from("<!DOCTYPE html><html><head><title>Spotlight Dimmer</title></head><body><h1>Spotlight Dimmer</h1><p>Default interface</p></body></html>")
            } else if filename == overlay_name {
                String::from("<!DOCTYPE html><html><head><style>body{margin:0;padding:0;background-color:rgba(0,0,0,0.7);width:100vw;height:100vh;}</style></head><body></body></html>")
            } else if filename == style_name {
                String::from("body { font-family: sans-serif; }")
            } else if filename == js_name {
                String::from("console.log('Spotlight Dimmer loaded');")
            } else {
                String::new()
            }
        }
    }
}

fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}
