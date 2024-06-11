use directories::BaseDirs;
use std::path::PathBuf;

fn get_base_directory() -> PathBuf {
    BaseDirs::new()
        .unwrap()
        .home_dir()
        .join(".config")
        .join("lunate")
}

pub fn get_plugin_directory() -> PathBuf {
    get_base_directory().join("plugins")
}
