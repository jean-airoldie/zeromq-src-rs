use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let wants_debug = env::var_os("PROFILE").unwrap() == "debug";

    let artifacts = zeromq_src::Build::new()
        .build_debug(wants_debug)
        .link_static(true)
        .build();
    artifacts.print_cargo_metadata();
}
