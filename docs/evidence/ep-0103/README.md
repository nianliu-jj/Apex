# EP-0103 验证证据

- EP：`EP-0103`
- VAL：`VAL-10`
- 主要交付物：workspace lint、`deny.toml`、CI quality/audit 门禁和 `xtask verify quality`
- warning 负例：`xtask/tests/fixtures/warning`

## 已配置门禁

- Rust workspace lints：`unsafe_code=deny`、`missing_docs=warn`。
- Clippy：`all=deny`，并显式拒绝 `unwrap_used`、`expect_used`、`panic`、`exit`。
- Cargo deny：advisories、bans、licenses、sources。
- Cargo audit：GitHub Actions 使用 `rustsec/audit-check@v2.0.0`。
- 本地统一质量入口：`cargo xtask verify quality`。

## 可复现命令

```text
cargo xtask verify quality
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings -A missing-docs
cargo test --workspace
cargo deny --offline check
cargo audit --db C:\rust\.cargo\advisory-dbs\advisory-db-3157b0e258782691 --no-fetch --stale
```

CI 的 `audit` job 使用 `rustsec/audit-check@v2.0.0`，在线刷新 advisory database；本地命令
使用已安装的 `cargo-audit 0.22.2` 和本机缓存数据库，不把网络可用性伪装成代码通过。

## VAL-10 负例

warning fixture 启用 `unused_variables = "deny"`，故意保留未使用变量。定向测试
`quality_gate_rejects_warning_fixture` 运行独立 fixture 的 `cargo check`，确认：

- 质量命令失败；
- 错误包含 `warning fixture quality gate failed`；
- 错误包含 `unused variable`。

## 质量结果

本地 `cargo xtask verify quality` 通过；`cargo deny --offline check` 和缓存 advisory
数据库下的 `cargo audit` 通过。`deny.toml` 中尚未命中的许可证白名单只产生 warning，
不影响当前 exit code。
