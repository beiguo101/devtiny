# Linux Packages

Linux packages are built on Linux with Tauri 2.

## Build

```sh
pnpm tauri:build:linux
```

Expected output:

```text
src-tauri/target/release/bundle/deb/*.deb
src-tauri/target/release/bundle/rpm/*.rpm
src-tauri/target/release/bundle/appimage/*.AppImage
```

The GitHub Actions workflow at `.github/workflows/linux-package.yml` builds the
same packages on Ubuntu 22.04 and uploads them as workflow artifacts.

Docker features require Docker to be installed and running. Git features require
Git to be installed on the local machine.
