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
    {
        Err(AppError::unavailable(
            "Moving files to the system Trash is not supported on this platform yet.",
        ))
    }
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
