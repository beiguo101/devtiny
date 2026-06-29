# DevTiny

DevTiny is a tiny desktop app for safe, minimal Git and Docker Compose
operations. It is designed for people who need to maintain a local project but
do not want to use terminal commands for routine work.

The app focuses on a small daily workflow:

- Select a local project directory.
- View files and changed files.
- Edit simple text/code files.
- Stage, commit, ignore, and restore selected Git changes.
- Start, stop, restart, and inspect Docker Compose projects.
- Run first-time dependency setup with visible logs.
- Configure project-level Python, Node, and Maven mirror files for Docker
  builds.

## Current Platform Support

DevTiny currently has verified macOS packaging and Linux package build support.

The tested release package is for Apple Silicon macOS:

```text
DevTiny_0.1.0_aarch64.dmg
```

Linux packages can be built on Linux as `.deb`, `.rpm`, and `.AppImage`.

Windows is not supported for the current public release. Some code paths already
include partial Windows-aware command lookup, but Windows packaging and runtime
behavior have not been verified.

## Requirements

To run DevTiny from source, install:

- macOS 11 or later, or a Linux desktop supported by Tauri 2
- Node.js 20 or later
- pnpm 9 or later
- Rust stable toolchain
- Tauri 2 build prerequisites for your OS
- Git
- Docker Desktop on macOS, or Docker Engine on Linux, for Docker Compose
  features

Docker must be running before using Docker actions.

## Install Dependencies

```sh
pnpm install
```

## Run in Development

```sh
pnpm tauri:dev
```

This starts the Vite frontend and the Tauri desktop app.

## Build the Frontend

```sh
pnpm run build
```

## Run Rust Tests

```sh
cd src-tauri
cargo test
```

## Build the macOS App

For the current machine architecture:

```sh
pnpm tauri build
```

Apple Silicon output:

```text
src-tauri/target/release/bundle/macos/DevTiny.app
src-tauri/target/release/bundle/dmg/DevTiny_0.1.0_aarch64.dmg
```

For a universal macOS build, install both Rust targets first:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri build --target universal-apple-darwin
```

DMG creation uses macOS system tooling such as `hdiutil`.

## Build Linux Packages

Linux packages must be built on Linux:

```sh
pnpm tauri:build:linux
```

Expected output:

```text
src-tauri/target/release/bundle/deb/*.deb
src-tauri/target/release/bundle/rpm/*.rpm
src-tauri/target/release/bundle/appimage/*.AppImage
```

The repository includes a GitHub Actions workflow for Linux package artifacts:

```text
.github/workflows/linux-package.yml
```

## Safety Model

DevTiny intentionally avoids arbitrary shell execution from the UI.

- The frontend sends action names and structured payloads.
- Rust maps actions to fixed `program + args` command specifications.
- Commands run with `current_dir` set to the selected project path.
- The app previews supported Git and Docker operations before execution.
- Risky actions require confirmation.
- Deleted files are moved to the system trash where supported.
- Command history is stored locally.

## Repository Contents

Important files:

```text
src/                  Vue frontend
src-tauri/src/        Rust command layer
src-tauri/tauri.conf.json
package.json
pnpm-lock.yaml
src-tauri/Cargo.toml
src-tauri/Cargo.lock
```

Generated files and local caches are intentionally not committed:

```text
node_modules/
dist/
src-tauri/target/
.pnpm-store/
.DS_Store
*.log
```

## License

DevTiny is released under the MIT License. See [LICENSE](./LICENSE).
