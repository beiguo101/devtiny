use crate::core::app_error::{AppError, AppResult};
use crate::core::path_guard::canonical_project_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const BLOCK_START: &str = "# DevTiny China mirrors start";
const BLOCK_END: &str = "# DevTiny China mirrors end";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureMirrorsRequest {
    pub project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureMirrorsResult {
    pub files: Vec<String>,
    pub dockerfile_path: String,
    pub message: String,
}

#[tauri::command]
pub fn configure_project_mirrors(
    request: ConfigureMirrorsRequest,
) -> AppResult<ConfigureMirrorsResult> {
    let project_path = canonical_project_path(&request.project_path)?;
    let dockerfile_path = find_dockerfile(&project_path).ok_or_else(|| {
        AppError::invalid_input("No Dockerfile found in the selected project root.")
    })?;

    let mut files = write_mirror_files(&project_path)?;
    patch_dockerfile(&dockerfile_path)?;
    ensure_dockerignore_allows_mirrors(&project_path)?;

    files.push(relative(&project_path, &dockerfile_path));
    files.push(".dockerignore".into());

    Ok(ConfigureMirrorsResult {
        dockerfile_path: relative(&project_path, &dockerfile_path),
        files,
        message: "Python, Node, and Maven mirror configuration has been written for Docker builds."
            .into(),
    })
}

fn find_dockerfile(project_path: &Path) -> Option<PathBuf> {
    ["Dockerfile", "dockerfile"]
        .iter()
        .map(|name| project_path.join(name))
        .find(|path| path.is_file())
}

fn write_mirror_files(project_path: &Path) -> AppResult<Vec<String>> {
    let mirror_dir = project_path.join(".devtiny").join("mirrors");
    fs::create_dir_all(&mirror_dir)?;

    let pip_conf = mirror_dir.join("pip.conf");
    fs::write(
        &pip_conf,
        "[global]\nindex-url = https://mirrors.aliyun.com/pypi/simple/\ntrusted-host = mirrors.aliyun.com\ntimeout = 120\n",
    )?;

    let npmrc = mirror_dir.join("npmrc");
    fs::write(&npmrc, "registry=https://registry.npmmirror.com\n")?;

    let maven_settings = mirror_dir.join("maven-settings.xml");
    fs::write(&maven_settings, maven_settings_xml())?;

    Ok(vec![
        relative(project_path, &pip_conf),
        relative(project_path, &npmrc),
        relative(project_path, &maven_settings),
    ])
}

fn patch_dockerfile(dockerfile_path: &Path) -> AppResult<()> {
    let original = fs::read_to_string(dockerfile_path)?;
    let without_existing = remove_existing_blocks(&original);
    let mut output = String::new();
    let mut inserted = false;

    for line in without_existing.lines() {
        output.push_str(line);
        output.push('\n');
        if line.trim_start().to_ascii_lowercase().starts_with("from ") {
            output.push_str(dockerfile_block());
            inserted = true;
        }
    }

    if !inserted {
        return Err(AppError::invalid_input(
            "Dockerfile does not contain a FROM instruction.",
        ));
    }

    fs::write(dockerfile_path, output)?;
    Ok(())
}

fn remove_existing_blocks(content: &str) -> String {
    let mut output = Vec::new();
    let mut in_block = false;

    for line in content.lines() {
        if line.trim() == BLOCK_START {
            in_block = true;
            continue;
        }
        if line.trim() == BLOCK_END {
            in_block = false;
            continue;
        }
        if !in_block {
            output.push(line);
        }
    }

    output.join("\n")
}

fn dockerfile_block() -> &'static str {
    r#"# DevTiny China mirrors start
ARG DEVTINY_USE_CN_MIRRORS=1
ENV PIP_INDEX_URL=https://mirrors.aliyun.com/pypi/simple/ \
    PIP_TRUSTED_HOST=mirrors.aliyun.com \
    npm_config_registry=https://registry.npmmirror.com
RUN if [ "$DEVTINY_USE_CN_MIRRORS" = "1" ]; then mkdir -p /root/.pip /root/.m2; fi
COPY .devtiny/mirrors/pip.conf /root/.pip/pip.conf
COPY .devtiny/mirrors/npmrc /root/.npmrc
COPY .devtiny/mirrors/maven-settings.xml /root/.m2/settings.xml
# DevTiny China mirrors end
"#
}

fn ensure_dockerignore_allows_mirrors(project_path: &Path) -> AppResult<()> {
    let dockerignore_path = project_path.join(".dockerignore");
    let mut content = fs::read_to_string(&dockerignore_path).unwrap_or_default();
    let required = ["!.devtiny", "!.devtiny/mirrors", "!.devtiny/mirrors/*"];

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    let mut changed = false;
    for line in required {
        if !content.lines().any(|existing| existing.trim() == line) {
            content.push_str(line);
            content.push('\n');
            changed = true;
        }
    }

    if changed || dockerignore_path.exists() {
        fs::write(dockerignore_path, content)?;
    }

    Ok(())
}

fn maven_settings_xml() -> &'static str {
    r#"<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0 https://maven.apache.org/xsd/settings-1.0.0.xsd">
  <mirrors>
    <mirror>
      <id>aliyunmaven</id>
      <mirrorOf>*</mirrorOf>
      <name>Aliyun Maven</name>
      <url>https://maven.aliyun.com/repository/public</url>
    </mirror>
  </mirrors>
</settings>
"#
}

fn relative(project_path: &Path, path: &Path) -> String {
    path.strip_prefix(project_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
