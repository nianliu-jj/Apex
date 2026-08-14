# EP-0102 target matrix

| OS | Architecture | Rust target |
|---|---|---|
| macOS | x86_64 | `x86_64-apple-darwin` |
| macOS | aarch64 | `aarch64-apple-darwin` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` |
| Windows | aarch64 | `aarch64-pc-windows-msvc` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Linux | aarch64 | `aarch64-unknown-linux-gnu` |

The six targets are the cross-compilation matrix. They are documented here
rather than listed in `rust-toolchain.toml`, so a local Cargo command does not
silently download a platform standard library. CI or release runners install
the target needed by their job before running the corresponding dry-run.
