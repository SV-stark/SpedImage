use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());

    #[cfg(windows)]
    {
        let libheif_dir = Path::new("assets/libheif");
        if libheif_dir.exists() && libheif_dir.is_dir() {
            println!("cargo:rerun-if-changed=assets/libheif");

            for entry in std::fs::read_dir(libheif_dir).expect("Failed to read libheif dir") {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("dll") {
                        let file_name = path.file_name().unwrap();

                        let dest = Path::new(&out_dir).join(file_name);
                        std::fs::copy(&path, &dest).expect("Failed to copy DLL");

                        let dest_in_target = Path::new(&target_dir).join(file_name);
                        let _ = std::fs::copy(&path, &dest_in_target);
                    }
                }
            }
            println!("cargo:warning=Copied HEIF codec DLLs to output directory");
        }
    }

    println!(
        "cargo:rustc-env=BUILD_TARGET_ARCH={}",
        env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default()
    );
}
