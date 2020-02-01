use cargo_metadata::Message;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

#[must_use]
fn clippy_driver_path(target_lib: &Path) -> PathBuf {
    if let Some(path) = option_env!("CLIPPY_DRIVER_PATH") {
        PathBuf::from(path)
    } else {
        target_lib.join("clippy-driver")
    }
}

#[must_use]
fn host_libs(target_dir: &Path) -> PathBuf {
    if let Some(path) = option_env!("HOST_LIBS") {
        PathBuf::from(path)
    } else {
        target_dir.join(env!("PROFILE"))
    }
}

#[must_use]
fn target_libs(host_lib: &Path) -> PathBuf {
    if let Some(path) = option_env!("TARGET_LIBS") {
        path.into()
    } else {
        let mut dir = host_lib.to_owned();
        if let Some(target) = env::var_os("CARGO_BUILD_TARGET") {
            dir.push(target);
        }
        dir
    }
}

#[must_use]
pub fn rustc_test_suite() -> Option<PathBuf> {
    option_env!("RUSTC_TEST_SUITE").map(PathBuf::from)
}

#[must_use]
pub fn rustc_lib_path() -> PathBuf {
    option_env!("RUSTC_LIB_PATH").unwrap().into()
}

// When we'll want to use `extern crate ..` for a dependency that is used
// both by the crate and the compiler itself, we can't simply pass -L flags
// as we'll get a duplicate matching versions. Instead, disambiguate with
// `--extern dep=path`.
// See https://github.com/rust-lang/rust-clippy/issues/4015.
fn extern_crates() -> Vec<(&'static str, PathBuf)> {
    let cargo = env::var_os("CARGO");
    let cargo = cargo.as_deref().unwrap_or_else(|| OsStr::new("cargo"));
    let output = Command::new(cargo)
        .arg("build")
        .arg("--test=compile-test")
        .arg("--message-format=json")
        .output()
        .unwrap();

    let needs_disambiguation = ["serde", "regex", "clippy_lints"];

    let mut result = Vec::with_capacity(needs_disambiguation.len());
    for message in cargo_metadata::parse_messages(output.stdout.as_slice()) {
        if let Message::CompilerArtifact(artifact) = message.unwrap() {
            if let Some(&krate) = needs_disambiguation
                .iter()
                .find(|&&krate| krate == artifact.target.name)
            {
                result.push((krate, artifact.filenames[0].clone()));
            }
        }
    }
    result
}

pub struct BuildInfo {
    pub host_lib: PathBuf,
    pub target_lib: PathBuf,
    pub clippy_driver_path: PathBuf,
    pub third_party_crates: Vec<(&'static str, PathBuf)>,
}

impl BuildInfo {
    pub fn new() -> Self {
        let data = cargo_metadata::MetadataCommand::new().exec().unwrap();
        let target_dir = data.target_directory;
        let host_lib = host_libs(&target_dir);
        let target_lib = target_libs(&host_lib);
        let clippy_driver_path = clippy_driver_path(&target_lib);
        let third_party_crates = extern_crates();
        Self {
            host_lib,
            target_lib,
            clippy_driver_path,
            third_party_crates,
        }
    }
}
