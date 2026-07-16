use crate::core::app_error::{AppError, AppResult};
use crate::core::path_guard::{canonical_project_path, validate_relative_path};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredRule {
    pub rule: String,
    pub display_name: String,
    pub mode: String,
}

#[tauri::command]
pub fn list_ignored_rules(project_path: String) -> AppResult<Vec<IgnoredRule>> {
    let project_path = canonical_project_path(&project_path)?;
    let path = project_path.join(".gitignore");
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('!'))
        .map(|rule| IgnoredRule {
            rule: rule.to_string(),
            display_name: display_name(rule),
            mode: if rule.starts_with("*.") {
                "extension"
            } else {
                "file"
            }
            .to_string(),
        })
        .collect())
}

#[tauri::command]
pub fn add_ignored_rule(
    project_path: String,
    relative_path: String,
    mode: String,
) -> AppResult<IgnoredRule> {
    let project_path = canonical_project_path(&project_path)?;
    let relative = validate_relative_path(&relative_path)?
        .to_string_lossy()
        .replace('\\', "/");
    let rule = match mode.as_str() {
        "file" => format!("/{}", escape_pattern(&relative)),
        "extension" => extension_rule(&relative)?,
        _ => {
            return Err(AppError::invalid_input(
                "Ignore mode must be file or extension.",
            ))
        }
    };
    let path = project_path.join(".gitignore");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    if !existing.lines().any(|line| line.trim() == rule) {
        let mut content = existing;
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&rule);
        content.push('\n');
        fs::write(path, content)?;
    }
    Ok(IgnoredRule {
        display_name: display_name(&rule),
        rule,
        mode,
    })
}

#[tauri::command]
pub fn remove_ignored_rule(project_path: String, rule: String) -> AppResult<()> {
    let project_path = canonical_project_path(&project_path)?;
    let rule = rule.trim();
    if rule.is_empty() || rule.contains('\n') || rule.contains('\r') {
        return Err(AppError::invalid_input("Ignore rule is invalid."));
    }
    let path = project_path.join(".gitignore");
    let existing = fs::read_to_string(&path)?;
    let trailing_newline = existing.ends_with('\n');
    let mut lines: Vec<&str> = existing.lines().collect();
    let original_len = lines.len();
    lines.retain(|line| line.trim() != rule);
    if lines.len() == original_len {
        return Err(AppError::invalid_input("Ignore rule no longer exists."));
    }
    let mut content = lines.join("\n");
    if trailing_newline && !content.is_empty() {
        content.push('\n');
    }
    fs::write(path, content)?;
    Ok(())
}

fn extension_rule(relative_path: &str) -> AppResult<String> {
    let name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    let dot = name
        .rfind('.')
        .filter(|index| *index > 0)
        .ok_or_else(|| AppError::invalid_input("Selected file has no extension."))?;
    Ok(format!("*{}", escape_pattern(&name[dot..])))
}

fn escape_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '\\' | ' ' | '#' | '!' | '*' | '?' | '[' | ']') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn display_name(rule: &str) -> String {
    let value = rule.trim_start_matches('/');
    value
        .rsplit('/')
        .next()
        .unwrap_or(value)
        .replace("\\ ", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_extension_rule_from_file_name() {
        assert_eq!(extension_rule("assets/archive.tar.gz").unwrap(), "*.gz");
        assert!(extension_rule("Dockerfile").is_err());
    }

    #[test]
    fn escapes_exact_gitignore_patterns() {
        assert_eq!(escape_pattern("docs/my #note.md"), "docs/my\\ \\#note.md");
        assert_eq!(display_name("/docs/my\\ note.md"), "my note.md");
    }

    #[test]
    fn adds_lists_and_removes_a_rule() {
        let root = std::env::temp_dir().join(format!("devtiny-ignore-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/new file.log"), "log").unwrap();
        fs::write(root.join(".gitignore"), "# existing\ntarget/\n").unwrap();
        let project = root.to_string_lossy().to_string();

        let added =
            add_ignored_rule(project.clone(), "src/new file.log".into(), "file".into()).unwrap();
        assert_eq!(added.rule, "/src/new\\ file.log");
        assert!(list_ignored_rules(project.clone())
            .unwrap()
            .iter()
            .any(|entry| entry.rule == added.rule));
        remove_ignored_rule(project, added.rule).unwrap();
        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(content, "# existing\ntarget/\n");
        fs::remove_dir_all(root).unwrap();
    }
}
