use std::{
    env, fmt, fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor")
}

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
    build.files(files.iter().map(|src| {
        let mut p = root.join(src);
        p.set_extension("c");
        p
    }));

    build.include(root);
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

#[derive(Debug, Clone)]
pub struct Artifacts {}

impl Artifacts {
    pub fn print_cargo_metadata(&self) {
        let target = env::var("TARGET").unwrap();
        if target.contains("windows") {
            println!("cargo:rustc-link-lib=dylib=iphlpapi");
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LinkType {
    Dynamic,
    Static,
    Unspecified,
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkType::Dynamic => write!(f, "dylib="),
            LinkType::Static => write!(f, "static="),
            LinkType::Unspecified => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LibLocation {
    include_dir: PathBuf,
    lib_dir: PathBuf,
}

impl LibLocation {
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

    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }
}

#[derive(Debug, Clone)]
pub struct Build {
    enable_draft: bool,
    enable_curve: bool,
    build_debug: bool,
    link_static: bool,
    perf_tool: bool,
    libsodium: Option<LibLocation>,
}

impl Build {
    pub fn new() -> Self {
        Self {
            enable_draft: false,
            enable_curve: true,
            build_debug: false,
            link_static: false,
            perf_tool: false,
            libsodium: None,
        }
    }

    /// Build & link statically instead of dynamically.
    pub fn link_static(&mut self, enabled: bool) -> &mut Self {
        self.link_static = enabled;
        self
    }

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

    // Enable CURVE mechanism.
    pub fn enable_curve(&mut self, enabled: bool) -> &mut Self {
        self.enable_curve = enabled;
        self
    }

    /// Build with perf-tools.
    pub fn perf_tool(&mut self, enabled: bool) -> &mut Self {
        self.perf_tool = enabled;
        self
    }

    /// Use an external `libsodium` library instead of `tweenacl`.
    ///
    /// Users can link against an installed lib or another `sys` or `src` crate
    /// that provides the lib.
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
    pub fn build(&mut self) -> Artifacts {
        let path = source_dir();

        let mut build = cc::Build::new();
        build
            .cpp(true)
            .define("ZMQ_BUILD_TESTS", "OFF")
            .include(path.join("include"))
            .include(path.join("src"));

        add_cpp_sources(
            &mut build,
            path.join("src"),
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

        add_c_sources(&mut build, path.join("external/sha1"), &["sha1.c"]);

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
            println!("cargo:rustc-link-search={:?}", libsodium.lib_dir());

            if target.contains("msvc") {
                std::fs::copy(
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
                println!("cargo:rustc-link-lib=sodium");
            }
        }

        let create_platform_hpp_shim = |build: &mut cc::Build| {
            // https://cmake.org/cmake/help/latest/command/configure_file.html
            // TODO: Replace `#cmakedefine` with the appropriate `#define`
            // let _platform_file =
            //     std::fs::read_to_string(path.join("builds/cmake/platform.hpp.in"))
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

        if target.contains("windows") {
            build.define("ZMQ_IOTHREAD_POLLER_USE_SELECT", "1");
            build.define("ZMQ_POLL_BASED_ON_SELECT", "1");
            build.define("_WIN32_WINNT", "0x0600"); // vista
            println!("cargo:rustc-link-lib=iphlpapi");

            if target.contains("msvc") {
                build.include(path.join("builds/deprecated-msvc"));
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
        } else if target.contains("linux") {
            create_platform_hpp_shim(&mut build);
            build.define("ZMQ_IOTHREAD_POLLER_USE_EPOLL", "1");
            build.define("ZMQ_POLL_BASED_ON_POLL", "1");

            build.define("HAVE_STRNLEN", "1");
            build.define("ZMQ_HAVE_UIO", "1");
        } else if target.contains("apple") {
            create_platform_hpp_shim(&mut build);
            build.define("ZMQ_IOTHREAD_POLLER_USE_KQUEUE", "1");
            build.define("ZMQ_POLL_BASED_ON_POLL", "1");
            build.define("HAVE_STRNLEN", "1");
            build.define("ZMQ_HAVE_STRLCPY", "1");
            build.define("ZMQ_HAVE_UIO", "1");
        }

        build.compile("zmq");
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let lib_dir = out_dir.join("");

        // On windows we need to rename the static compiled lib
        // since its name is unpredictable.
        if target.contains("msvc")
            && rename_libzmq_in_dir(&lib_dir, "zmq.lib").is_err()
        {
            panic!("unable to find compiled `libzmq` lib");
        }
        Artifacts {}
    }
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}
