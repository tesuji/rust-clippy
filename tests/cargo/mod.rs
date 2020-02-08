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