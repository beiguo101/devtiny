use crate::core::app_error::{AppError, AppResult};
use std::path::Path;
use std::process::Command;

pub fn move_path_to_trash(path: &Path) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell application \"Finder\" to delete POSIX file \"{}\"",
            escape_applescript_string(&path.to_string_lossy())
        );
        let output = Command::new("osascript").args(["-e", &script]).output()?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::command_failed(format!(
            "Failed to move file to Trash: {}",
            stderr.trim()
        )));
    }

    #[cfg(not(target_os = "macos"))]
    #[cfg(target_os = "linux")]
    {
        for (program, args) in [
            (
                "gio",
                vec!["trash".to_string(), path.to_string_lossy().to_string()],
            ),
            ("trash-put", vec![path.to_string_lossy().to_string()]),
        ] {
            if let Ok(output) = Command::new(program).args(args).output() {
                if output.status.success() {
                    return Ok(());
                }
            }
        }
        Err(AppError::unavailable(
            "No supported system Trash command was found.",
        ))
    }

    #[cfg(target_os = "windows")]
    {
        let escaped = path.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$shell = New-Object -ComObject Shell.Application; $shell.Namespace(10).MoveHere('{}')",
            escaped
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()?;
        if output.status.success() {
            return Ok(());
        }
        Err(AppError::command_failed(
            "Failed to move file to Recycle Bin.",
        ))
    }
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
