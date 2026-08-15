# EP-0102 验证证据

- EP：`EP-0102`
- VAL：`VAL-09`
- 主要交付物：`rust-toolchain.toml` 与 `docs/governance/target-matrix.txt`
- 验证入口：`cargo xtask verify targets`

## 锁定内容

- Rust toolchain：`1.96.1`
- Profile：`minimal`
- Components：`clippy`、`rustfmt`
- Target 数量：6

## VAL-09 dry-run 语义

对矩阵中的每个 target 执行：

```text
rustc --target <target> --print cfg
```

该命令由锁定 toolchain 的 `rustc` 解析 target 并输出目标配置，不要求下载对应
标准库、链接器或在目标硬件上运行。实际交叉编译和实机验证属于后续平台与发布
Gate。

## 可复现命令

```text
cargo xtask verify targets
cargo test -p xtask
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings -A missing-docs
cargo test --workspace
```

## 失败和边界路径

- target 数量不是 6 时拒绝；
- target 重复时拒绝；
- 未知 target 被 `rustc` 拒绝，错误包含 target 名与 Cargo 可操作诊断。
