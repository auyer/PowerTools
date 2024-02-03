use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

pub fn dump_sys_info() -> Result<(), ()> {
    let (tx, rx) = channel();
    let mut join_handles = Vec::new();
    let useful_files = vec![
        PathBuf::from("/proc/ioports"),
        PathBuf::from("/proc/cpuinfo"),
        PathBuf::from("/etc/os-release"),
    ];
    for file in useful_files {
        join_handles.push(read_file(file, tx.clone()));
    }

    let useful_commands = vec!["dmidecode"];
    for cmd in useful_commands.into_iter() {
        join_handles.push(execute_command(cmd, tx.clone()));
    }

    for join_handle in join_handles.into_iter() {
        if let Err(e) = join_handle.join() {
            log::error!("Thread failed to complete: {:?}", e);
        }
    }

    let mut dump_file =
        std::fs::File::create("powertools_sys_dump.txt").expect("Failed to create dump file");
    for response in rx.into_iter() {
        dump_file
            .write(
                &format!(
                    "{} v{} ###### {} ######\n{}\n",
                    crate::consts::PACKAGE_NAME,
                    crate::consts::PACKAGE_VERSION,
                    response.0,
                    response.1.unwrap_or("[None]".to_owned())
                )
                .into_bytes(),
            )
            .expect("Failed to write to dump file");
    }
    Ok(())
}

fn read_file(
    file: impl AsRef<Path> + Send + 'static,
    tx: Sender<(String, Option<String>)>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let file = file.as_ref();
        tx.send((
            file.display().to_string(),
            std::fs::read_to_string(file).ok(),
        ))
        .expect("Failed to send file contents");
    })
}

fn execute_command(command: &'static str, tx: Sender<(String, Option<String>)>) -> JoinHandle<()> {
    thread::spawn(move || {
        tx.send((
            command.to_owned(),
            Command::new(command)
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
                .ok(),
        ))
        .expect("Failed to send command output");
    })
}
