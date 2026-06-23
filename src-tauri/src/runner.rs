use std::{
    process::{Command, Stdio},
    time::Instant,
};

use crate::{
    domain::LaunchCommand,
    error::{AppError, AppResult},
};

pub struct ProcessOutcome {
    pub exit_code: Option<i32>,
    pub duration_seconds: i64,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(command: &LaunchCommand) -> AppResult<ProcessOutcome> {
    let started = Instant::now();
    let mut process = Command::new(&command.executable);
    process
        .args(&command.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(directory) = &command.working_directory {
        process.current_dir(directory);
    }
    let output = process.spawn()?.wait_with_output()?;
    if !output.status.success() && output.status.code().is_none() {
        return Err(AppError::InvalidInput(
            "Proces emulátora bol násilne ukončený.".into(),
        ));
    }
    Ok(ProcessOutcome {
        exit_code: output.status.code(),
        duration_seconds: started.elapsed().as_secs() as i64,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn launches_and_tracks_a_fake_emulator() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("fake_emulator.rs");
        let executable = directory.path().join("fake_emulator.exe");
        fs::write(
            &source,
            r#"fn main() {
                let args: Vec<String> = std::env::args().skip(1).collect();
                println!("fake-emulator:{}", args.join("|"));
            }"#,
        )
        .unwrap();
        let status = Command::new("rustc")
            .args([source.to_str().unwrap(), "-o", executable.to_str().unwrap()])
            .status()
            .unwrap();
        assert!(status.success());
        let outcome = run(&LaunchCommand {
            executable,
            args: vec!["-L".into(), "core.dll".into(), "fixture.nes".into()],
            working_directory: Some(directory.path().to_path_buf()),
        })
        .unwrap();
        assert_eq!(outcome.exit_code, Some(0));
        assert!(outcome.stdout.contains("core.dll|fixture.nes"));
        assert!(outcome.stderr.is_empty());
    }
}
