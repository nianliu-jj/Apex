# Apex 编号注册表

规范权威源：`docs/governance/identifier-registry.json`。本文件说明 EP-0002 的机器可读规则。

## 规则

- `RQ-*`：产品需求；当前主序列为 `RQ-001`–`RQ-124`。废弃需求保留原编号，不得复用。
- `AC-*`：产品验收标准；当前主序列为 `AC-001`–`AC-026`。
- `EP-*`：最小执行单元；按领域号段追加，不填历史空洞、不复用旧编号。当前计划注册 256 个 EP，包含 Active 与 Superseded 生命周期。
- `VAL-*`：独立验证项；当前主序列为 `VAL-01`–`VAL-254`。`VAL-02B` 是保留的追加验证编号，不改变主序列。

## 校验

```text
python scripts/validate_identifier_registry.py
cargo xtask verify identifiers
```

校验器检查：

1. 注册表 ID 唯一。
2. 主序列无断号，所有扩展编号已登记。
3. 执行计划 EP 总表与注册表一致且按数字顺序排列。
4. 设计文档、原子模块系分文档和执行计划引用的 ID 均已登记。

编号采用只追加策略。发现需求拆分、替代或废弃时，更新生命周期和迁移记录，不删除或复用历史编号。
