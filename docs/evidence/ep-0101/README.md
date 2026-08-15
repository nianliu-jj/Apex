# EP-0101 验证证据

- EP：`EP-0101`
- VAL：`VAL-08`
- 主要交付物：根 `Cargo.toml` workspace 清单与 `xtask` workspace 验证闭环
- 失败 fixture：`xtask/tests/fixtures/missing-member/Cargo.toml`

## 验证范围

- 当前 workspace 通过 `cargo metadata` 成员和路径检查。
- 缺失成员 workspace 被拒绝，并返回包含 Cargo 诊断的可操作错误。
- 验证 fixture 使用显式 `--manifest-path`，不依赖当前进程的 workspace 上下文。

## 可复现命令

```text
cargo xtask verify workspace
cargo check --workspace --all-targets
cargo test --workspace
```

## 失败路径

缺失成员 fixture 声明 `missing-crate`，但未创建对应目录。定向测试确认：

- 验证结果为 `Err`；
- 错误包含 `Cargo workspace validation failed`；
- 错误包含 Cargo 输出中的 `missing-crate`。

## 范围裁决

本 EP 保持最小 workspace 成员集合 `xtask`。下游业务 crate 仅在对应 EP 实现时加入，避免用无行为占位 crate 冒充功能完成。
