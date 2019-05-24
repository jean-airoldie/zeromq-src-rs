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
pub struct Build {
    enable_draft: bool,
    build_debug: bool,
    link_static: bool,
    perf_tool: bool,
}

impl Build {
    pub fn new() -> Self {
        Self {
            enable_draft: false,
            build_debug: false,
            link_static: false,
            perf_tool: false,
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

    pub fn perf_tool(&mut self, enabled: bool) -> &mut Self {
        self.perf_tool = enabled;
        self
    }

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

        let mut libs = vec![];

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

        libs.push(Lib::new("zmq", link_type));

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
            // Force a consistant generator.
            config.generator("NMake Makefiles");
        }

        let dest = config.build();

        let lib_dir = dest.join("lib");
        let include_dir = dest.join("include");
        let pkg_config_dir = lib_dir.join("pkgconfig");

        // On windows we need to rename the static compiled lib
        // since its name is unpredictable.
        if target.contains("msvc") {
            if let Err(_) = rename_libzmq_in_dir(&lib_dir, "zmq.lib") {
                panic!("unable to find compiled `libzmq` lib");
            }
        }

        Artifacts {
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
