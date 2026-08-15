# Apex

> 一个以 **Spec 驱动开发（Spec-Driven Development）** 为核心，强调规范校验、权限控制、可恢复执行与全链路审计的开源 AI 编程 Agent。

## 当前阶段

仓库处于 **v0.1 工程治理与契约基座建设阶段**。当前可执行能力仅包括：

- Rust Cargo workspace 基座；
- `xtask` 仓库自动化入口；
- Feature Spec frontmatter schema、fixture 与校验脚本；
- 系统分析、详细设计、原子化执行计划和工程规范。

Agent Runtime、TUI、Provider、SQLite Event Store、Tool Gateway、权限引擎等业务能力尚未实现。历史 observability 原型已删除，不属于当前实现基线。

## 仓库结构

```text
Apex/
├── .cargo/                     # Cargo 别名与本地构建配置
├── docs/                       # 系统分析、设计与原子化执行计划
├── rules/                      # 编码规范与 Git 提交规范
├── schemas/                    # Schema 与正反例 fixture
├── scripts/                    # 无第三方依赖的治理校验脚本
├── specs/                      # Feature Spec 模板与功能规格
├── xtask/                      # 仓库自动化入口
├── Cargo.toml                  # Cargo workspace 根清单
├── Cargo.lock                  # 应用型 workspace 依赖锁
└── rust-toolchain.toml         # Rust 工具链锁定
```

后续 crate 和应用只在对应 EP 实现时加入 workspace，避免用无行为的占位 crate 冒充功能完成。

## 工程约束

- Rust 2024 Edition；
- 工具链与 target 基线以 `rust-toolchain.toml` 和执行计划为准；
- 所有 Rust 成员继承 workspace package、依赖和 lint 配置；
- 生产代码禁止 `unwrap`、`expect`、`panic` 和直接退出进程；
- 依赖方向、错误语义、测试和提交格式遵守 `rules/`；
- README 状态不得作为 EP 完成证据，完成状态以代码、VAL 和可复现验证记录为准。

## 当前验证命令

```bash
cargo xtask verify workspace
cargo xtask verify identifiers
cargo xtask verify specs
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings -A missing-docs
cargo test --workspace
```

Feature Spec 模板、schema 与正负 fixture 由 `cargo xtask verify specs` 统一校验。

## 文档入口

- `docs/Apex 功能开发原子化执行计划.md`：EP 顺序、依赖、VAL 与 DoD；
- `docs/Apex 原子模块系分文档.md`：模块边界、接口、失败语义与测试策略；
- `docs/Apex 设计文档.md`：全局架构、领域模型、协议与版本路线；
- `rules/Apex 编码规范.md`：编码、依赖、安全与质量规则；
- `rules/Apex Git 提交规范.md`：提交、分支和提交前检查。

## 开发原则

1. EP 是最小可领取、可提交、可验证、可回滚单元；
2. 一次只实现一个 Active EP，不提前混入下游功能；
3. 先建立失败用例，再实现最小闭环；
4. 领域事实优先于 UI 展示和临时日志；
5. 安全、权限、审计和 Verification Gate 不得延期补做。

## 许可证

Workspace 元数据声明采用 `MIT OR Apache-2.0`。正式发布前需补充 `LICENSE-MIT` 与 `LICENSE-APACHE` 正文。
