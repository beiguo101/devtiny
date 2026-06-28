# Contributing

Thanks for your interest in DevTiny.

## Local Checks

Before opening a pull request, run:

```sh
pnpm run build
cd src-tauri
cargo test
```

## Scope

DevTiny is intentionally small. Changes should preserve these constraints:

- Keep the interface understandable for non-software professionals.
- Prefer fixed, reviewed command actions over free-form shell command input.
- Require confirmation for risky operations.
- Move deleted files to the system trash when the platform supports it.
- Keep Docker and Git workflows focused on common daily maintenance tasks.

## Platform Support

The current supported platform is macOS. Apple Silicon builds are tested. Intel
macOS, Windows, and Linux support may require additional verification and
platform-specific fixes.
