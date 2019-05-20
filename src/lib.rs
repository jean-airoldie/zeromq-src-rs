use cmake::Config;

use std::{
    fmt,
    path::{Path, PathBuf},
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
}

impl Build {
    pub fn new() -> Self {
        Self {
            enable_draft: false,
            build_debug: false,
            link_static: false,
        }
    }

    /// Build & link statically instead of dynamically.
    pub fn link_static(&mut self) -> &mut Self {
        self.link_static = true;
        self
    }

    /// Build the debug version of the lib.
    pub fn build_debug(&mut self) -> &mut Self {
        self.build_debug = true;
        self
    }

    /// Enable the DRAFT API.
    pub fn enable_draft(&mut self) -> &mut Self {
        self.enable_draft = true;
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

        let mut libs = vec![];

        libs.push(Lib {
            link_type: None,
            name: "stdc++".to_owned(),
        });

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

        let dest = config.build();

        let lib_path = "lib64";

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
