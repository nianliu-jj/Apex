# EP-0102 target matrix

机器可读事实源：`docs/governance/target-matrix.txt`。

| OS | Architecture | Rust target |
|---|---|---|
| macOS | x86_64 | `x86_64-apple-darwin` |
| macOS | aarch64 | `aarch64-apple-darwin` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` |
| Windows | aarch64 | `aarch64-pc-windows-msvc` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Linux | aarch64 | `aarch64-unknown-linux-gnu` |

`cargo xtask verify targets` 会读取机器可读事实源并验证数量、唯一性和 rustc
识别能力。文档表不再作为程序输入，避免 Markdown 解析耦合。
