
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut artifacts = zeromq_src::Build::new();
    artifacts
        .build()
        .print_cargo_metadata();
}
