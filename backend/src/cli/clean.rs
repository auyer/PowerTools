pub fn clean_up() -> Result<(), ()> {
    let dirs = vec![
        crate::utility::settings_dir_old(),
        crate::utility::settings_dir(),
    ];

    if let Err(e) = clean_up_io(dirs.iter()) {
        log::error!("Error removing directories: {}", e);
        Err(())
    } else {
        Ok(())
    }
}

fn clean_up_io(
    directories: impl Iterator<Item = impl AsRef<std::path::Path>>,
) -> std::io::Result<()> {
    let results = directories.map(|dir| std::fs::remove_dir_all(dir));
    for res in results {
        res?;
    }
    Ok(())
}
