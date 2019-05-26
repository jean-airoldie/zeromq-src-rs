extern crate cc;

use std::env;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor")
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
    configure: PathBuf,
    configure_args: Vec<String>,
}

pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    libs: Vec<String>,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| {
                PathBuf::from(s).join("build")
            }),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
            configure: source_dir().join("src").join("configure"),
            configure_args: vec!(),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    pub fn configure(&mut self, path: &PathBuf, args: Vec<String>) -> &mut Build {
        self.configure = path.clone();
        self.configure_args.clone_from(&args);
        self
    }

    pub fn args(&mut self, args: Vec<String>) -> &mut Build {
        self.configure_args.clone_from(&args);
        self
    }

    pub fn build(&mut self) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET dir not set")[..];
        let host = &self.host.as_ref().expect("HOST dir not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let build_dir = out_dir.join("build");
        let install_dir = out_dir.join("install");

        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).unwrap();
        }
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).unwrap();
        }

        let inner_dir = build_dir.join("src");
        fs::create_dir_all(&inner_dir).unwrap();
        cp_r(&source_dir(), &inner_dir);

        let cmd = inner_dir.join("configure");
        if !cmd.exists() {
            let autogen = inner_dir.join("autogen.sh");
            if !autogen.exists() {
                panic!("neither configure nor autogen.sh exist! {:?}", cmd)
            }
            let mut autogen = Command::new(autogen);
            autogen.current_dir(&inner_dir);
            self.run_command(autogen, "running autogen.sh");
        }
        let mut configure = Command::new(&cmd);
        for arg in &self.configure_args {
            configure.arg(arg);
        }
        if target.contains("pc-windows-gnu") {
            configure.arg(&format!("--prefix={}", sanitize_sh(&install_dir)));
        } else {
            configure.arg(&format!("--prefix={}", install_dir.display()));
        }

        if host != target {
            // don't ask
            configure.arg(&format!("--target={}", host));
            configure.arg(&format!("--host={}", target));
        }

        // And finally, run configure!
        configure.current_dir(&inner_dir);
        self.run_command(configure, "configuring build");

        // On MSVC we use `nmake.exe` with a slightly different invocation, so
        // have that take a different path than the standard `make` below.
        if target.contains("msvc") {
            let mut build = cc::windows_registry::find(target, "nmake.exe")
                .expect("failed to find nmake");
            build.current_dir(&inner_dir);
            self.run_command(build, "building");

            let mut install = cc::windows_registry::find(target, "nmake.exe")
                .expect("failed to find nmake");
            install.arg("install_sw").current_dir(&inner_dir);
            self.run_command(install, "installing");
        } else {
            let mut build = Command::new("make");
            build.current_dir(&inner_dir);
            if !cfg!(windows) {
                if let Some(s) = env::var_os("CARGO_MAKEFLAGS") {
                    build.env("MAKEFLAGS", s);
                }
            }
            self.run_command(build, "building");

            let mut install = Command::new("make");
            install.arg("install").current_dir(&inner_dir);
            self.run_command(install, "installing");
        }

        let libs = if target.contains("msvc") {
            vec!["libzmq".to_string()]
        } else {
            vec!["zmq".to_string()]
        };

        //fs::remove_dir_all(&inner_dir).unwrap();

        Artifacts {
            lib_dir: install_dir.join("lib"),
            include_dir: install_dir.join("include"),
            libs,
        }
    }

    fn run_command(&self, mut command: Command, desc: &str) {
        println!("running {:?}", command);
        let status = command.status().unwrap();
        if !status.success() {
            panic!("


Error {}:
    Command: {:?}
    Exit status: {}


    ",
                   desc,
                   command,
                   status);
        }
    }
}

fn cp_r(src: &Path, dst: &Path) {
    for f in fs::read_dir(src).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();
        let dst = dst.join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}

fn sanitize_sh(path: &Path) -> String {
    if !cfg!(windows) {
        return path.to_str().unwrap().to_string();
    }
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None;
        }
        if ch.next() != Some('/') {
            return None;
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}

impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}
