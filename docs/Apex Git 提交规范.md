# Apex Git 提交规范

> 本规范是 Apex 项目所有 Git 提交必须遵守的规则。采用 [Conventional Commits 1.0.0](https://www.conventionalcommits.org/)（Angular 风格），subject 使用中文。
>
> 参考来源：[项目代码提交规范](https://yccoding.com/pages/100c7a/)。与项目全局规则 `common/git-workflow.md` 保持一致并细化。

## 1. Commit Message 格式 【必须】

```text
<type>(<scope>): <subject>
<空行>
<body>
<空行>
<footer>
```

| 部分 | 必填 | 说明 |
|---|---|---|
| type | 必填 | 提交类型，见 §2 |
| scope | 可选 | 影响范围/模块名，括号括起，见 §3 |
| subject | 必填 | 中文，≤ 50 字符，祈使句，首字母不大写，结尾不加句号 |
| body | 可选 | 说明 what 与 why，每行 ≤ 72 字符，可用 `-` 列表 |
| footer | 可选 | `BREAKING CHANGE: ...` 或 `Closes #123` / `Refs #456` |

## 2. Type 类型 【必须】

| 类型 | 说明 | 语义化版本 |
|---|---|---|
| `feat` | 新功能 | minor |
| `fix` | 修复 Bug | patch |
| `docs` | 文档变更 | — |
| `style` | 格式调整（不影响逻辑） | — |
| `refactor` | 重构（不增功能、不修 Bug） | — |
| `perf` | 性能优化 | patch |
| `test` | 测试相关 | — |
| `build` | 构建系统/外部依赖（Cargo.toml、pnpm 等） | patch |
| `ci` | CI/CD 配置（.github/workflows 等） | — |
| `chore` | 其他不修改 src/test 的杂项 | — |
| `revert` | 回退之前的提交 | 视被回退提交 |

## 3. Scope 取值 【推荐】

优先使用 crate 名（去掉 `apex-` 前缀）或功能域：

```text
feat(permission): …      fix(dag): …           refactor(session-runtime): …
docs(spec): …            test(checkpoint): …   build(deps): …
ci: …                    chore: …
```

通用：`ci`、`build`、`deps`、`config`、`release`。

## 4. Subject 写法 【必须】

```text
✅ 添加微信支付功能          ❌ 添加了微信支付功能（不用"了"）
✅ 修复 OOM 崩溃问题          ❌ 修复了一个 OOM 崩溃（去掉"一个"）
✅ 删除废弃的 v1 接口         ❌ 删除文件（太模糊）
✅ fix(login): 修复登录 401   ❌ Fix(login): fixed 登录 bug.（大小写/中英混杂/句号）
```

## 5. 完整示例

```text
feat(permission): 新增路径白名单的 glob 匹配支持

- 在 WhiteList 中引入 globset 编译缓存
- 权限决策证据记录匹配到的规则编号

Refs RQ-052
```

```text
fix(dag): 修复路径租约 TTL 过期后旧写仍生效的问题

在 Claim 释放前比较 fencing token，过期 owner 的写入被拒绝。

Closes #178
```

```text
revert: 回退 feat(provider-openai) 的流式重连改动

线上出现事件乱序，先回退待排查。

Refs #201
```

## 6. 分支管理 【推荐】

```text
main                     主分支，永远可构建通过 CI
feature/<模块>-<描述>     功能分支，完成后合并删除
fix/<描述>                修复分支
release/<版本号>          发布分支
hotfix/<版本号>-<描述>    热修复分支
```

- 全小写，词间 `-`；不用个人名字做分支名。
- feature → main：**Squash Merge**（压缩为一条整洁提交）。
- 同步上游用 `git pull --rebase`，避免多余 merge commit。
- `force push` 仅限自己的 feature 分支，且用 `--force-with-lease`。

## 7. PR 规范 【必须】

- 一个 PR 只改一个关注点；变更行数目标 ≤ 200，> 500 必须拆分。
- PR 描述包含：变更说明、变更类型、影响范围、测试说明、关联 Issue/RQ 编号。
- 自检清单：通过 CI（fmt + clippy + test）、commit 符合本规范、无硬编码敏感信息、新增代码有测试、考虑了向后兼容。

## 8. 提交前检查 【必须】

提交前在本地运行（与 CI 一致）：

```bash
cargo fmt --all -- --check          # 格式
cargo clippy --workspace --all-targets -- -D warnings -A missing-docs
cargo test --workspace              # 测试
```

任何一项不通过不得提交。不要提交 `/target/`、`/logs/`、`*.log` 等产物（已在 `.gitignore`）。

## 9. 反模式（禁止） 【必须】

| 反模式 | 改进 |
|---|---|
| `update` / `fix bug` / `asdfgh` 作为 message | `fix(login): 修复密码输入框焦点问题` |
| 一个 PR 20+ 条 wip/fix typo 提交 | 发 PR 前 `git rebase -i` 整理或 squash merge |
| `git add .` 一次性提交所有改动 | 按逻辑分组，不同关注点分多次 commit |
| `git pull` 产生无意义 merge commit | `git pull --rebase` |
| commit 中含 TODO 注释无跟踪 | TODO 必须署名 + 关联 issue/RQ 编号 |
| 提交密钥/Token/密码 | `.gitignore` + 凭据存储；泄漏后立即轮换 |
| 提交编译产物（target/dist） | `.gitignore` 排除 |
| force push 到共享分支 | 仅自己 feature 分支 + `--force-with-lease` |
