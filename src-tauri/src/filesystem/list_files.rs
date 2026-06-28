use crate::core::app_error::AppResult;
use crate::core::path_guard::{canonical_project_path, relative_to_project, resolve_existing_path};
use crate::git::git_status::{collect_git_status_map, GitFileStatus};
use serde::Serialize;
use std::fs;
use std::path::Path;

const MAX_TREE_DEPTH: usize = 8;
const MAX_TREE_ENTRIES: usize = 5000;
const IGNORED_DIRS: [&str; 8] = [
    ".git",
    "node_modules",
    "target",
    "dist",
    ".next",
    ".tauri",
    ".DS_Store",
    "__pycache__",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTreeNode {
    pub name: String,
    pub relative_path: String,
    pub kind: FileNodeKind,
    pub status: GitFileStatus,
    pub children: Vec<FileTreeNode>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileNodeKind {
    Directory,
    File,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorySummary {
    pub relative_path: String,
    pub directories: u32,
    pub files: u32,
}

#[tauri::command]
pub fn list_project_files(project_path: String) -> AppResult<Vec<FileTreeNode>> {
    let project_path = canonical_project_path(&project_path)?;
    let status_map = collect_git_status_map(&project_path).unwrap_or_default();
    let mut entries_seen = 0usize;
    read_directory(
        &project_path,
        &project_path,
        0,
        &mut entries_seen,
        &status_map,
    )
}

#[tauri::command]
pub fn get_directory_summary(
    project_path: String,
    relative_path: Option<String>,
) -> AppResult<DirectorySummary> {
    let project_path = canonical_project_path(&project_path)?;
    let directory = match relative_path.as_deref().filter(|value| !value.is_empty()) {
        Some(relative_path) => resolve_existing_path(&project_path, relative_path)?,
        None => project_path.clone(),
    };

    let mut directories = 0u32;
    let mut files = 0u32;
    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            directories += 1;
        } else if file_type.is_file() {
            files += 1;
        }
    }

    Ok(DirectorySummary {
        relative_path: if directory == project_path {
            String::new()
        } else {
            relative_to_project(&project_path, &directory)?
        },
        directories,
        files,
    })
}

fn read_directory(
    project_path: &Path,
    directory: &Path,
    depth: usize,
    entries_seen: &mut usize,
    status_map: &std::collections::HashMap<String, GitFileStatus>,
) -> AppResult<Vec<FileTreeNode>> {
    if depth > MAX_TREE_DEPTH || *entries_seen >= MAX_TREE_ENTRIES {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if should_ignore(&file_name) {
            continue;
        }

        let path = entry.path();
        let file_type = entry.file_type()?;
        let is_directory = file_type.is_dir();
        let is_file = file_type.is_file();
        if !is_directory && !is_file {
            continue;
        }

        *entries_seen += 1;
        let relative_path = relative_to_project(project_path, &path)?;
        let children = if is_directory {
            read_directory(project_path, &path, depth + 1, entries_seen, status_map)?
        } else {
            Vec::new()
        };
        let status = status_map
            .get(&relative_path)
            .cloned()
            .unwrap_or(GitFileStatus::Clean);

        entries.push(FileTreeNode {
            name: file_name,
            relative_path,
            kind: if is_directory {
                FileNodeKind::Directory
            } else {
                FileNodeKind::File
            },
            status,
            children,
        });
    }

    entries.sort_by(|a, b| match (&a.kind, &b.kind) {
        (FileNodeKind::Directory, FileNodeKind::File) => std::cmp::Ordering::Less,
        (FileNodeKind::File, FileNodeKind::Directory) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

fn should_ignore(file_name: &str) -> bool {
    IGNORED_DIRS.contains(&file_name)
}
