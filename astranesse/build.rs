use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let src = "../assets/scripts";
    println!("cargo:rerun-if-changed={}", src);
    
    let mut include_text = "lazy_static!{
    static ref COMPILED_SCRIPTS: FxHashMap<&'static str, &'static [u8]> = {
        let mut scripts: FxHashMap<&'static str, &'static [u8]> = FxHashMap::default();".to_owned();
    for entry in fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() { continue; }
        match path.extension() {
            None => continue,
            Some(s) => if s.to_str() != Some("bisc") {continue}
        };
        
        let bytes = match biscuit::compile_file(&path.to_string_lossy()) {
            Ok(b) => b,
            Err(e) => {
                println!("cargo::error={}", e);
                std::process::exit(1);
            }
        };
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let bin_file = Path::new(&out_dir).join(format!("{}.b", name));
        fs::write(&bin_file, bytes).unwrap();

        include_text = format!("{}\n        scripts.insert(\"{}\", include_bytes!(\"{}\"));", include_text, name, bin_file.to_string_lossy());
    }

    include_text = format!("{}\n        scripts\n    }};\n}}", include_text);
    fs::write(Path::new(&out_dir).join("scripts.rs"), include_text).unwrap();
}