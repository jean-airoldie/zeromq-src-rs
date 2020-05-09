use cmake::Config;

use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor")
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
pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    out_dir: PathBuf,
    pkg_config_dir: PathBuf,
    libs: Vec<Lib>,
}

impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }
    pub fn pkg_config_dir(&self) -> &Path {
        &self.pkg_config_dir
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn libs(&self) -> &[Lib] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib={}{}", lib.link_type, lib.name);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
        println!("cargo:out={}", self.out_dir.display());
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

#[derive(Debug, Clone, Copy)]
pub enum BuildType {
    Release,
    Debug,
}

#[derive(Debug, Clone)]
pub struct Lib {
    name: String,
    link_type: LinkType,
}

impl Lib {
    fn new<S>(name: S, link_type: LinkType) -> Self
    where
        S: Into<String>,
    {
        let name = name.into();

        Self { name, link_type }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn link_type(&self) -> LinkType {
        self.link_type
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
        let mut config = Config::new(source_dir());

        config
            // For the LIBDIR to always be `lib`.
            .define("CMAKE_INSTALL_LIBDIR", "lib")
            // `libzmq` uses C99 but doesn't specify it.
            .define("CMAKE_C_STANDARD", "99")
            .define("ZMQ_BUILD_TESTS", "OFF");

        if self.enable_draft {
            config.define("ENABLE_DRAFTS", "ON");
        } else {
            config.define("ENABLE_DRAFTS", "OFF");
        }

        if self.enable_curve {
            config.define("ENABLE_CURVE", "ON");
        } else {
            config.define("ENABLE_CURVE", "OFF");
        }

        if self.build_debug {
            config.define("CMAKE_BUILD_TYPE", "Debug");
        } else {
            config.define("CMAKE_BUILD_TYPE", "Release");
        }

        if self.perf_tool {
            config.define("WITH_PERF_TOOL", "ON");
        } else {
            config.define("WITH_PERF_TOOL", "OFF");
        }

        let target = env::var("TARGET").unwrap();

        let link_type = {
            if self.link_static {
                config
                    .define("BUILD_SHARED", "OFF")
                    .define("BUILD_STATIC", "ON");

                if target.contains("msvc") {
                    config.cflag("-DZMQ_STATIC");
                }

                LinkType::Static
            } else {
                config
                    .define("BUILD_SHARED", "ON")
                    .define("BUILD_STATIC", "OFF");

                LinkType::Dynamic
            }
        };

        if target.contains("msvc") && link_type == LinkType::Dynamic {
            panic!("dynamic compilation is currently not supported on windows");
        }

        let mut libs = vec![];

        libs.push(Lib::new("zmq", link_type));

        if let Some(ref location) = self.libsodium {
            config.define("WITH_LIBSODIUM", "ON");

            config.define("SODIUM_LIBRARIES", location.lib_dir());
            config.define("SODIUM_INCLUDE_DIRS", location.include_dir());

            if target.contains("msvc") {
                libs.push(Lib::new("libsodium", LinkType::Static));
            } else {
                libs.push(Lib::new("sodium", LinkType::Unspecified));
            }
        } else {
            config.define("WITH_LIBSODIUM", "OFF");
        }

        if target.contains("apple")
            || target.contains("freebsd")
            || target.contains("openbsd")
        {
            libs.push(Lib::new("c++", LinkType::Dynamic));
        } else if target.contains("linux") {
            libs.push(Lib::new("stdc++", LinkType::Dynamic));
        } else if target.contains("msvc") {
            libs.push(Lib::new("iphlpapi", LinkType::Dynamic));
        }

        if target.contains("msvc") {
            // We need to explicitly disable `/GL` flag, otherwise
            // we get linkage error.
            config.cxxflag("/GL-");
            // Fix warning C4530: "C++ exception handler used, but unwind
            // semantics are not enabled. Specify /EHsc"
            config.cxxflag("/EHsc");
        }

        let out_dir = config.build();
        let lib_dir = out_dir.join("lib");
        let include_dir = out_dir.join("include");
        let pkg_config_dir = lib_dir.join("pkgconfig");

        // On windows we need to rename the static compiled lib
        // since its name is unpredictable.
        if target.contains("msvc")
            && rename_libzmq_in_dir(&lib_dir, "zmq.lib").is_err()
        {
            panic!("unable to find compiled `libzmq` lib");
        }

        Artifacts {
            out_dir,
            lib_dir,
            include_dir,
            pkg_config_dir,
            libs,
        }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}
