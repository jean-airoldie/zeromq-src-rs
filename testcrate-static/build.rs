fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let wants_debug = cfg!(debug_assertions);
    let artifacts = zeromq_src::Build::new()
        .build_debug(wants_debug)
        .link_static(true)
        .build();
    artifacts.print_cargo_metadata();
}
