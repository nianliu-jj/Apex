# Apex 编码规范

> 本规范是 Apex 项目编码阶段必须遵守的规则。执行机制见 `docs/08-spec-rules-verification.md §7`（三层编码规范强制），质量阈值见 `docs/15-quality-risks-roadmap.md §6.2`（覆盖率）。
>
> 规则等级：**【必须】** 违反将在 Code Review 中驳回（部分由 clippy deny 强制）；**【推荐】** 理应采用，例外需注释说明；**【可选】** 自行决定。
>
> 参考来源（项目无关的通用规则）：
> - [Rust 编程代码规范指南](https://yccoding.com/pages/rust-style-guide/)
> - [TypeScript 编程代码规范指南](https://yccoding.com/pages/ts-style-guide/)
> - [项目代码提交规范](https://yccoding.com/pages/100c7a/)

## 0. 规范权威顺序

冲突时按以下顺序处理（与 `../../../README.md` 一致）：

1. 本文件与 `rules/git-commit.md` —— 工程执行规则。
2. `docs/01-requirements.md` —— 产品范围与验收事实。
3. `docs/02-system-architecture.md`、`docs/03-workspace-and-crates.md` —— 依赖方向与 crate 职责。
4. `docs/05-trait-contracts.md`、`docs/06-protocol-and-clients.md` —— Trait 与 Wire 契约。

**禁止**：在下层文档中静默覆盖上层契约。若本规范与 `../../../Cargo.toml` lint 配置或 CI 冲突，以 CI 实际执行为准并立即修正文档。

---

## 1. Rust 编码规范

### 1.1 工具链与版本 【必须】

- Edition：**2024**（`workspace.package.edition = "2024"`）。
- 工具链：**1.96.1**（`../../../rust-toolchain.toml` 锁定），组件 `clippy` + `rustfmt`。
- `../../../Cargo.lock` 必须提交（应用型 workspace）。
- 所有依赖集中于根 `Cargo.toml [workspace.dependencies]`，成员 crate 只写 `dep.workspace = true`。

### 1.2 命名 【必须】

| 对象 | 规则 | 示例 |
|---|---|---|
| 模块/文件名 | snake_case | `session_runtime`, `tool_gateway` |
| 类型/结构体/枚举/trait | CamelCase | `SessionId`, `ToolGateway`, `PermissionDecision` |
| 函数/方法 | snake_case | `create_checkpoint()`, `resolve_permission` |
| 常量/静态 | SCREAMING_SNAKE | `MAX_TURN_TOKENS`, `DEFAULT_TIMEOUT` |
| 泛型参数 | 单大写或大驼峰 | `T`, `Item`, `Event` |
| 生命周期 | 短小写字母 | `'a`, `'ctx`, `'de` |
| crate 名 | snake_case，词间 `-` | `apex-permission`, `apex-dag` |
| 构造器 | `new()` 为主，`with_`/`from_` 辅助 | `Checkpoint::new()`, `Config::from_env()` |

转换方法约定：`as_` 廉价借用转换；`to_` 昂贵值转换；`into_` 消耗所有权转换。

反模式（禁止）：

- `get_` 前缀滥用 —— 简单字段访问直接 `name()`；`get_` 暗示有副作用或开销。
- 过度缩写 `usr_svc`、`cfg`；含义不清 `data`、`info`、`tmp`；拼音命名；冗余前缀 `MyUser`/`ImplService`。

### 1.3 格式 【必须】

由 `cargo fmt --all -- --check` 在 CI 强制执行。要点：

- 4 空格缩进；行宽 ≤ 100；左大括号不换行。
- `use` 分三组，组间空行：`std` → 第三方 crate → `crate::`/`self::`/`super::`。
- 链式调用以 `.` 开头换行；参数过多时逐参数换行。
- 不手工调整格式，全部交给 `cargo fmt`。

### 1.4 注释与文档 【必须】

- 注释解释"为什么"，代码说明"做了什么"。
- 公开 API 必须有 `///` 文档注释；模块级用 `//!`。`missing_docs` 当前为 `warn`，待补全后收紧为 `deny`（见根 `../../../Cargo.toml` 注释）。
- 文档注释中的代码示例必须可编译（`cargo test --doc`）。
- `TODO(名字): 内容` / `FIXME(名字): 内容` / `HACK(名字): 内容` 必须署名并关联 issue/编号。
- `unsafe` 块必须配 `// SAFETY:` 注释说明不变量。

### 1.5 所有权与借用 【必须】

- 优先借用 `&T` / `&mut T`；确需所有权时才按值接收。
- **`.clone()` 是 Code Review 红色警报**：只读访问用借用；确需独立副本、闭包 `move` 捕获或性能证明非瓶颈时才允许。
- 智能指针选型：`Box<T>` 递归/trait 对象；`Arc<T>` 多线程共享；`Rc<T>` 单线程；`Cow<'a, T>` 写时复制。**禁止** `Rc` 跨线程。
- 不为绕过编译器而使用 `RefCell` —— 重新审视所有权设计。

### 1.6 类型系统 【推荐】

- 字段默认私有，通过方法访问；热路径字段在前。
- 用枚举替代多个 `Option` 字段的非法组合（如 `enum Response { Success(Json), Error(String) }`）。
- 错误类型用 `thiserror` 派生（库）；应用/二进制可用 `anyhow`。**库不得暴露 `anyhow`**。
- newtype 模式做类型安全包装：`struct SessionId(Uuid)` 不与 `AgentId(Uuid)` 混淆。
- trait 小而精；泛型约束用 `where` 子句；能用 `impl Trait` 简化就不写显式泛型参数。

### 1.7 错误处理 【必须】

由 `workspace.lints.clippy` 强制执行（当前已 deny）：

```toml
unwrap_used = "deny"
expect_used = "deny"
panic       = "deny"
exit        = "deny"
```

- 生产代码**禁止** `unwrap()` / `expect()` / `panic!` / `std::process::exit()`。
- 可恢复错误返回 `Result<T, E>`，用 `?` 传播。
- 对外 API 永不 panic；不可恢复的内部错误用 `unreachable!()` / `debug_assert!()`。
- `Option` → `Result` 用 `.ok_or(...)` 或 `.ok_or_else(...)`。
- 测试代码通过模块级 `#[allow(clippy::unwrap_used, ...)]` 豁免（见各 crate `src/tests.rs` 模式），豁免范围不得扩散到非测试模块。

### 1.8 函数 【推荐】

- 函数 ≤ 40 行，做一件事；嵌套 ≤ 4 层，用卫语句/`let-else` 扁平化。
- 迭代器优先于手写 `for` + `push`；及早返回用 `any`/`find`/`first`。
- 闭包 ≤ 5 行，更长则提取为具名函数。

### 1.9 模式匹配 【必须】

- `match` 必须穷举；单分支用 `if let`；提前返回用 `let-else`（Rust 1.65+）。
- 布尔匹配用 `matches!` 宏替代 `if let ... { true } else { false }`。

### 1.10 模块与可见性 【必须】

- 模块/字段/方法默认私有，按需 `pub`；crate 内部协作用 `pub(crate)`。
- 对外隐藏内部结构：`lib.rs` 用 `pub use` 重导出公开 API。
- 依赖方向硬规则（`docs/03 §3/§8`，违反即驳回）：
  - `apex-domain` 不依赖 Tokio/SQLx/Tonic/Actix/Tauri/Provider SDK。
  - `apex-ports` 只定义 Trait，不含具体实现。
  - 应用能力 crate 不依赖具体 Adapter；`apexd` 是唯一组合根。
  - 领域层不得导入生成的 Protobuf 类型。
  - `apex-permission` 禁止依赖 Provider/LLM。
  - Provider/MCP/Plugin/Shell 类型不得越过 Adapter 泄漏到领域事件。

### 1.11 并发与异步 【必须】

- 多线程共享用 `Arc`；单线程才用 `Rc`。
- **锁内禁止 `.await`**（死锁风险）：先 `drop` 锁再 `.await`，或用 `tokio::sync::Mutex`。
- 异步中禁止 `std::thread::sleep`，用 `tokio::time::sleep`。
- Channel 优先于共享内存。
- `unsafe` 默认 workspace deny；仅 `apex-platform`/`apex-plugin-api`/loader 可局部 `#[allow]`，且必须 `SAFETY` 注释 + Miri/平台测试。

### 1.12 测试 【必须】

覆盖率阈值（`docs/15 §6.2`，CI 门禁）：

- `apex-permission`、`apex-dag`、`apex-spec`、Checkpoint/恢复相关：**行/分支 ≥ 90%**。
- 其他 Rust crate：**行 ≥ 80%**，关键状态机要求分支阈值。
- FFI/unsafe、补偿、UnknownSideEffect、Schema migration、Secret Firewall 必须有显式测试，不接受"难以覆盖"豁免。

组织方式：

- 单元测试：模块内 `#[cfg(test)] mod tests`。
- 集成测试：`tests/` 目录，每文件一个独立 crate；共享工具放 `tests/common/mod.rs`。
- 异步测试用 `#[tokio::test]`；文档测试覆盖公开 API 示例。
- 写实现的 Agent 不得以自身摘要作为验证证据（`docs/15 §6.3` 独立验证）。

### 1.13 代码审查清单（Rust）

```markdown
## 命名与格式
- [ ] 类型 CamelCase，函数/变量 snake_case，常量 SCREAMING_SNAKE
- [ ] cargo fmt --check 通过
- [ ] 无 get_ 前缀滥用、无拼音/单字母命名

## 所有权
- [ ] 无不必要 .clone()
- [ ] 智能指针选型合理（Arc/Rc/Box/Cow）
- [ ] 无 Rc 跨线程

## 错误处理
- [ ] 无 unwrap/expect/panic/exit（生产代码）
- [ ] 用 ? 传播；Option → Result 用 ok_or
- [ ] 库用 thiserror，不暴露 anyhow

## 依赖与边界
- [ ] 未违反 docs/03 §3/§8 的依赖硬规则
- [ ] 领域层未导入 Protobuf/Provider/Adapter 类型

## 并发
- [ ] 锁内无 .await
- [ ] 无 std::thread::sleep（异步中）

## unsafe
- [ ] 无 unsafe（除非 apex-platform/plugin-api + SAFETY 注释）

## 测试
- [ ] 新增代码有测试，覆盖率达 docs/15 §6.2 阈值
- [ ] 关键路径有正常+异常用例
```

---

## 2. TypeScript / Vue 规范（`ui/` 目录）

> 项目前端技术栈：**Vue 3 + TypeScript + Naive UI + Uno CSS + Vite**（见用户 CLAUDE.md）。仅在 `ui/` 目录进入实现阶段后适用。

### 2.1 类型 【必须】

- `strict: true` 开启全部严格检查（含 `noUncheckedIndexedAccess`、`noUnusedLocals`、`noUnusedParameters`）。
- **禁止 `any`**；不确定类型用 `unknown` + 类型守卫收窄。
- 对象结构优先 `interface`；联合/交叉/映射类型用 `type`。
- 类型断言用 `as`（禁止 `<Type>`）；优先类型守卫 `is` 收窄而非断言。
- 可选属性用 `?.` 访问、空值用 `??`；可选链后不接 `!`。
- 条件中 Promise 必须 `await`（Promise 恒为 truthy）。

### 2.2 命名与模块 【必须】

- 变量/函数 `camelCase`；类/接口/类型 `PascalCase`；常量 `UPPER_CASE`；文件 kebab-case。
- 用 `import` 不用 `require`；类型导入用 `import type`。
- 组件名 `PascalCase`（与文件名一致）。

### 2.3 异步 【必须】

- `async` 函数显式标注 `Promise<T>` 返回类型。
- 并发请求用 `Promise.all`；错误用 `try-catch` 包裹。

### 2.4 代码审查清单（TS/Vue）

```markdown
- [ ] 无 any；无多余非空断言 !
- [ ] strict: true 编译无错误
- [ ] import type 用于纯类型导入
- [ ] async 函数有错误处理
```

---

## 3. 安全红线（所有语言）【必须】

- 无硬编码密钥/Token/密码；Secret 只走环境变量或凭据存储。
- `apex-observability` 的 Secret Firewall 在所有日志 sink 前执行脱敏（见 `docs/03 §4.4`）。
- 未配置 Provider/MCP/Update 时，daemon 不发出任何外部网络请求（无遥测基线）。
- 第三方 Plugin 永不进入 `apexd` 地址空间（`docs/03 §4.6`）。
- Web 只监听 localhost，且仅在 TUI 持有启用租约时开放。
