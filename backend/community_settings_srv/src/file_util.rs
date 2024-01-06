use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const RON_EXTENSION: &'static str = "ron";
pub const JSON_EXTENSION: &'static str = "json";

const SETTING_FOLDER: &'static str = "settings";
const ID_FOLDER: &'static str = "by_id";
const APP_ID_FOLDER: &'static str = "by_app_id";
const USER_ID_FOLDER: &'static str = "by_user_id";
const TAG_FOLDER: &'static str = "by_tag";

static LAST_SETTING_ID: Mutex<u128> = Mutex::new(0);

pub fn build_folder_layout(root: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(
        root.as_ref()
            .join(SETTING_FOLDER)
            .join(ID_FOLDER)
    )?;
    std::fs::create_dir_all(
        root.as_ref()
            .join(SETTING_FOLDER)
            .join(APP_ID_FOLDER)
    )?;
    std::fs::create_dir_all(
        root.as_ref()
            .join(SETTING_FOLDER)
            .join(USER_ID_FOLDER)
    )?;
    std::fs::create_dir_all(
        root.as_ref()
            .join(SETTING_FOLDER)
            .join(TAG_FOLDER)
    )?;
    Ok(())
}

pub fn filename(id: u128, ext: &str) -> String {
    format!("{}.{}", id, ext)
}

pub fn setting_path_by_id(root: impl AsRef<Path>, id: u128, ext: &str) -> PathBuf {
    root.as_ref()
        .join(SETTING_FOLDER)
        .join(ID_FOLDER)
        .join(filename(id, ext))
}

pub fn setting_folder_by_app_id(root: impl AsRef<Path>, steam_app_id: u32) -> PathBuf {
    root.as_ref()
        .join(SETTING_FOLDER)
        .join(APP_ID_FOLDER)
        .join(steam_app_id.to_string())
}

pub fn setting_folder_by_user_id(root: impl AsRef<Path>, steam_user_id: u64) -> PathBuf {
    root.as_ref()
        .join(SETTING_FOLDER)
        .join(USER_ID_FOLDER)
        .join(steam_user_id.to_string())
}

pub fn setting_folder_by_tag(root: impl AsRef<Path>, tag: &str) -> PathBuf {
    root.as_ref()
        .join(SETTING_FOLDER)
        .join(TAG_FOLDER)
        .join(tag)
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
