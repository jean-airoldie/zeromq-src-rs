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
    build_type: BuildType,
    link_type: LinkType,
}

impl Build {
    pub fn new() -> Self {
        Self {
            enable_draft: false,
            link_type: LinkType::Dynamic,
            build_type: BuildType::Release,
        }
    }

    pub fn build_type(&mut self, build_type: BuildType) -> &mut Self {
        self.build_type = build_type;
        self
    }

    pub fn link_type(&mut self, link_type: LinkType) -> &mut Self {
        self.link_type = link_type;
        self
    }

    pub fn draft(&mut self, cond: bool) -> &mut Self {
        self.enable_draft = cond;
        self
    }

    pub fn build(&mut self) -> Artifacts {
        let mut config = Config::new(source_dir());

        if self.enable_draft {
            config.define("ENABLE_DRAFTS", "ON");
        } else {
            config.define("ENABLE_DRAFTS", "OFF");
        }

        match self.build_type {
            BuildType::Release => config.define("CMAKE_BUILD_TYPE", "Release"),
            BuildType::Debug => config.define("CMAKE_BUILD_TYPE", "Debug"),
        };

        let mut libs = vec![];

        libs.push(Lib {
            link_type: Some(self.link_type),
            name: "zmq".to_owned(),
        });
        libs.push(Lib {
            link_type: None,
            name: "stdc++".to_owned(),
        });

        match self.link_type {
            LinkType::Static => config
                .define("BUILD_SHARED", "ON")
                .define("BUILD_STATIC", "OFF"),
            LinkType::Dynamic => config
                .define("BUILD_SHARED", "OFF")
                .define("BUILD_STATIC", "ON"),
        };

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
