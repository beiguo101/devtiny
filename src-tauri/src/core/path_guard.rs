use crate::core::app_error::{AppError, AppResult};
use std::path::{Component, Path, PathBuf};

pub fn canonical_project_path(project_path: &str) -> AppResult<PathBuf> {
    let trimmed = project_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input("Project path is required."));
    }

    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(AppError::invalid_input("Project path does not exist."));
    }
    if !path.is_dir() {
        return Err(AppError::invalid_input("Project path must be a directory."));
    }

    Ok(path.canonicalize()?)
}

pub fn validate_relative_path(relative_path: &str) -> AppResult<PathBuf> {
    let trimmed = relative_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input("Relative path is required."));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(AppError::invalid_input("Absolute paths are not allowed."));
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(AppError::invalid_input(
                    "Parent path segments are not allowed.",
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::invalid_input("Absolute paths are not allowed."));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(AppError::invalid_input("Relative path is required."));
    }

    Ok(normalized)
}

pub fn resolve_existing_path(project_path: &Path, relative_path: &str) -> AppResult<PathBuf> {
    let relative = validate_relative_path(relative_path)?;
    let full_path = project_path.join(relative);
    if !full_path.exists() {
        return Err(AppError::invalid_input("Path does not exist."));
    }
    let canonical = full_path.canonicalize()?;
    ensure_inside(project_path, &canonical)?;
    Ok(canonical)
}

pub fn resolve_parent_checked(project_path: &Path, relative_path: &str) -> AppResult<PathBuf> {
    let relative = validate_relative_path(relative_path)?;
    let full_path = project_path.join(&relative);
    if full_path.exists() {
        let canonical = full_path.canonicalize()?;
        ensure_inside(project_path, &canonical)?;
        return Ok(canonical);
    }

    let parent = full_path
        .parent()
        .ok_or_else(|| AppError::invalid_input("Path parent is invalid."))?;
    if !parent.exists() {
        return Err(AppError::invalid_input("Path parent does not exist."));
    }
    let canonical_parent = parent.canonicalize()?;
    ensure_inside(project_path, &canonical_parent)?;
    Ok(full_path)
}

pub fn ensure_inside(project_path: &Path, candidate: &Path) -> AppResult<()> {
    let canonical_project = project_path.canonicalize()?;
    if candidate.starts_with(&canonical_project) {
        Ok(())
    } else {
        Err(AppError::invalid_input(
            "Resolved path is outside the selected project.",
        ))
    }
}

pub fn relative_to_project(project_path: &Path, path: &Path) -> AppResult<String> {
    let relative = path
        .strip_prefix(project_path)
        .map_err(|_| AppError::invalid_input("Path is outside project."))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_and_parent_paths() {
        assert!(validate_relative_path("/tmp/a").is_err());
        assert!(validate_relative_path("../a").is_err());
        assert!(validate_relative_path("a/../../b").is_err());
    }

    #[test]
    fn accepts_normal_relative_path() {
        assert_eq!(
            validate_relative_path("./src/main.ts")
                .expect("valid relative path")
                .to_string_lossy(),
            "src/main.ts"
        );
    }
}
