use std::path::Path;

fn add_res<'a>(var: &'a str, path: &'a str) {
    let path = std::path::absolute(Path::new(path)).unwrap();
    println!("cargo::rerun-if-changed={}", path.to_str().unwrap());
    println!("cargo::rustc-env={}={}", var, path.to_str().unwrap());
}

fn main() {
    add_res("SIMPLE_SHADER", "src/simple_shader.wgsl");
    add_res("SQUARE_SHADER", "src/square_shader.wgsl");
}

