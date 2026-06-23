use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    domain::{EmulatorStatus, LaunchCommand},
    error::{AppError, AppResult},
};

pub trait EmulatorAdapter {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn executable_names(&self) -> &'static [&'static str];
    fn supported_systems(&self) -> &'static [&'static str];
    fn build_launch_command(
        &self,
        executable: &Path,
        game: &Path,
        core: Option<&Path>,
    ) -> AppResult<LaunchCommand>;

    fn detect_installation(&self) -> Option<PathBuf> {
        let mut roots = vec![];
        for key in [
            "ProgramFiles",
            "ProgramFiles(x86)",
            "LOCALAPPDATA",
            "APPDATA",
        ] {
            if let Some(value) = env::var_os(key) {
                roots.push(PathBuf::from(value));
            }
        }
        roots.push(dirs::home_dir()?.join("Applications"));
        for root in roots {
            for name in self.executable_names() {
                let direct = root.join(self.display_name()).join(name);
                if direct.is_file() {
                    return Some(direct);
                }
            }
        }
        None
    }
}

struct StandardAdapter {
    id: &'static str,
    name: &'static str,
    executables: &'static [&'static str],
    systems: &'static [&'static str],
}

impl EmulatorAdapter for StandardAdapter {
    fn id(&self) -> &'static str {
        self.id
    }
    fn display_name(&self) -> &'static str {
        self.name
    }
    fn executable_names(&self) -> &'static [&'static str] {
        self.executables
    }
    fn supported_systems(&self) -> &'static [&'static str] {
        self.systems
    }
    fn build_launch_command(
        &self,
        executable: &Path,
        game: &Path,
        core: Option<&Path>,
    ) -> AppResult<LaunchCommand> {
        if !executable.is_file() {
            return Err(AppError::NotFound(executable.display().to_string()));
        }
        if !game.exists() {
            return Err(AppError::NotFound(game.display().to_string()));
        }
        let mut args = Vec::new();
        match self.id {
            "retroarch" => {
                let core = core.ok_or_else(|| {
                    AppError::InvalidInput("RetroArch vyžaduje zvolené jadro".into())
                })?;
                args.extend([
                    "-L".into(),
                    core.display().to_string(),
                    game.display().to_string(),
                ]);
            }
            "pcsx2" => args.extend(["-fullscreen".into(), game.display().to_string()]),
            "dolphin" => args.extend([
                "--batch".into(),
                "--exec".into(),
                game.display().to_string(),
            ]),
            "ppsspp" => args.extend(["--fullscreen".into(), game.display().to_string()]),
            "duckstation" => args.extend([
                "-batch".into(),
                "-fullscreen".into(),
                game.display().to_string(),
            ]),
            "rpcs3" | "cemu" => args.push(game.display().to_string()),
            _ => return Err(AppError::InvalidInput("Neznámy adaptér".into())),
        }
        Ok(LaunchCommand {
            executable: executable.to_path_buf(),
            args,
            working_directory: executable.parent().map(Path::to_path_buf),
        })
    }
}

pub fn adapters() -> Vec<Box<dyn EmulatorAdapter + Send + Sync>> {
    vec![
        Box::new(StandardAdapter {
            id: "retroarch",
            name: "RetroArch",
            executables: &["retroarch.exe"],
            systems: &[
                "NES", "SNES", "GB", "GBC", "GBA", "N64", "NDS", "Sega", "Atari", "Arcade", "DOS",
                "ScummVM", "C64", "Amiga", "MSX",
            ],
        }),
        Box::new(StandardAdapter {
            id: "pcsx2",
            name: "PCSX2",
            executables: &["pcsx2-qt.exe", "pcsx2.exe"],
            systems: &["PlayStation 2"],
        }),
        Box::new(StandardAdapter {
            id: "dolphin",
            name: "Dolphin",
            executables: &["Dolphin.exe"],
            systems: &["GameCube", "Wii"],
        }),
        Box::new(StandardAdapter {
            id: "ppsspp",
            name: "PPSSPP",
            executables: &["PPSSPPWindows64.exe"],
            systems: &["PSP"],
        }),
        Box::new(StandardAdapter {
            id: "duckstation",
            name: "DuckStation",
            executables: &[
                "duckstation-qt-x64-ReleaseLTCG.exe",
                "duckstation-qt-x64-Release.exe",
            ],
            systems: &["PlayStation"],
        }),
        Box::new(StandardAdapter {
            id: "rpcs3",
            name: "RPCS3",
            executables: &["rpcs3.exe"],
            systems: &["PlayStation 3"],
        }),
        Box::new(StandardAdapter {
            id: "cemu",
            name: "Cemu",
            executables: &["Cemu.exe"],
            systems: &["Wii U"],
        }),
    ]
}

pub fn statuses() -> Vec<EmulatorStatus> {
    adapters()
        .into_iter()
        .map(|adapter| {
            let installation = adapter.detect_installation();
            EmulatorStatus {
                id: adapter.id().into(),
                display_name: adapter.display_name().into(),
                state: if installation.is_some() {
                    "detected"
                } else {
                    "not-installed"
                }
                .into(),
                version: None,
                executable: installation.map(|path| path.display().to_string()),
                supported_systems: adapter
                    .supported_systems()
                    .iter()
                    .map(|value| (*value).into())
                    .collect(),
            }
        })
        .collect()
}

pub fn adapter(id: &str) -> AppResult<Box<dyn EmulatorAdapter + Send + Sync>> {
    adapters()
        .into_iter()
        .find(|item| item.id() == id)
        .ok_or_else(|| AppError::InvalidInput(format!("Neznámy emulátor: {id}")))
}

pub fn validate_executable(id: &str, path: &Path) -> AppResult<PathBuf> {
    let adapter = adapter(id)?;
    let canonical = path
        .canonicalize()
        .map_err(|_| AppError::NotFound(path.display().to_string()))?;
    let filename = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !adapter
        .executable_names()
        .iter()
        .any(|expected| expected.eq_ignore_ascii_case(filename))
    {
        return Err(AppError::InvalidInput(format!(
            "Súbor {filename} nie je platný executable pre {}",
            adapter.display_name()
        )));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn launch_arguments_are_separate_and_safe() {
        let directory = tempdir().unwrap();
        let executable = directory.path().join("pcsx2-qt.exe");
        let game = directory.path().join("My Game.iso");
        fs::write(&executable, b"fixture").unwrap();
        fs::write(&game, b"fixture").unwrap();
        let adapter = adapters()
            .into_iter()
            .find(|item| item.id() == "pcsx2")
            .unwrap();
        let command = adapter
            .build_launch_command(&executable, &game, None)
            .unwrap();
        assert_eq!(command.args, vec!["-fullscreen", game.to_str().unwrap()]);
        assert!(!command.args.iter().any(|arg| arg.contains("cmd.exe")));
    }
}
