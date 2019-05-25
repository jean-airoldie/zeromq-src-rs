fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let artifacts = zeromq_src::Build::new()
        .args(vec!("--enable-static".to_string(), "--disable-shared".to_string()))
        .build();
    artifacts.print_cargo_metadata();
}
