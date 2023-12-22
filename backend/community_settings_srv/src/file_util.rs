use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const RON_EXTENSION: &'static str = "ron";
pub const JSON_EXTENSION: &'static str = "json";

const SETTING_FOLDER: &'static str = "settings";
const ID_FOLDER: &'static str = "by_id";

static LAST_SETTING_ID: Mutex<u128> = Mutex::new(0);

pub fn setting_path_by_id(root: impl AsRef<Path>, id: u128, ext: &str) -> PathBuf {
    root.as_ref()
        .join(SETTING_FOLDER)
        .join(ID_FOLDER)
        .join(format!("{}.{}", id, ext))
}

pub fn next_setting_id(root: impl AsRef<Path>) -> u128 {
    let mut lock = LAST_SETTING_ID.lock().unwrap();
    let mut last_id = *lock;
    if last_id == 0 {
        // needs init
        let mut path = setting_path_by_id(root.as_ref(), last_id, RON_EXTENSION);
        while path.exists() {
            last_id += 1;
            path = setting_path_by_id(root.as_ref(), last_id, RON_EXTENSION);
        }
        *lock = last_id;
        println!("setting id initialized to {}", last_id);
    }
    *lock += 1;
    *lock
}
