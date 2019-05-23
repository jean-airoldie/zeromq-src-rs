fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let artifacts = zeromq_src::Build::new()
        .link_static(true)
        .define("CMAKE_TRY_COMPILE_TARGET_TYPE", "STATIC_LIBRARY")
        .build();
    artifacts.print_cargo_metadata();
}
