use cmake::Config;

use std::{
    fmt,
    fs::read_dir,
    path::{Path, PathBuf},
    env,
};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor")
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
            if let Some(link_type) = lib.link_type {
                println!("cargo:rustc-link-lib={}={}", link_type, lib.name);
            } else {
                println!("cargo:rustc-link-lib={}", lib.name);
            }
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LinkType {
    Dynamic,
    Static,
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkType::Dynamic => write!(f, "dylib"),
            LinkType::Static => write!(f, "static"),
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
    link_type: Option<LinkType>,
}

impl Lib {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn link_type(&self) -> Option<LinkType> {
        self.link_type
    }
}

#[derive(Debug, Clone)]
pub struct Build {
    enable_draft: bool,
    build_debug: bool,
    link_static: bool,
    perf_tool: bool
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

                LinkType::Static
            } else {
                config
                    .define("BUILD_SHARED", "ON")
                    .define("BUILD_STATIC", "OFF");

                LinkType::Dynamic
            }
        };

        libs.push(Lib {
            link_type: Some(link_type),
            name: "zmq".to_owned(),
        });

        if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
            libs.push(Lib {
                link_type: Some(LinkType::Dynamic),
                name: "c++".to_owned(),
            });
        } else {
            libs.push(Lib {
                link_type: Some(LinkType::Dynamic),
                name: "stdc++".to_owned(),
            });
        }

        let dest = config.build();

        // Find the system dependant lib directory.
        let lib_path = {
            if read_dir(dest.join("lib")).is_ok() {
                "lib"
            } else if read_dir(dest.join("lib64")).is_ok() {
                "lib64"
            } else {
                panic!("cannot find lib directory")
            }
        };

        let lib_dir = dest.join(lib_path);
        let include_dir = dest.join("include");
        let pkg_config_dir = lib_dir.join("pkgconfig");

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
