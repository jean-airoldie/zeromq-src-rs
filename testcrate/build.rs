fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let artifacts = zeromq_src::Build::new().link_static(true).build();
    artifacts.print_cargo_metadata();
}
