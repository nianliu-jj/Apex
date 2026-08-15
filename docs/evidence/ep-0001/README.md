# EP-0001 验证证据

- EP：`EP-0001`
- VAL：`VAL-01`
- 主要交付物：Feature Spec 四文档模板与 frontmatter JSON Schema
- 验证入口：`cargo xtask verify specs`

## 交付物

- `specs/_templates/{requirements,design,tasks,verification}.md`
- `schemas/feature-spec-frontmatter.schema.json`
- `schemas/fixtures/spec-frontmatter/valid/`
- `schemas/fixtures/spec-frontmatter/invalid/`
- `scripts/validate_spec_templates.py`

## VAL-01

统一入口校验四类模板和四个正例 fixture 必须通过；四个负例 fixture 必须被拒绝。
当前负例覆盖缺失必填字段、非法状态、空写路径和非法验收模式。

## 可复现命令

```text
cargo xtask verify specs
cargo xtask verify quality
```

## 范围说明

本 EP 只固定模板 frontmatter 与 schema。四阶段 parser、renderer、审批和失效传播由
EP-0401–EP-0407 实现，不在本 EP 提前引入。
