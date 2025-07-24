use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

fn add_cpp_sources(
    build: &mut cc::Build,
    root: impl AsRef<Path>,
    files: &[&str],
) {
    let root = root.as_ref();
    build.files(files.iter().map(|src| {
        let mut p = root.join(src);
        p.set_extension("cpp");
        p
    }));

    build.include(root);
}

fn add_c_sources(
    build: &mut cc::Build,
    root: impl AsRef<Path>,
    files: &[&str],
) {
    let root = root.as_ref();
    // Temporarily use c instead of c++.
    build.cpp(false);
    build.files(files.iter().map(|src| {
        let mut p = root.join(src);
        p.set_extension("c");
        p
    }));

    build.include(root);
    build.cpp(true);
}

// Returns Ok(()) is file was renamed,
// Returns Err(()) otherwise.
fn rename_libzmq_in_dir<D, N>(dir: D, new_name: N) -> Result<(), ()>
where
    D: AsRef<Path>,
    N: AsRef<Path>,
{
    let dir = dir.as_ref();
    let new_name = new_name.as_ref();

    for entry in fs::read_dir(dir).unwrap() {
        let file_name = entry.unwrap().file_name();
        if file_name.to_string_lossy().starts_with("libzmq") {
            fs::rename(dir.join(file_name), dir.join(new_name)).unwrap();
            return Ok(());
        }
    }

    Err(())
}

mod glibc {
    use std::{
        env,
        path::{Path, PathBuf},
    };

    // Attempt to compile a c program that links to strlcpy from the std
    // library to determine whether glibc packages it.
    pub(crate) fn has_strlcpy() -> bool {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/strlcpy.c");
        println!("cargo:rerun-if-changed={}", src.display());

        let dest =
            PathBuf::from(env::var("OUT_DIR").unwrap()).join("has_strlcpy");

        cc::Build::new()
            .warnings(false)
            .get_compiler()
            .to_command()
            .arg(src)
            .arg("-o")
            .arg(dest)
            .status()
            .expect("failed to execute gcc")
            .success()
    }
}

mod windows {
    use std::{
        env,
        path::{Path, PathBuf},
    };

    // Attempt to compile a c program that links to winsock2.h & aflinux.h
    // library to determine whether windows has these header files.
    pub(crate) fn has_icp_headers() -> bool {
        let src =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/windows_ipc.c");
        println!("cargo:rerun-if-changed={}", src.display());

        let dest = PathBuf::from(env::var("OUT_DIR").unwrap())
            .join("has_windows_ipc_headers");

        cc::Build::new()
            .warnings(false)
            .get_compiler()
            .to_command()
            .arg(src)
            .arg("-o")
            .arg(dest)
            .status()
            .expect("failed to execute gcc")
            .success()
    }
}

mod cxx11 {
    use std::{
        env,
        path::{Path, PathBuf},
    };

    // Attempt to compile a c program that links has the c++11 flag to determine
    // whether it is supported.
    pub(crate) fn has_cxx11() -> bool {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/trivial.c");
        println!("cargo:rerun-if-changed={}", src.display());

        let dest =
            PathBuf::from(env::var("OUT_DIR").unwrap()).join("has_cxx11");

        cc::Build::new()
            .cpp(true)
            .warnings(true)
            .warnings_into_errors(true)
            .std("c++11")
            .get_compiler()
            .to_command()
            .arg(src)
            .arg("-o")
            .arg(dest)
            .status()
            .expect("failed to execute gcc")
            .success()
    }
}

/// The location of a library.
#[derive(Debug, Clone)]
pub struct LibLocation {
    include_dir: PathBuf,
    lib_dir: PathBuf,
}

impl LibLocation {
    /// Create a new `LibLocation`.
    pub fn new<L, I>(lib_dir: L, include_dir: I) -> Self
    where
        L: Into<PathBuf>,
        I: Into<PathBuf>,
    {
        Self {
            include_dir: include_dir.into(),
            lib_dir: lib_dir.into(),
        }
    }

    /// Returns the `include_dir`.
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    /// Returns the `lib_dir`.
    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }
}

/// Settings for building zmq.
#[derive(Debug, Clone)]
pub struct Build {
    enable_draft: bool,
    build_debug: bool,
    libsodium: Option<LibLocation>,
}

impl Build {
    /// Create a new build.
    pub fn new() -> Self {
        Self {
            enable_draft: false,
            build_debug: false,
            libsodium: None,
        }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}

impl Build {
    /// Build the debug version of the lib.
    pub fn build_debug(&mut self, enabled: bool) -> &mut Self {
        self.build_debug = enabled;
        self
    }

    /// Enable the DRAFT API.
    pub fn enable_draft(&mut self, enabled: bool) -> &mut Self {
        self.enable_draft = enabled;
        self
    }

