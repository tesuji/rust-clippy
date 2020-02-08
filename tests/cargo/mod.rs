use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;

lazy_static! {
    static ref CARGO_TARGET_DIR: PathBuf = {
        match env::var_os("CARGO_TARGET_DIR") {
            Some(v) => v.into(),
            None => "target".into(),
        }
    };

    pub static ref TARGET_LIB: PathBuf = {
        if let Some(path) = option_env!("TARGET_LIBS") {
            path.into()
        } else {
            let mut dir = CARGO_TARGET_DIR.clone();
            if let Some(target) = env::var_os("CARGO_BUILD_TARGET") {
                dir.push(target);
            }
            dir.push(env!("PROFILE"));
            dir
        }
    };
}

#[must_use]
pub fn is_rustc_test_suite() -> bool {
    option_env!("RUSTC_TEST_SUITE").is_some()
}

// When we'll want to use `extern crate ..` for a dependency that is used
// both by the crate and the compiler itself, we can't simply pass -L flags
// as we'll get a duplicate matching versions. Instead, disambiguate with
// `--extern dep=path`.
// See https://github.com/rust-lang/rust-clippy/issues/4015.
pub fn third_party_crates() -> Vec<(String, PathBuf)> {
    let cargo = env::var_os("CARGO");
    let cargo = cargo.as_deref().unwrap_or_else(|| OsStr::new("cargo"));
    let output = Command::new(cargo)
        .arg("build")
        .arg("--test=compile-test")
        .arg("--message-format=json")
        .output()
        .unwrap();

    let mut crates = Vec::new();
    for message in cargo_metadata::parse_messages(output.stdout.as_slice()) {
        if let CompilerArtifact(mut artifact) = message.unwrap() {
            if ["lib"] == artifact.target.kind.as_slice() {
                crates.push((artifact.target.name, mem::take(&mut artifact.filenames[0])));
            }
        }
    }
    crates
}
