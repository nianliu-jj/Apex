# EP-0002 验证证据

- EP：`EP-0002`
- VAL：`VAL-02`
- 交付物：`docs/governance/identifier-registry.json`
- 规则说明：`docs/governance/identifier-registry.md`
- 校验器：`scripts/validate_identifier_registry.py`
- 负例测试：`scripts/test_identifier_registry.py`

## 可复现命令

```text
python -m unittest scripts/test_identifier_registry.py
python scripts/validate_identifier_registry.py
cargo xtask verify identifiers
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings -A missing-docs
cargo test --workspace
cargo deny check
```

## 覆盖范围

- 重复编号被拒绝。
- 主序列断号被拒绝。
- 未登记编号被拒绝。
- EP 总表与注册表集合不一致被拒绝。
- 执行计划、设计文档、原子模块系分文档中的正式编号引用必须已登记。
- `RQ-006`、Superseded EP 与 `VAL-02B` 保留历史编号，不得复用。

命令结果以本提交的本地验证输出和 CI 记录为准。
