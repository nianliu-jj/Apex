# EP-0001 验证记录

## 结果

- EP：`EP-0001`
- VAL：`VAL-01`
- 状态：本地验证通过
- 验证日期：2026-08-15

## 验证项

| 验证项 | 命令/测试 | 结果 |
|---|---|---|
| 模板与 schema 正例 | `cargo xtask verify specs` | 4 个模板与 4 个正例通过 |
| schema 负例 | `cargo xtask verify specs` | 4 个非法 fixture 均被拒绝 |
| xtask 定向测试 | `validates_feature_spec_templates` | 通过 |
| Workspace 质量门 | `cargo xtask verify quality` | 通过 |

## CI

`.github/workflows/quality.yml` 的 `specs` job 在 push 和 pull request 时执行
`cargo xtask verify specs`，防止模板、schema 与 fixture 漂移。
