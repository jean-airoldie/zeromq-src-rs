use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let wants_debug = env::var("PROFILE").unwrap() == "debug";
    let wants_static = cfg!(feature = "static");
    let wants_libsodium = cfg!(feature = "libsodium");

    zeromq_src::Build::new()
        .build_debug(wants_debug)
        .link_static(wants_static)
        .with_libsodium(wants_libsodium)
        .build();
    //artifacts.print_cargo_metadata();
}