    /// Enable the CURVE feature and link against an external `libsodium` library.
    ///
    /// Users can link against an installed lib or another `sys` or `src` crate
    /// that provides the lib.
    ///
    /// Note that by default `libzmq` builds without `libsodium` by instead
    /// relying on `tweetnacl`. However since this `tweetnacl` [has never been
    /// audited nor is ready for production](https://github.com/zeromq/libzmq/issues/3006),
    /// we require linking against `libsodium` to enable `ZMQ_CURVE`.
    ///
    /// [`links build metadata`]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key
    pub fn with_libsodium(&mut self, maybe: Option<LibLocation>) -> &mut Self {
        self.libsodium = maybe;
        self
    }

    /// Build and link the lib based on the provided options.
    ///
    /// Returns an `Artifacts` which contains metadata for linking
    /// against the compiled lib from rust code.
    pub fn build(&mut self) {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor");

        let mut build = cc::Build::new();
        build
            // We use c++ as the default.
            .cpp(true)
            .define("ZMQ_BUILD_TESTS", "OFF")
            .include(vendor.join("include"))
            .include(vendor.join("src"));

        add_cpp_sources(
            &mut build,
            vendor.join("src"),
            &[
                "address",
                "channel",
                "client",
                "clock",
                "ctx",
                "curve_client",
                "curve_mechanism_base",
                "curve_server",
                "dealer",
                "decoder_allocators",
                "devpoll",
                "dgram",
                "dish",
                "dist",
                "endpoint",
                "epoll",
                "err",
                "fq",
                "gather",
                "gssapi_client",
                "gssapi_mechanism_base",
                "gssapi_server",
                "io_object",
                "io_thread",
                "ip_resolver",
                "ip",
                "ipc_address",
                "ipc_connecter",
                "ipc_listener",
                "kqueue",
                "lb",
                "mailbox_safe",
                "mailbox",
                "mechanism_base",
                "mechanism",
                "metadata",
                "msg",
                "mtrie",
                "norm_engine",
                "null_mechanism",
                "object",
                "options",
                "own",
                "pair",
                "peer",
                "pgm_receiver",
                "pgm_sender",
                "pgm_socket",
                "pipe",
                "plain_client",
                "plain_server",
                "poll",
                "poller_base",
                "polling_util",
                "pollset",
                "precompiled",
                "proxy",
                "pub",
                "pull",
                "push",
                "radio",
                "radix_tree",
                "random",
                "raw_decoder",
                "raw_encoder",
                "raw_engine",
                "reaper",
                "rep",
                "req",
                "router",
                "scatter",
                "select",
                "server",
                "session_base",
                "signaler",
                "socket_base",
                "socket_poller",
                "socks_connecter",
                "socks",
                "stream_connecter_base",
                "stream_engine_base",
                "stream_listener_base",
                "stream",
                "sub",
                "tcp_address",
                "tcp_connecter",
                "tcp_listener",
                "tcp",
                "thread",
                "timers",
                "tipc_address",
                "tipc_connecter",
                "tipc_listener",
                "trie",
                "udp_address",
                "udp_engine",
                "v1_decoder",
                "v1_encoder",
                "v2_decoder",
                "v2_encoder",
                "v3_1_encoder",
                "vmci_address",
                "vmci_connecter",
                "vmci_listener",
                "vmci",
                "ws_address",
                "ws_connecter",
                "ws_decoder",
                "ws_encoder",
                "ws_engine",
                "ws_listener",
                // "wss_address", // requires gnutls
                // "wss_engine", // requires gnutls
                "xpub",
                "xsub",
                "zap_client",
                "zmq_utils",
                "zmq",
                "zmtp_engine",
            ],
        );

        add_c_sources(&mut build, vendor.join("external/sha1"), &["sha1.c"]);

        if self.enable_draft {
            build.define("ZMQ_BUILD_DRAFT_API", "1");
        }

        build.define("ZMQ_USE_CV_IMPL_STL11", "1");
        build.define("ZMQ_STATIC", "1");
        build.define("ZMQ_USE_BUILTIN_SHA1", "1");

        build.define("ZMQ_HAVE_WS", "1");

        let target = env::var("TARGET").unwrap();

        if let Some(libsodium) = &self.libsodium {
            build.define("ZMQ_USE_LIBSODIUM", "1");
            build.define("ZMQ_HAVE_CURVE", "1");

            build.include(libsodium.include_dir());
            println!(
                "cargo:rustc-link-search={}",
                libsodium.lib_dir().display()
            );

            if target.contains("msvc") {
                fs::copy(
                    libsodium
                        .include_dir()
                        .join("../../../builds/msvc/version.h"),
                    libsodium.include_dir().join("sodium/version.h"),
                )
                .unwrap();
            }

            if target.contains("msvc") {
                println!("cargo:rustc-link-lib=static=libsodium");
            } else {
                println!("cargo:rustc-link-lib=static=sodium");
            }
        }

        let create_platform_hpp_shim = |build: &mut cc::Build| {
            // https://cmake.org/cmake/help/latest/command/configure_file.html
            // TODO: Replace `#cmakedefine` with the appropriate `#define`
            // let _platform_file =
            //     fs::read_to_string(path.join("builds/cmake/platform.hpp.in"))
            //         .unwrap();

            let out_includes = PathBuf::from(std::env::var("OUT_DIR").unwrap());

            // Write out an empty platform file: defines will be set through cc directly,
            // sync to prevent potential IO troubles later on
            let mut f =
                File::create(out_includes.join("platform.hpp")).unwrap();
            f.write_all(b"").unwrap();
            f.sync_all().unwrap();

            build.include(out_includes);
        };

        let mut has_strlcpy = false;
        if target.contains("windows") {
            // on windows vista and up we can use `epoll` through the `wepoll` lib

            add_c_sources(
                &mut build,
                vendor.join("external/wepoll"),
                &["wepoll.c"],
            );

            build.define("ZMQ_HAVE_WINDOWS", "1");
            build.define("ZMQ_IOTHREAD_POLLER_USE_EPOLL", "1");
            build.define("ZMQ_POLL_BASED_ON_POLL", "1");
            build.define("_WIN32_WINNT", "0x0600"); // vista
            build.define("ZMQ_HAVE_STRUCT_SOCKADDR_UN", "1");

            println!("cargo:rustc-link-lib=iphlpapi");

            if target.contains("msvc") {
                build.include(vendor.join("builds/deprecated-msvc"));
                // We need to explicitly disable `/GL` flag, otherwise
                // we get linkage error.
                build.flag("/GL-");

                // Fix warning C4530: "C++ exception handler used, but unwind
                // semantics are not enabled. Specify /EHsc"
                build.flag("/EHsc");
            } else {
                create_platform_hpp_shim(&mut build);
                build.define("HAVE_STRNLEN", "1");
            }

            if !target.contains("uwp") && windows::has_icp_headers() {
                build.define("ZMQ_HAVE_IPC", "1");
            }
        } else if target.contains("linux") {
            create_platform_hpp_shim(&mut build);
            build.define("ZMQ_HAVE_LINUX", "1");
            build.define("ZMQ_IOTHREAD_POLLER_USE_EPOLL", "1");
            build.define("ZMQ_POLL_BASED_ON_POLL", "1");
            build.define("ZMQ_HAVE_IPC", "1");

            build.define("HAVE_STRNLEN", "1");
            build.define("ZMQ_HAVE_UIO", "1");
            build.define("ZMQ_HAVE_STRUCT_SOCKADDR_UN", "1");

            if target.contains("android") {
                has_strlcpy = true;
            }

            if target.contains("musl") {
                has_strlcpy = true;
            }
        } else if target.contains("apple") || target.contains("freebsd") {
            create_platform_hpp_shim(&mut build);
            build.define("ZMQ_IOTHREAD_POLLER_USE_KQUEUE", "1");
            build.define("ZMQ_POLL_BASED_ON_POLL", "1");
            build.define("HAVE_STRNLEN", "1");
            build.define("ZMQ_HAVE_UIO", "1");
            build.define("ZMQ_HAVE_IPC", "1");
            build.define("ZMQ_HAVE_STRUCT_SOCKADDR_UN", "1");
            has_strlcpy = true;
        }

        // https://github.com/jean-airoldie/zeromq-src-rs/issues/28
        if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "gnu"
            && !has_strlcpy
            && glibc::has_strlcpy()
        {
            has_strlcpy = true;
        }

        if has_strlcpy {
            build.define("ZMQ_HAVE_STRLCPY", "1");
        }

        // MSVC does not support c++11, since c++14 is the minimum.
        if !target.contains("msvc") {
            // Enable c++11 if possible to fix issue 45
            // (https://github.com/jean-airoldie/zeromq-src-rs/issues/45).
            if cxx11::has_cxx11() {
                build.std("c++11");
            }
        }

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let lib_dir = out_dir.join("lib");

        build.out_dir(&lib_dir);
        build.compile("zmq");

        // On windows we need to rename the static compiled lib
        // since its name is unpredictable.
        if target.contains("msvc")
            && rename_libzmq_in_dir(&lib_dir, "zmq.lib").is_err()
        {
            panic!("unable to find compiled `libzmq` lib");
        }

        let source_dir = out_dir.join("source");
        let include_dir = source_dir.join("include");

        // Finally we need to copy the include files.
        dircpy::copy_dir(vendor.join("include"), &include_dir)
            .expect("unable to copy include dir");
        dircpy::copy_dir(vendor.join("src"), source_dir.join("src"))
            .expect("unable to copy src dir");
        dircpy::copy_dir(vendor.join("external"), source_dir.join("external"))
            .expect("unable to copy external dir");

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=zmq");
        println!("cargo:include={}", include_dir.display());
        println!("cargo:lib={}", lib_dir.display());
        println!("cargo:out={}", out_dir.display());
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn version_works() {
        let version = testcrate::version();
        println!("{version:?}");
        assert_eq!(version, (4, 3, 5));
    }

    #[test]
    fn sodium_version_works() {
        let version = testcrate::sodium_version();
        println!("{:?}", version.to_str().unwrap());
        assert!(version.to_str().unwrap().starts_with("1.0"));
    }
}
