# Security Policy

## Supported Versions

DevTiny is currently an early-stage desktop tool. The supported platform for
the first public release is macOS.

## Reporting a Vulnerability

Please open a private security advisory on GitHub if the repository enables
that feature. If private advisories are not available, open an issue with a
minimal description and avoid posting exploit details publicly.

## Safety Model

- The frontend does not send arbitrary shell commands to the backend.
- Runtime actions are mapped to fixed Rust command specifications.
- Commands run in the selected project directory.
- Risky Git, Docker, file deletion, and setup actions require user
  confirmation.
- Deleted files are moved to the system trash on supported platforms.
