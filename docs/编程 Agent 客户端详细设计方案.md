# Apex TUI — 编程 Agent 客户端详细设计方案

## 一、产品概述

### 1.1 产品定义

Apex TUI 是一款面向软件开发者的 AI 编程 Agent 客户端。它以自包含原生窗口应用的形式运行，内部采用 TUI（文本用户界面）渲染，将传统 IDE 的复杂界面压缩为以会话驱动的轻量级工作台。

### 1.2 核心特征

| 特征             | 说明                                                                                 |
|------------------|--------------------------------------------------------------------------------------|
| **自包含启动**   | 双击 `.app` / `.exe` / AppImage 直接运行，无需预先打开系统终端                       |
| **项目中心**     | 启动时选择工作目录，所有操作基于项目上下文                                           |
| **会话驱动**     | 主界面为聊天式会话流，Agent 与用户通过多轮对话协作编程                               |
| **流水线可视化** | 顶部 Spec 流水线实时展示项目从需求到验证的推进状态                                   |
| **多视图切换**   | 底部标签栏在 Session / Spec / Activity / DAG / Memory / Checkpoint / Terminal 间切换 |
| **零 IDE 元素**  | 不显示文件树、不内嵌代码编辑器，代码通过消息块和 diff 展示                           |
| **本地终端**     | Terminal 标签仅支持本地 Shell，不暴露远程 SSH 功能                                   |

### 1.3 目标用户

- 习惯键盘操作、追求效率的开发者
- 需要 AI 辅助编程但不愿打开重型 IDE 的用户
- 偏好终端/文本界面美学、同时需要原生应用体验的用户

## 二、用户旅程

### 2.1 首次启动

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. 用户双击 Apex 图标                                                        │
│       │                                                                       │
│       ▼                                                                       │
│  2. 检测 ~/.config/apex/config.toml                                          │
│       ├── 存在最近项目记录 ──→ 显示项目选择弹层                                │
│       │                        ├─ 最近项目列表（项目名称 + 路径 + 最后打开时间）│
│       │                        ├─ [打开新项目] 按钮 → 文件浏览器选择目录        │
│       │                        └─ 按 Enter 默认选中第一项                      │
│       │                                                                       │
│       └── 无记录 ──→ 直接显示目录选择器                                        │
│       │                                                                       │
│       ▼                                                                       │
│  3. 用户确认项目目录                                                           │
│       ├── 验证目录有效性（是否存在 .git 或允许初始化）                          │
│       ├── 加载 .apex/config.toml（项目级配置）                                 │
│       ├── 读取 Git 分支信息                                                    │
│       └── 初始化 Session（生成唯一 Session ID）                                │
│       │                                                                       │
│       ▼                                                                       │
│  4. 进入主会话界面                                                             │
│       ├── 顶部导航栏显示项目名称                                                 │
│       ├── 流水线根据 .apex/spec.md 解析当前阶段                                │
│       ├── 状态栏显示 model / tokens / branch / mode                            │
│       └── 消息区显示系统欢迎消息（项目摘要 + 可用命令提示）                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 日常会话流程

```plain
用户输入："给这个 Rust 项目加一个权限管理模块"
│
▼
输入框解析 → 普通消息模式
│
▼
消息追加到历史区（user · T-14 · 15:42:03）
│
▼
发送给 Agent API（含项目上下文：文件树摘要、最近修改、当前分支）
│
▼
状态栏：idle → loading（◐ 动画）
│
▼
Agent 流式返回：
├── 文本块："已读取 docs/01-requirements.md 与 docs/03-workspace-and-crates.md"
├── 进度块：Read Cargo.toml  ████████░░  2.1s · 38 lines
├── 代码块：pub trait Permission { ... }
└── 建议块："建议新增 crate apex-permission..."
│
▼
用户审阅 → 按 y 确认 / 按 n 拒绝 / 输入追问
```

### 2.3 状态转换图

```plain
┌─────────────┐
│   启动中    │
└──────┬──────┘
       │ 选择项目
       ▼
┌─────────────┐
│  项目加载   │
└──────┬──────┘
       │ 初始化完成
       ▼
┌─────────────┐          ┌────────────────┐
│   空闲态    │◄─────────│    中断态      │
│   (idle)    │          │ (interrupted)  │
└──────┬──────┘          └───────▲────────┘
       │ 用户输入 / Agent 调用    │
       ▼                         │
┌─────────────┐                  │
│   处理中    │                  │
│  (loading)  │                  │
└──────┬──────┘                  │
       │ 流式响应完成            │
       ▼                         │
┌─────────────┐                  │
│  等待确认   │──── y / n / ! ───┘
│  (pending)  │  （需要用户确认的操作，如写文件、执行命令）
└─────────────┘
```

## 三、界面结构详解

界面采用严格的七层垂直布局，每层职责单一，通过 ratatui::layout::Layout 精确控制。

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│ [Layer 1] 导航栏                    Height: 1 row                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Layer 2] Spec 流水线                 Height: 1 row                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│ [Layer 3] 主内容区                    Height: Fill (剩余空间)                 │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Layer 4] 输入控制区                  Height: 3-5 rows（自适应）              │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Layer 5] 状态栏                      Height: 1 row                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Layer 6] 标签栏                      Height: 1 row                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Layer 7] 通知浮层                    Overlay（右下角，不占用布局空间）         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.1 Layer 1：导航栏（Nav Bar）

**位置**：窗口最顶部  
**高度**：1 行  
**样式**：深色背景，文字使用低对比度灰色，当前位置高亮  
**内容结构**：

```plain
Apex TUI 原型  ·  非功能演示  ·  ← 返回导航  ·  tui-01  ·  主会话界面
│__________│    │__________│    │__________│   │______│   │__________│
产品名称        当前模式        返回按钮      会话ID      当前页面名称
```

元素说明：

| 元素            | 类型       | 交互                            |
|-----------------|------------|---------------------------------|
| `Apex TUI 原型` | 静态文本   | 无                              |
| `非功能演示`    | 状态标签   | 无，仅标识当前运行模式          |
| `← 返回导航`    | 可点击按钮 | 鼠标点击或 `Alt+←` 返回上级视图 |
| `tui-01`        | 会话标识   | 无                              |
| `主会话界面`    | 页面标题   | 无                              |

Rust 渲染逻辑：

```rust
fn render_nav_bar(frame: &mut Frame, area: Rect, app: &App) {
    let spans = vec![
        Span::styled("Apex TUI 原型", Style::default().fg(Color::Gray)),
        Span::styled(" · ", Style::default().fg(Color::DarkGray)),
        Span::styled("非功能演示", Style::default().fg(Color::DarkGray)),
        Span::styled(" · ", Style::default().fg(Color::DarkGray)),
        Span::styled("← 返回导航", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        Span::styled(" · ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.session_id_short, Style::default().fg(Color::Gray)),
        Span::styled(" · ", Style::default().fg(Color::DarkGray)),
        Span::styled("主会话界面", Style::default().fg(Color::White)),
    ];
    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}
```

### 3.2 Layer 2：Spec 流水线（Pipeline）

**位置**：导航栏下方  
**高度**：1 行  
**样式**：深色背景，阶段名称为灰色，已完成阶段绿色 ✓，当前阶段黄色 ▶，未开始阶段灰色 ○  
**视觉结构**：

```plain
Spec 流水线    ✓ 需求  →  ✓ 设计  →  ✓ 任务  →  ▶ 实现  →  ○ 验证        3/5 阶段
│________│    │_____________________________________________________│   │______│
标签           5 个阶段，箭头连接                                     进度计数
```

**颜色编码**：

| 状态  | 符号  | 颜色                   | 说明           |
| --- | --- | -------------------- | ------------ |
| 已完成 | `✓` | Green (`#98c379`)    | 该阶段所有任务已验收   |
| 进行中 | `▶` | Yellow (`#e5c07b`)   | 当前活跃阶段       |
| 未开始 | `○` | DarkGray (`#5c6370`) | 等待前置阶段完成     |
| 失败  | `✗` | Red (`#e06c75`)      | 该阶段有任务失败，需重试 |

**交互**：

- 鼠标悬停某阶段 → 显示 Tooltip（阶段描述 + 任务数 + 预计耗时）
- 点击某阶段 → 自动切换至 Spec 标签页并定位到该阶段详情

### 3.3 Layer 3：主内容区（Main Content）

**位置**：占据窗口绝大部分垂直空间  
**行为**：根据当前激活标签动态渲染内容  
**滚动**：垂直滚动由内容区自身管理，鼠标滚轮或 PgUp/PgDn 触发

**各标签内容概览**：

| 标签       | 内容描述                                                     |
|------------|--------------------------------------------------------------|
| Session    | 消息历史时间线，可滚动，消息块按角色分组                     |
| Spec       | 需求文档树、设计文档、任务清单、检查点列表                   |
| Activity   | 时间倒序的操作日志流（文件读取、命令执行、API 调用）         |
| DAG        | ASCII/Unicode 绘制的任务依赖关系图                           |
| Memory     | Agent 上下文记忆片段列表，可查看/编辑/删除                   |
| Checkpoint | 代码检查点（轻量级快照）列表，可回滚/对比                    |
| Terminal   | 本地 Shell 终端，支持命令输入和彩色输出                      |

### 3.4 Layer 4：输入控制区（Input Area）

**位置**：主内容区下方  
**高度**：最小 3 行（提示行 + 输入框 + 发送按钮），最大 5 行（输入框扩展时）  
**样式**：与主内容区有分隔线，输入框有独立背景色

**子区域划分**：

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│ [4a] 快捷键提示行                                                             │
│ Enter 发送   Shift+Enter 换行   / 命令   @file 引用   ! 中断                  │
│ │____│       │____________│     │____│   │________│   │__│                   │
│  发送键        换行键           命令前缀  文件引用前缀  中断信号               │
├─────────────────────────────────────────────────────────────────────────────┤
│ [4b] 输入框                                                                   │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ 输入消息，/ 命令，@ 引用文件                                            │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                         [send]
└─────────────────────────────────────────────────────────────────────────────┘
```

**输入框行为**：

| 状态   | 触发条件     | 表现                                                            |
|--------|--------------|-----------------------------------------------------------------|
| 空态   | 无输入       | 显示占位符 `输入消息，/ 命令，@ 引用文件`，灰色斜体             |
| 普通态 | 输入普通文本 | 白色文字，Enter 发送                                            |
| 命令态 | 输入 `/`     | 弹出命令补全列表（/clear, /mode, /help, /quit）                 |
| 引用态 | 输入 `@`     | 弹出文件路径补全浮层，支持模糊搜索项目文件                      |
| 多行态 | Shift+Enter  | 输入框扩展至最多 5 行，显示行号                                 |

**输入历史**：

- 按 `↑` 浏览已发送消息历史（类似 Shell history）
- 历史记录持久化到 `~/.config/apex/history.toml`，保留最近 1000 条

### 3.5 Layer 5：状态栏（Status Bar）

**位置**：输入区下方  
**高度**：1 行  
**样式**：深色背景，信息分三列左中右对齐

**内容结构**：

```plain
● idle        model claude-opus-4-8    tokens 42.3k/200k    cache 87%
│____│        │____________________│   │________________│   │________│
连接状态        当前模型                Token 使用量            缓存命中率

branch feature/permission    mode ask    session 01HXY…AB7    workspace ~/code/apex
│________________________│   │________│   │________________│   │__________________│
Git 分支                    交互模式      会话 ID（截断）         工作区路径
```

**连接状态指示器**：

| 状态     | 符号 | 颜色               | 说明                           |
|----------|------|--------------------|--------------------------------|
| 空闲     | ●    | Green              | 等待用户输入                   |
| 处理中   | ◐    | Yellow（旋转动画） | Agent 正在生成响应             |
| 错误     | ●●   | Red                | 连接断开或 API 错误            |
| 等待确认 | ◐    | Cyan（闪烁）       | 有 pending 的 Permission 请求  |

**交互模式（mode）**：

- `ask`：每次操作前询问用户确认（默认）
- `auto`：Agent 自主执行，仅在危险操作（删除、执行命令）时弹窗确认

### 3.6 Layer 6：标签栏（Tab Bar）

**位置**：状态栏下方，窗口最底部  
**高度**：1 行  
**样式**：当前激活标签高亮背景 + 白色文字，非激活标签暗色背景

**标签定义**：

```plain
^1 Session    ^2 Spec[2]    ^3 Activity[▶1]    ^4 DAG[◀1]    ^5 Memory
^6 Checkpoint    ^7 Terminal
│________│    │________│    │____________│    │__________│    │________│
标签名        角标：未读数   角标：进行中任务      角标：阻塞依赖
```

**角标规则**：

| 角标  | 含义                    | 示例                            |
|-------|-------------------------|---------------------------------|
| [n]   | 该标签有 n 条未读/待办  | Spec[2] = 2 个待确认的任务      |
| [▶n]  | 有 n 个进行中的异步任务 | Activity[▶1] = 1 个文件正在读取 |
| [◀n]  | 有 n 个阻塞的依赖       | DAG[◀1] = 1 个任务等待前置完成  |

**快捷键**：

- `Ctrl+1` ~ `Ctrl+7`：直接切换标签
- 鼠标点击标签切换

### 3.7 Layer 7：通知浮层（Notification Overlay）

**位置**：右下角，叠加在所有层级之上  
**样式**：带边框的弹窗，根据类型有不同颜色

**类型与样式**：

```plain
┌─────────────────┐
│ Permission      │  ← 标题，Cyan 色
│─────────────────│
│ Agent 请求删除    │
│ target/debug/    │
│ 目录，是否允许？  │
│                 │
│ [Y] 允许  [N] 拒绝│
└─────────────────┘
```

| 类型       | 边框色 | 自动消失   | 交互     |
|------------|--------|------------|----------|
| Permission | Cyan   | 否（常驻） | y/n 响应 |
| Info       | Gray   | 3 秒       | 无       |
| Warning    | Yellow | 5 秒       | 无       |
| Error      | Red    | 否（常驻） | Esc 关闭 |

**堆叠规则**：最多同时显示 3 个通知，新通知从底部顶入，旧通知自动淡出。

## 四、数据模型

### 4.1 核心实体关系

```plain
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│   Project   │◄─────►│   Session   │◄─────►│   Message   │
│  (项目)      │ 1:N   │  (会话)      │ 1:N   │  (消息)      │
└──────┬──────┘       └──────┬──────┘       └──────┬──────┘
│                     │                     │
│              ┌──────┴──────┐             │
│              │             │             │
▼              ▼             ▼             ▼
┌─────────────┐  ┌─────────┐  ┌──────────┐  ┌─────────────┐
│   Config    │  │  Spec   │  │ Activity │  │   Checkpoint │
│ (项目配置)   │  │(流水线)  │  │ (日志)    │  │  (检查点)    │
└─────────────┘  └─────────┘  └──────────┘  └─────────────┘
```

### 4.2 项目（Project）

```rust
pub struct Project {
    pub id: String,                 // 唯一标识（UUID v7）
    pub name: String,               // 目录名
    pub path: PathBuf,              // 绝对路径（如 ~/code/apex）
    pub git_branch: Option<String>, // 当前 Git 分支
    pub created_at: DateTime<Utc>,
    pub last_opened_at: DateTime<Utc>,

    // 项目级配置
    pub config: ProjectConfig,

    // 子实体
    pub spec: SpecPipeline,
    pub sessions: Vec<Session>,
    pub checkpoints: Vec<Checkpoint>,
}
```

### 4.3 会话（Session）

```rust
pub struct Session {
    pub id: String,            // 如 01HXY…AB7（Crockford Base32 编码的 ULID）
    pub project_id: String,
    pub model: String,         // claude-opus-4-8
    pub mode: InteractionMode, // Ask / Auto
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub token_usage: TokenUsage,
}

pub enum InteractionMode {
    Ask,  // 每次操作询问确认
    Auto, // 自动执行，危险操作弹窗
}

pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub limit: u64, // 200k
}
```

### 4.4 消息（Message）

```rust
pub struct Message {
    pub id: String,
    pub turn_id: String, // T-12, T-13（Turn 序号）
    pub role: MessageRole,
    pub timestamp: DateTime<Local>,
    pub blocks: Vec<MessageBlock>,
    pub status: MessageStatus,
    pub metadata: MessageMetadata,
}

pub enum MessageRole {
    User,
    Apex,
    System,
}

pub enum MessageStatus {
    Sending,     // 正在发送到 API
    Streaming,   // 正在接收流式响应
    Complete,    // 已完成
    Failed,      // 发送/接收失败
    Interrupted, // 被用户中断（!）
}

pub enum MessageBlock {
    Text(TextBlock),
    Code(CodeBlock),
    Diff(DiffBlock),
    Progress(ProgressBlock),
    FileRef(FileRefBlock),
    Error(ErrorBlock),
}

pub struct TextBlock {
    pub content: String, // Markdown 格式
}

pub struct CodeBlock {
    pub language: String, // rust, python, toml...
    pub code: String,
    pub path: Option<String>, // 可选：代码所属文件路径
    pub line_range: Option<(usize, usize)>,
}

pub struct DiffBlock {
    pub path: String,
    pub old_lines: Vec<DiffLine>,
    pub new_lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub line_no: Option<usize>,
    pub content: String,
    pub kind: DiffLineKind, // Context / Added / Removed
}

pub struct ProgressBlock {
    pub description: String, // "Read Cargo.toml"
    pub elapsed: Duration,
    pub total_lines: Option<usize>, // 38 lines
    pub percent: Option<f32>,       // 0.0 ~ 1.0
}

pub struct FileRefBlock {
    pub path: String,
    pub content: String, // 引用的文件内容摘要
}

pub struct ErrorBlock {
    pub message: String,
    pub suggestion: Option<String>,
}
```

### 4.5 Spec 流水线

```rust
pub struct SpecPipeline {
    pub stages: Vec<Stage>,
    pub current_stage_index: usize, // 0-based
}

pub struct Stage {
    pub id: String,
    pub name: String, // 需求 / 设计 / 任务 / 实现 / 验证
    pub status: StageStatus,
    pub tasks: Vec<Task>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

pub struct Task {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub assignee: Option<String>,  // user / apex
    pub dependencies: Vec<String>, // 依赖的其他任务 ID
}
```

### 4.6 Activity 日志

```rust
pub struct ActivityLog {
    pub entries: Vec<ActivityEntry>,
}

pub struct ActivityEntry {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub kind: ActivityKind,
    pub description: String,
    pub status: ActivityStatus,
    pub duration: Option<Duration>,
    pub metadata: ActivityMetadata,
}

pub enum ActivityKind {
    FileRead { path: String, lines: usize },
    FileWrite { path: String, bytes: usize },
    CommandExecute { command: String, exit_code: Option<i32> },
    AgentCall { model: String, tokens_in: u64, tokens_out: u64 },
    GitOperation { command: String },
    UserAction { action: String },
}

pub enum ActivityStatus {
    Running,   // ▶
    Completed, // ✓
    Failed,    // ✗
    Cancelled, // ⊘
}
```

## 五、交互设计

### 5.1 全局快捷键

| 快捷键          | 功能                                                    | 上下文 |
|-----------------|---------------------------------------------------------|--------|
| Ctrl+C          | 复制选中内容 / 中断当前操作                             | 全局   |
| Ctrl+V          | 粘贴到输入框                                            | 全局   |
| Ctrl+1 ~ Ctrl+7 | 切换至对应标签页                                        | 全局   |
| Ctrl+Tab        | 切换到下一个标签                                        | 全局   |
| Ctrl+Shift+Tab  | 切换到上一个标签                                        | 全局   |
| Ctrl+L          | 清空当前标签内容（Session 不清除消息，仅清屏）          | 全局   |
| Ctrl+Q          | 退出应用                                                | 全局   |
| Esc             | 关闭弹层 / 取消当前操作 / 退出命令模式                  | 全局   |
| PgUp / PgDn     | 主内容区滚动                                            | 全局   |
| Home / End      | 滚动到顶部 / 底部                                       | 全局   |

### 5.2 输入框快捷键

| 快捷键      | 功能                                         |
|-------------|----------------------------------------------|
| Enter       | 发送消息                                     |
| Shift+Enter | 输入框内换行                                 |
| ↑           | 浏览上一条发送历史（仅当光标在第一行且行首） |
| ↓           | 浏览下一条发送历史                           |
| Tab         | 触发命令/文件补全                            |
| Ctrl+A      | 全选输入框内容                               |
| Ctrl+K      | 清空输入框                                   |

### 5.3 消息历史区快捷键

| 快捷键 | 功能                                                |
|--------|-----------------------------------------------------|
| ↑ / ↓  | 逐行滚动                                            |
| y      | 当有 Permission 通知时，确认允许                    |
| n      | 当有 Permission 通知时，拒绝                        |
| Space  | 展开/折叠长消息块                                   |
| Enter  | 点击消息中的可交互元素（如 diff 块的 "Apply" 按钮） |

### 5.4 鼠标交互

| 操作               | 响应                                         |
|--------------------|----------------------------------------------|
| 左键点击消息       | 聚焦该消息，显示操作菜单（复制、引用、折叠） |
| 左键点击代码块     | 展开/折叠代码块                              |
| 左键点击标签       | 切换标签                                     |
| 左键点击流水线阶段 | 跳转至 Spec 标签对应阶段                     |
| 滚轮滚动           | 主内容区垂直滚动                             |
| 右键点击消息       | 弹出上下文菜单（复制、引用、删除）           |

### 5.5 命令系统（/ 前缀）

| 命令        | 参数                                  | 功能                        |
|-------------|---------------------------------------|-----------------------------|
| /clear      | 无                                    | 清空当前 Session 的所有消息 |
| /mode       | ask / auto                            | 切换交互模式                |
| /model      | `<model-name>`                        | 切换 AI 模型                |
| /spec       | status / next / back                  | 查看/推进 Spec 流水线       |
| /checkpoint | list / save [name] / restore `<id>`   | 检查点管理                  |
| /memory     | clear / show                          | 上下文记忆管理              |
| /help       | [command]                             | 显示帮助信息                |
| /quit       | 无                                    | 退出应用                    |

### 5.6 文件引用系统（@ 前缀）

**触发方式**：输入 `@` 后自动弹出文件补全浮层

**补全逻辑**：

1. 读取当前项目文件树（缓存，监听文件系统变化）
2. 支持模糊匹配（fzf 风格）
3. 按 `Tab` 循环选择，`Enter` 确认引用
4. 引用后，文件内容作为消息附件发送到 Agent

**引用格式**：

```plain
用户输入：给 @src/permission/mod.rs 加一个默认实现
实际发送：给 [file:src/permission/mod.rs] 加一个默认实现
（附件包含该文件完整内容）
```

## 六、技术架构

### 6.1 模块依赖图

```plain
apex-tui/
├── Cargo.toml
└── src/
├── main.rs              # 入口：初始化 winit 事件循环
├── app.rs               # App 状态机与消息分发
├── config.rs            # 配置加载与验证
├── lib.rs               # 库入口（供测试）
│
├── backend/             # 自定义 ratatui Backend
│   ├── mod.rs
│   ├── pixel.rs         # PixelBackend 实现
│   └── diff.rs          # 增量渲染算法
│
├── ui/                  # 界面渲染
│   ├── mod.rs           # ui() 主函数，标签路由
│   ├── nav.rs           # 导航栏
│   ├── pipeline.rs      # Spec 流水线
│   ├── status_bar.rs    # 状态栏
│   ├── tab_bar.rs       # 标签栏
│   ├── input.rs         # 输入控制区
│   ├── notification.rs  # 通知浮层
│   │
│   ├── session/         # Session 标签
│   │   ├── mod.rs
│   │   ├── message_list.rs
│   │   ├── message_block.rs
│   │   └── input_box.rs
│   │
│   ├── spec/            # Spec 标签
│   │   ├── mod.rs
│   │   ├── stage_tree.rs
│   │   └── task_list.rs
│   │
│   ├── activity/        # Activity 标签
│   │   └── mod.rs
│   │
│   ├── dag/             # DAG 标签
│   │   └── mod.rs
│   │
│   ├── memory/          # Memory 标签
│   │   └── mod.rs
│   │
│   ├── checkpoint/      # Checkpoint 标签
│   │   └── mod.rs
│   │
│   └── terminal/        # Terminal 标签
│       ├── mod.rs
│       └── local_shell.rs
│
├── agent/               # Agent API 通信
│   ├── mod.rs
│   ├── client.rs        # HTTP/SSE 客户端
│   ├── stream.rs        # 流式响应解析
│   └── context.rs       # 上下文组装（项目摘要 + 文件引用）
│
├── project/             # 项目管理
│   ├── mod.rs
│   ├── loader.rs        # 项目加载与验证
│   ├── git.rs           # Git 信息读取
│   └── file_tree.rs     # 文件树缓存与监听
│
├── spec/                # Spec 流水线管理
│   ├── mod.rs
│   ├── parser.rs        # .apex/spec.md 解析
│   └── state.rs         # 阶段状态机
│
└── platform/            # 平台适配
    ├── mod.rs
    ├── window.rs        # winit 窗口封装
    ├── input.rs         # 输入事件转换
    └── clipboard.rs     # 剪贴板操作
```

### 6.2 核心状态机

```rust
// src/app.rs
pub struct App {
    // 生命周期
    pub running: bool,
    pub startup_phase: StartupPhase,

    // 项目上下文
    pub project: Option<Project>,

    // 视图路由
    pub current_tab: TabId,
    pub tabs: HashMap<TabId, TabState>,

    // 会话状态
    pub session: SessionState,

    // 输入状态
    pub input: InputState,

    // 通知队列
    pub notifications: VecDeque<Notification>,

    // 异步运行时
    pub rt: Runtime,
    pub agent_tx: mpsc::Sender<AgentRequest>,
    pub agent_rx: mpsc::Receiver<AgentResponse>,
}

pub enum StartupPhase {
    ProjectSelector, // 显示项目选择界面
    LoadingProject { path: PathBuf, progress: f32 },
    MainInterface,
}

impl App {
    pub fn new() -> Self { ... }

    /// 主消息分发入口
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Window(event) => self.handle_window_event(event),
            AppEvent::Key(key) => self.handle_key_event(key),
            AppEvent::Mouse(mouse) => self.handle_mouse_event(mouse),
            AppEvent::Agent(response) => self.handle_agent_response(response),
            AppEvent::Tick => self.handle_tick(),
        }
    }
    
    /// 键盘事件处理
    fn handle_key_event(&mut self, key: KeyEvent) {
        // 优先级 1：通知响应（Permission 弹窗）
        if let Some(notif) = self.notifications.front() {
            if notif.requires_input() {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => self.confirm_permission(true),
                    KeyCode::Char('n') | KeyCode::Char('N') => self.confirm_permission(false),
                    KeyCode::Esc => self.dismiss_notification(),
                    _ => {}
                }
                return;
            }
        }
        
        // 优先级 2：全局快捷键
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => self.interrupt_or_copy(),
                KeyCode::Char('q') => self.quit(),
                KeyCode::Char('l') => self.clear_current_tab(),
                KeyCode::Char('1')..=KeyCode::Char('7') => {
                    let tab = match key.code {
                        KeyCode::Char('1') => TabId::Session,
                        KeyCode::Char('2') => TabId::Spec,
                        // ...
                        _ => unreachable!(),
                    };
                    self.switch_tab(tab);
                }
                _ => {}
            }
            return;
        }
        
        // 优先级 3：输入框焦点
        if self.input.has_focus {
            self.input.handle_key(key);
            return;
        }
        
        // 优先级 4：当前标签页导航
        match self.current_tab {
            TabId::Session => self.session.handle_key(key),
            TabId::Spec => self.spec_view.handle_key(key),
            // ...
            _ => {}
        }
    }
}
```

### 6.3 自定义 PixelBackend

```rust
// src/backend/pixel.rs
use ratatui::backend::Backend;
use ratatui::buffer::Cell;

/// 将 ratatui 的 Cell 网格渲染为像素缓冲区
pub struct PixelBackend {
    width: u16,
    height: u16,

    // 双缓冲
    front_buffer: Vec<Cell>, // 当前帧（已提交到屏幕）
    back_buffer: Vec<Cell>,  // 正在绘制的帧

    // 脏区域追踪
    dirty_rows: Vec<bool>, // 每行是否有变化
    cursor: Option<(u16, u16)>,

    // 渲染参数
    cell_width: u16,  // 像素/字符（由字体决定）
    cell_height: u16, // 像素/行
}

impl PixelBackend {
    pub fn new(cols: u16, rows: u16, cell_width: u16, cell_height: u16) -> Self {
        let size = (cols as usize) * (rows as usize);
        Self {
            width: cols,
            height: rows,
            front_buffer: vec![Cell::default(); size],
            back_buffer: vec![Cell::default(); size],
            dirty_rows: vec![false; rows as usize],
            cursor: None,
            cell_width,
            cell_height,
        }
    }

    /// 获取需要重绘的区域
    pub fn dirty_regions(&self) -> Vec<Rect> {
        // 将连续的脏行合并为矩形区域
        let mut regions = Vec::new();
        let mut start: Option<usize> = None;
        
        for (row, is_dirty) in self.dirty_rows.iter().enumerate() {
            if *is_dirty && start.is_none() {
                start = Some(row);
            } else if !*is_dirty && start.is_some() {
                regions.push(Rect::new(0, start.unwrap() as u16, self.width, (row - start.unwrap()) as u16));
                start = None;
            }
        }
        
        if let Some(s) = start {
            regions.push(Rect::new(0, s as u16, self.width, (self.height as usize - s) as u16));
        }
        
        regions
    }
    
    /// 将后端缓冲区转换为 RGBA 像素（供 softbuffer 消费）
    pub fn to_pixels(&self, font_renderer: &FontRenderer) -> Vec<u32> {
        let mut pixels = vec![0u32; (self.width as usize * self.cell_width as usize) 
                                      * (self.height as usize * self.cell_height as usize)];
        
        for row in 0..self.height {
            if !self.dirty_rows[row as usize] {
                continue; // 跳过干净行
            }
            
            for col in 0..self.width {
                let idx = (row as usize) * (self.width as usize) + (col as usize);
                let cell = &self.front_buffer[idx];
                
                font_renderer.render_cell(
                    &mut pixels,
                    col * self.cell_width,
                    row * self.cell_height,
                    self.cell_width,
                    self.cell_height,
                    cell,
                );
            }
        }
        
        pixels
    }
}

impl Backend for PixelBackend {
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            if idx < self.back_buffer.len() {
                self.back_buffer[idx] = cell.clone();
                self.dirty_rows[y as usize] = true;
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        self.back_buffer.fill(Cell::default());
        // dirty_rows 在下一帧 draw 时重置
        Ok(())
    }
    
    // ... 其他 Backend trait 方法
}
```

### 6.4 渲染管线集成

```rust
// src/main.rs — 事件循环中的渲染逻辑
fn run_event_loop(app: &mut App, window: &Window, surface: &mut Surface) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                // 1. 让 ratatui 绘制到 PixelBackend
                app.terminal.draw(|frame| ui(frame, app)).unwrap();
                
                // 2. 获取脏区域
                let backend = app.terminal.backend();
                let dirty_regions = backend.dirty_regions();
                
                // 3. 仅重绘脏区域（优化性能）
                if !dirty_regions.is_empty() {
                    let mut buffer = surface.buffer_mut().unwrap();
                    let pixels = backend.to_pixels(&app.font_renderer);
                    
                    // 将像素写入 softbuffer
                    for row in 0..app.window_height {
                        for col in 0..app.window_width {
                            let idx = row * app.window_width + col;
                            buffer[idx] = pixels[idx];
                        }
                    }
                    
                    buffer.present().unwrap();
                }
            }
            
            Event::AboutToWait => {
                // 处理异步消息（Agent 响应、文件系统事件）
                while let Ok(response) = app.agent_rx.try_recv() {
                    app.handle_event(AppEvent::Agent(response));
                    window.request_redraw();
                }
            }
            
            _ => {}
        }
    }).unwrap();
}
```

## 七、标签页详细设计

### 7.1 Session 标签（默认标签）

**布局结构**：

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│ 消息历史区（可滚动）                                                           │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ user   15:42:03 · T-12                                                  │ │
│ │ ┌─────────────────────────────────────────────────────────────────────┐ │ │
│ │ │ 给这个 Rust 项目加一个权限管理模块...                                 │ │ │
│ │ └─────────────────────────────────────────────────────────────────────┘ │ │
│ │                                                                         │ │
│ │ apex   15:42:08                                                        │ │
│ │ ┌─────────────────────────────────────────────────────────────────────┐ │ │
│ │ │ 已读取 docs/01-requirements.md 与 docs/03-workspace-and-crates.md   │ │ │
│ │ │ 建议新增 crate apex-permission...                                   │ │ │
│ │ │                                                                     │ │ │
│ │ │ [代码块] pub trait Permission { ... }                                │ │ │
│ │ │                                                                     │ │ │
│ │ │ [Progress] Read Cargo.toml  ████████░░  2.1s · 38 lines            │ │ │
│ │ └─────────────────────────────────────────────────────────────────────┘ │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

**消息块渲染规则**：

| 块类型        | 视觉样式                                                                 |
|---------------|--------------------------------------------------------------------------|
| TextBlock     | 普通文本，Markdown 解析（粗体、斜体、链接）                              |
| CodeBlock     | 独立背景色块，左上角显示语言标签，行号右侧对齐，语法高亮                 |
| DiffBlock     | 三列布局：行号 + 符号(+/-) + 内容，新增行绿色背景，删除行红色背景        |
| ProgressBlock | 单行，描述 + 进度条（Unicode block chars）+ 耗时 + 行数                  |
| FileRefBlock  | 折叠态显示路径，展开态显示内容摘要，带"复制路径"按钮                     |
| ErrorBlock    | 红色边框，错误图标，建议操作（如"重试"按钮）                             |

**代码块交互**：

- 鼠标悬停：显示"复制"、"引用"、"在编辑器中打开"按钮
- 点击展开/折叠（超过 20 行自动折叠）
- 语法高亮：基于 tree-sitter 或预定义词法规则

### 7.2 Spec 标签

**三栏布局**：

```plain
┌─────────────┬─────────────────────────────────────────────────────────────┐
│ 阶段列表     │  阶段详情                                                    │
│             │                                                             │
│ ✓ 需求       │  [任务列表]                                                  │
│   └─ 任务1   │  □ 定义 Permission trait                                   │
│   └─ 任务2   │  □ 实现基于路径前缀的 ACL                                   │
│ ✓ 设计       │  □ 编写 TOML 配置 loader                                   │
│   └─ ...     │                                                             │
│ ▶ 任务       │  [文档预览]                                                  │
│   └─ 任务3   │  ## 权限管理模块设计                                         │
│   └─ 任务4   │                                                             │
│ ○ 实现       │                                                             │
│ ○ 验证       │                                                             │
└─────────────┴─────────────────────────────────────────────────────────────┘
```

**功能**：

- 左侧：阶段树，可展开/折叠，显示任务计数
- 右侧：选中阶段的任务清单（复选框）+ 关联文档预览
- 任务状态：□ 未开始 / ◐ 进行中 / ✓ 已完成 / ✗ 失败
- 点击任务 → 可跳转至相关 Session 消息上下文

### 7.3 Activity 标签

**时间倒序日志流**：

```plain
15:42:31  ◐  Read Cargo.toml                    2.1s    38 lines
15:42:30  ✓  Parse workspace members             0.3s    4 crates
15:42:28  ✓  Git status check                    0.1s    feature/permission
15:42:25  ▶  Agent API call                     --      claude-opus-4-8
15:42:03  ✓  User message sent                   0.0s    128 tokens
```

**交互**：

- 点击条目展开详情（完整命令输出、API 请求/响应摘要）
- 按 `d` 删除单条 / `D` 清空全部
- 支持过滤（`/` 输入过滤条件，如 `kind:FileRead`）

### 7.4 DAG 标签

**ASCII 依赖图**：

```plain
┌─────────────┐
│   需求分析   │
└──────┬──────┘
│
┌───────────────┼───────────────┐
▼               ▼               ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│  接口设计    │ │  数据模型    │ │  测试策略    │
└──────┬──────┘ └──────┬──────┘ └──────┬──────┘
│               │               │
└───────────────┼───────────────┘
▼
┌─────────────┐
│  实现编码    │ ◄── 当前阻塞（等待上游）
└─────────────┘
```

**功能**：

- 自动从 Spec 任务依赖关系生成
- 阻塞任务标红，进行中任务标黄
- 点击节点 → 显示任务详情和阻塞原因

### 7.5 Memory 标签

**记忆片段列表**：

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│ [Memory 1]  2024-08-13 15:30                                               │
│ 用户偏好：使用 TOML 而非 YAML 作为配置格式                                   │
│ [编辑] [删除]                                                                │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Memory 2]  2024-08-13 15:25                                               │
│ 项目结构：apex-permission crate 位于 workspace 根目录下                      │
│ [编辑] [删除]                                                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

**功能**：

- Agent 自动提取并持久化关键上下文
- 用户可手动添加/编辑/删除记忆
- 记忆用于跨 Session 保持项目上下文一致性

### 7.6 Checkpoint 标签

**检查点列表（轻量级 Git 替代）**：

```plain
ID          Time                Description                      Files    [操作]
───────     ─────────────────   ─────────────────────────────    ─────    ──────
cp-001      15:42:31            Before adding permission mod     3        [Restore] [Diff]
cp-002      15:45:12            After trait definition           1        [Restore] [Diff]
```

**功能**：

- Agent 在关键操作前自动创建检查点
- 用户可手动创建命名检查点
- Restore：回滚到该检查点状态
- Diff：对比当前工作区与该检查点的差异

### 7.7 Terminal 标签

**本地 Shell 终端**：

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│ ~/code/apex on feature/permission via 🦀 v1.80                              │
│ ❯ cargo test                                                                │
│    Compiling apex-permission v0.1.0                                         │
│     Running unittests src/lib.rs                                            │
│                                                                             │
│ running 12 tests                                                            │
│ test acl::tests::test_prefix_match ... ok                                   │
│ test acl::tests::test_role_verdict ... ok                                   │
│ ...                                                                         │
│ test result: ok. 12 passed; 0 failed; 0 ignored                           │
│                                                                             │
│ ❯ _                                                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

**技术实现**：

- 使用 portable-pty 创建本地 PTY
- Shell 输出直接写入内容区（基础文本渲染，暂不支持 VT 序列解析）
- 若后续需要彩色输出支持，引入 libghostty-rs 进行 VT 解析
- 输入框与 Session 标签共享（在 Terminal 标签时，输入框内容直接发送到 PTY）

## 八、消息系统与渲染管线

### 8.1 Agent 通信协议

```rust
// src/agent/client.rs
pub struct AgentClient {
    http_client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl AgentClient {
    /// 发送消息并返回流式响应通道
    pub async fn stream(
        &self,
        request: AgentRequest,
    ) -> Result<mpsc::Receiver<AgentChunk>, AgentError> {
        let response = self.http_client
            .post(format!("{}/v1/chat", self.base_url))
            .json(&request)
            .send()
            .await?;

        let (tx, rx) = mpsc::channel(128);
        
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let bytes = chunk?;
                let text = String::from_utf8_lossy(&bytes);
                
                // SSE 解析
                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if let Ok(chunk) = serde_json::from_str::<AgentChunk>(data) {
                            tx.send(chunk).await.ok();
                        }
                    }
                }
            }
            Ok::<_, AgentError>(())
        });
        
        Ok(rx)
    }
}
```

### 8.2 流式响应解析

```rust
// src/agent/stream.rs
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentChunk {
    Text { content: String },
    Code { language: String, code: String },
    Diff { path: String, changes: Vec<DiffChange> },
    Progress { description: String, percent: Option<f32> },
    FileRef { path: String, content: String },
    ToolCall { name: String, arguments: Value },
    ToolResult { name: String, result: Value, status: ToolStatus },
    Error { message: String },
    Done,
}

/// 将 AgentChunk 流转换为 MessageBlock
pub struct ChunkAssembler {
    buffer: Vec<AgentChunk>,
    current_block: Option<MessageBlock>,
}

impl ChunkAssembler {
    pub fn push(&mut self, chunk: AgentChunk) -> Option<MessageBlock> {
        match chunk {
            AgentChunk::Text { content } => {
                if let Some(MessageBlock::Text(ref mut block)) = self.current_block {
                    block.content.push_str(&content);
                    None // 累积中，不返回完整块
                } else {
                    // 上一个块结束，开始新块
                    let block = self.current_block.take();
                    self.current_block = Some(MessageBlock::Text(TextBlock { content }));
                    block
                }
            }
            AgentChunk::Code { language, code } => {
                let block = self.current_block.take();
                self.current_block = Some(MessageBlock::Code(CodeBlock { language, code, .. }));
                block
            }
            // ... 其他类型处理
            AgentChunk::Done => self.current_block.take(),
            _ => None,
        }
    }
}
```

### 8.3 消息渲染状态机

```rust
// src/ui/session/message_block.rs
pub struct MessageRenderer {
    pub scroll_offset: u16,
    pub collapsed_blocks: HashSet<String>, // 按块 ID 折叠
    pub selected_block: Option<String>,
}

impl MessageRenderer {
    pub fn render(&self, frame: &mut Frame, area: Rect, message: &Message) {
        let mut y_offset = 0;

        // 消息头部：角色 + 时间戳 + Turn ID
        let header = format!("{}   {} · {}", 
            match message.role {
                MessageRole::User => "user",
                MessageRole::Apex => "apex",
                MessageRole::System => "system",
            },
            message.timestamp.format("%H:%M:%S"),
            message.turn_id
        );
        let header_style = match message.role {
            MessageRole::User => Style::default().fg(Color::Cyan),
            MessageRole::Apex => Style::default().fg(Color::Magenta),
            MessageRole::System => Style::default().fg(Color::Gray),
        };
        frame.render_widget(Paragraph::new(header).style(header_style), 
            Rect::new(area.x, area.y + y_offset, area.width, 1));
        y_offset += 1;
        
        // 渲染各消息块
        for block in &message.blocks {
            let block_area = Rect::new(area.x + 2, area.y + y_offset, area.width - 4, area.height - y_offset);
            match block {
                MessageBlock::Text(t) => self.render_text(frame, block_area, t),
                MessageBlock::Code(c) => self.render_code(frame, block_area, c),
                MessageBlock::Diff(d) => self.render_diff(frame, block_area, d),
                MessageBlock::Progress(p) => self.render_progress(frame, block_area, p),
                _ => {}
            }
            y_offset += self.block_height(block, area.width - 4) + 1;
        }
    }
    
    fn render_code(&self, frame: &mut Frame, area: Rect, block: &CodeBlock) {
        // 背景块
        let block_widget = Block::default()
            .title(format!(" {} ", block.language))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(30, 30, 30)));
        
        let inner = block_widget.inner(area);
        frame.render_widget(block_widget, area);
        
        // 语法高亮（简化版：基于 tree-sitter 或正则匹配）
        let highlighted = syntax_highlight(&block.code, &block.language);
        let paragraph = Paragraph::new(highlighted)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }
    
    fn render_progress(&self, frame: &mut Frame, area: Rect, block: &ProgressBlock) {
        let percent = block.percent.unwrap_or(0.0);
        let bar_width = (area.width as f32 * percent) as u16;
        
        let bar = format!("{}{}", 
            "█".repeat(bar_width as usize),
            "░".repeat((area.width - bar_width) as usize)
        );
        
        let line = format!("{}  {}  {} · {} lines", 
            block.description,
            bar,
            format_duration(block.elapsed),
            block.total_lines.map(|n| n.to_string()).unwrap_or_default()
        );
        
        frame.render_widget(Paragraph::new(line), area);
    }
}
```

## 九、配置与持久化

### 9.1 配置层级

```plain
┌─────────────────────────────────────────────────────────────────────────────┐
│ 系统默认配置 (编译时嵌入)                                                      │
│   ~/.config/apex/config.toml  ← 用户级配置（覆盖系统默认）                     │
│   <project>/.apex/config.toml  ← 项目级配置（覆盖用户级）                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.2 配置结构（TOML）

```toml
# ~/.config/apex/config.toml
[ui]
theme = "dark"                    # dark / light
font_family = "JetBrains Mono"
font_size = 14
line_height = 1.2
cursor_style = "block"            # block / line / underscore

[agent]
default_model = "claude-opus-4-8"
max_tokens = 200000
temperature = 0.7
mode = "ask"                      # ask / auto

[agent.context]
auto_include_git_status = true
max_file_size_kb = 100
excluded_patterns = [".git", "target", "node_modules", "*.lock"]

[keybindings]
quit = "Ctrl+Q"
clear = "Ctrl+L"
new_session = "Ctrl+N"

[terminal]
shell = "/bin/zsh"                # 默认使用 $SHELL
startup_commands = ["cd $APEX_WORKSPACE"]
```

### 9.3 持久化数据

| 数据     | 路径                                                | 格式       |
|----------|-----------------------------------------------------|------------|
| 用户配置 | `~/.config/apex/config.toml`                        | TOML       |
| 最近项目 | `~/.config/apex/recent_projects.toml`               | TOML       |
| 消息历史 | `~/.local/share/apex/sessions/<session_id>.jsonl`   | JSON Lines |
| 输入历史 | `~/.config/apex/input_history.toml`                 | TOML       |
| 检查点   | `<project>/.apex/checkpoints/<id>/`                 | 文件快照   |
| 日志     | `~/.local/share/apex/logs/`                         | 按日期轮转 |

## 十、开发路线图

### Phase 1：窗口骨架与项目选择器（Week 1）

| 任务                    | 交付物                  | 验收标准                       |
|-------------------------|-------------------------|--------------------------------|
| winit + softbuffer 集成 | 可运行的空白窗口        | 双击启动，300ms 内显示窗口     |
| PixelBackend 实现       | ratatui 渲染到窗口      | "Hello Apex" 文本正确显示      |
| 项目选择器 UI           | 最近项目列表 + 目录选择 | 可选项目、确认后进入主界面     |
| 配置加载                | 读取 TOML 配置          | 层级覆盖正确，错误配置友好提示 |

### Phase 2：核心会话界面（Week 2）

| 任务         | 交付物                                                     | 验收标准                             |
|--------------|------------------------------------------------------------|--------------------------------------|
| 七层布局渲染 | 导航栏、流水线、内容区、输入区、状态栏、标签栏             | 各层位置正确，颜色区分明显           |
| 消息历史     | 消息列表、滚动、时间戳、Turn ID                            | 支持 1000+ 消息流畅滚动              |
| 输入框       | 多行输入、历史回溯、Enter/Shift+Enter                      | 输入、发送、换行行为正确             |
| 命令系统     | /clear, /mode, /help                                       | 命令解析、执行、反馈正确             |
| 文件引用     | @ 补全、文件读取                                           | 路径补全准确，文件内容附加到消息     |

### Phase 3：Agent 集成与消息块（Week 3）

| 任务             | 交付物                            | 验收标准                         |
|------------------|-----------------------------------|----------------------------------|
| Agent HTTP 客户端 | SSE 流式接收                      | 模拟流式响应，逐字显示           |
| 消息块解析       | Text / Code / Progress / Error    | 各类型块渲染正确                 |
| 代码块高亮       | 基于 tree-sitter 的语法高亮       | Rust/Python/TOML 高亮正确        |
| Diff 块渲染      | 三列布局、颜色区分                | 新增/删除行颜色正确              |
| 通知系统         | Permission / Info / Warning / Error | 弹层显示、y/n 响应、自动消失   |

### Phase 4：工作台标签（Week 4）

| 任务            | 交付物                     | 验收标准                        |
|-----------------|----------------------------|---------------------------------|
| Spec 标签       | 阶段树、任务清单、文档预览 | 流水线状态与标签同步            |
| Activity 标签   | 日志流、过滤、展开详情     | 实时记录文件/命令/API 操作      |
| DAG 标签        | ASCII 依赖图               | 从 Spec 任务依赖自动生成        |
| Memory 标签     | 记忆片段 CRUD              | Agent 自动提取 + 用户手动管理   |
| Checkpoint 标签 | 检查点列表、恢复、Diff     | 文件快照、回滚正确              |

### Phase 5：Terminal 与产品化（Week 5）

| 任务       | 交付物                                         | 验收标准                       |
|------------|------------------------------------------------|--------------------------------|
| 本地终端   | portable-pty 集成                              | 可执行 Shell 命令，基础输出显示 |
| 主题系统   | 暗色/亮色主题                                  | 所有组件颜色随主题切换         |
| 跨平台打包 | macOS .app + DMG，Windows .exe，Linux AppImage | 各平台双击运行                 |
| 性能优化   | 增量渲染、内存优化                             | 60fps 滚动，内存 < 100MB       |

## 十一、验收标准

### 11.1 功能验收

| ID    | 验收项                                              | 验收方法                         |
|-------|-----------------------------------------------------|----------------------------------|
| AC-01 | 双击图标 300ms 内显示窗口                           | 手动测试 + 日志计时              |
| AC-02 | 项目选择器正确显示最近项目                          | 创建 3 个项目，验证列表顺序      |
| AC-03 | 消息历史区正确渲染 Markdown、代码块、Diff、Progress | 构造包含各类型块的消息，目视检查 |
| AC-04 | 输入框 / 命令补全、@ 文件引用补全准确               | 输入 / 和 @，验证补全列表        |
| AC-05 | Enter 发送、Shift+Enter 换行、! 中断行为正确        | 手动操作验证                     |
| AC-06 | 底部 7 个标签可 Ctrl+1~7 切换，内容独立渲染         | 逐一切换，验证内容正确性         |
| AC-07 | 状态栏实时更新连接状态、Token、分支、模式           | 观察状态栏随操作变化             |
| AC-08 | Permission 通知常驻，y/n 响应正确                   | 触发权限请求，验证响应流程       |
| AC-09 | 流水线阶段点击跳转至 Spec 标签                      | 点击"实现"阶段，验证跳转         |
| AC-10 | Terminal 标签可执行本地命令                         | 输入 echo hello，验证输出        |

### 11.2 性能验收

| ID    | 指标               | 测试方法                              |
|-------|--------------------|---------------------------------------|
| AC-11 | 冷启动时间 ≤ 300ms | time ./apex 测量 10 次取平均          |
| AC-12 | 消息滚动 60fps     | 1000 条消息，滚轮快速滚动，目测无卡顿 |
| AC-13 | 输入延迟 ≤ 16ms    | 键盘事件到字符显示，日志测量          |
| AC-14 | 空闲内存 ≤ 100MB   | ps / Activity Monitor 观察            |
| AC-15 | 单文件体积 ≤ 30MB  | ls -lh 检查 Release 构建产物          |

### 11.3 兼容性验收

| ID    | 平台                              | 测试环境                          |
|-------|-----------------------------------|-----------------------------------|
| AC-16 | macOS 12+ (Intel & Apple Silicon) | macOS 14, M1 & x86_64             |
| AC-17 | Windows 10+                       | Windows 11, x86_64                |
| AC-18 | Ubuntu 22.04+                     | Ubuntu 22.04 & 24.04, x86_64      |
| AC-19 | 配置文件跨平台兼容                | 在 A 平台创建配置，B 平台读取验证 |

## 附录：参考实现片段

### A.1 主 UI 路由函数

```rust
// src/ui/mod.rs
pub fn ui(frame: &mut Frame, app: &App) {
    let main = frame.area();

    // 垂直七层布局
    let layers = Layout::vertical([
        Constraint::Length(1),      // 导航栏
        Constraint::Length(1),      // 流水线
        Constraint::Min(10),        // 主内容
        Constraint::Length(app.input.height()), // 输入区（动态高度）
        Constraint::Length(1),      // 状态栏
        Constraint::Length(1),      // 标签栏
    ]).split(main);
    
    // 渲染固定层
    render_nav_bar(frame, layers[0], app);
    render_pipeline(frame, layers[1], app);
    render_status_bar(frame, layers[4], app);
    render_tab_bar(frame, layers[5], app);
    
    // 渲染动态内容层
    match app.current_tab {
        TabId::Session => render_session(frame, layers[2], &app.session),
        TabId::Spec => render_spec(frame, layers[2], &app.spec),
        TabId::Activity => render_activity(frame, layers[2], &app.activity),
        TabId::Dag => render_dag(frame, layers[2], &app.dag),
        TabId::Memory => render_memory(frame, layers[2], &app.memory),
        TabId::Checkpoint => render_checkpoint(frame, layers[2], &app.checkpoints),
        TabId::Terminal => render_terminal(frame, layers[2], &app.terminal),
    }
    
    // 渲染输入区（所有标签共享）
    render_input_area(frame, layers[3], &app.input);
    
    // 渲染通知浮层（叠加层）
    if let Some(notif) = app.notifications.front() {
        let notif_area = calculate_notification_area(main, notif);
        render_notification(frame, notif_area, notif);
    }
}
```

### A.2 项目选择器界面

```rust
// src/ui/project_selector.rs
pub fn render_project_selector(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" 选择项目 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    // 最近项目列表
    let items: Vec<ListItem> = app.recent_projects.iter().map(|p| {
        ListItem::new(format!("{}  {}", p.name, p.path.display()))
            .style(Style::default().fg(Color::White))
    }).collect();
    
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::Cyan))
        .highlight_symbol("▶ ");
    
    frame.render_stateful_widget(list, inner, &mut app.project_list_state);
    
    // 底部提示
    let hint = "Enter 确认 | N 新项目 | D 删除选中";
    frame.render_widget(
        Paragraph::new(hint).alignment(Alignment::Center).style(Style::default().fg(Color::Gray)),
        Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
    );
}
```

## 补充设计：编码规范与 AI 模型配置

### 一、编码规范：严格封装原则

#### 1.1 核心规则

| 规则                   | 说明                                                     | 违反后果                      |
|------------------------|----------------------------------------------------------|-------------------------------|
| **禁止 `pub` 字段**    | 所有 `struct` 字段必须为私有（默认或显式 `priv`）        | 编译错误（通过 CI lint 拦截） |
| **强制 Getter/Setter** | 所有外部访问必须通过方法，禁止直接字段访问               | Code Review 不通过            |
| **宏自动生成**         | 使用自定义 derive 宏或声明宏生成访问器，禁止手写重复代码 | 减少样板代码，统一命名规范    |
| **不可变性优先**       | Setter 仅在必要时提供，优先通过构造函数一次性构建        | 降低状态突变带来的 Bug        |

#### 1.2 命名规范

| 类型                 | 命名                                               | 示例                                      |
|----------------------|----------------------------------------------------|-------------------------------------------|
| Getter（不可变借用） | `field_name()`                                     | `fn name(&self) -> &str`                  |
| Getter（克隆返回）   | `field_name_owned()` 或 `field_name().to_string()` | `fn name(&self) -> String`                |
| Setter               | `set_field_name(val)`                              | `fn set_name(&mut self, val: String)`     |
| Builder 风格         | `with_field_name(val)`                             | `fn with_name(self, val: String) -> Self` |

### 二、Getter/Setter 宏实现方案

#### 2.1 方案选型：自定义 Derive 宏（推荐）

在 Workspace 中新增 apex-macros crate，提供 #[derive(Getters, Setters)]。

```plain
apex-workspace/
├── Cargo.toml
├── crates/
│   ├── apex-tui/              # 主应用
│   ├── apex-core/             # 业务逻辑
│   └── apex-macros/           # 过程宏（proc-macro）
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
```

crates/apex-macros/Cargo.toml：

```toml
[package]
name = "apex-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "derive"] }
convert_case = "0.6"  # 用于 snake_case / camelCase 转换
```

crates/apex-macros/src/lib.rs：

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// 为结构体所有字段生成不可变 getter
#[proc_macro_derive(Getters)]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let getters = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    let field_ty = &f.ty;
                    
                    // 处理 Option<T> 类型：返回 &Option<T>
                    // 处理 Vec<T> 类型：返回 &[T]
                    let return_ty = quote! { &#field_ty };
                    
                    quote! {
                        #[inline]
                        #[must_use]
                        pub fn #field_name(&self) -> #return_ty {
                            &self.#field_name
                        }
                    }
                }).collect::<Vec<_>>()
            }
            _ => panic!("Getters only supports named fields"),
        },
        _ => panic!("Getters only supports structs"),
    };
    
    let expanded = quote! {
        impl #name {
            #(#getters)*
        }
    };
    
    TokenStream::from(expanded)
}

/// 为结构体所有字段生成 setter
/// 支持 #[setter(skip)] 属性跳过特定字段
#[proc_macro_derive(Setters, attributes(setter))]
pub fn derive_setters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let setters = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields.named.iter().filter_map(|f| {
                    let field_name = &f.ident;
                    let field_ty = &f.ty;
                    
                    // 检查是否有 #[setter(skip)] 属性
                    let skip = f.attrs.iter().any(|attr| {
                        attr.path().is_ident("setter") &&
                        attr.parse_args::<syn::Ident>().map(|i| i == "skip").unwrap_or(false)
                    });
                    
                    if skip {
                        return None;
                    }
                    
                    let setter_name = syn::Ident::new(
                        &format!("set_{}", field_name.as_ref().unwrap()),
                        field_name.span(),
                    );
                    
                    Some(quote! {
                        #[inline]
                        pub fn #setter_name(&mut self, val: #field_ty) -> &mut Self {
                            self.#field_name = val;
                            self
                        }
                    })
                }).collect::<Vec<_>>()
            }
            _ => panic!("Setters only supports named fields"),
        },
        _ => panic!("Setters only supports structs"),
    };
    
    let expanded = quote! {
        impl #name {
            #(#setters)*
        }
    };
    
    TokenStream::from(expanded)
}

/// 为结构体生成 Builder 风格的 with_ 方法（消费 self，返回 Self）
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let builders = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    let field_ty = &f.ty;
                    let with_name = syn::Ident::new(
                        &format!("with_{}", field_name.as_ref().unwrap()),
                        field_name.span(),
                    );
                    
                    quote! {
                        #[inline]
                        #[must_use]
                        pub fn #with_name(mut self, val: #field_ty) -> Self {
                            self.#field_name = val;
                            self
                        }
                    }
                }).collect::<Vec<_>>()
            }
            _ => panic!("Builder only supports named fields"),
        },
        _ => panic!("Builder only supports structs"),
    };
    
    let expanded = quote! {
        impl #name {
            #(#builders)*
        }
    };
    
    TokenStream::from(expanded)
}
```

#### 2.2 使用示例

```rust
// crates/apex-core/src/models/message.rs
use apex_macros::{Getters, Setters, Builder};

/// 消息实体
#[derive(Debug, Clone, Getters, Setters, Builder)]
pub struct Message {
    // 私有字段，无 pub 修饰
    id: String,
    turn_id: String,
    role: MessageRole,
    timestamp: chrono::DateTime<chrono::Local>,
    blocks: Vec<MessageBlock>,
    status: MessageStatus,

    // 该字段禁止外部修改（如数据库自增或内部生成）
    #[setter(skip)]
    created_at: chrono::DateTime<chrono::Utc>,
}

// 自动生成的代码等价于：
// impl Message {
//     #[inline] #[must_use] pub fn id(&self) -> &String { &self.id }
//     #[inline] #[must_use] pub fn turn_id(&self) -> &String { &self.turn_id }
//     #[inline] #[must_use] pub fn role(&self) -> &MessageRole { &self.role }
//     // ... 所有 getter
//     
//     #[inline] pub fn set_id(&mut self, val: String) -> &mut Self { self.id = val; self }
//     #[inline] pub fn set_turn_id(&mut self, val: String) -> &mut Self { self.turn_id = val; self }
//     // ... 除 created_at 外的所有 setter
//     
//     #[inline] #[must_use] pub fn with_id(mut self, val: String) -> Self { self.id = val; self }
//     // ... 所有 builder 方法
// }

// 使用方式
let msg = Message::default()
    .with_id("msg-001".to_string())
    .with_turn_id("T-12".to_string())
    .with_role(MessageRole::User);

// 访问
println!("{}", msg.id()); // 不可变借用
println!("{}", msg.turn_id());

// 修改
msg.set_status(MessageStatus::Complete);
```

#### 2.3 特殊类型处理

对于 Vec<T> 和 Option<T>，提供扩展宏以返回更友好的类型：

```rust
// crates/apex-macros/src/lib.rs（扩展）
#[proc_macro_derive(GettersExt)]
pub fn derive_getters_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let methods = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields.named.iter().filter_map(|f| {
                    let field_name = f.ident.as_ref()?;
                    let field_ty = &f.ty;
                    
                    // 解析类型路径，检查是否为 Vec<T> 或 Option<T>
                    let type_str = quote!(#field_ty).to_string();
                    
                    if type_str.starts_with("Vec ") {
                        let method_name = syn::Ident::new(
                            &format!("{}_slice", field_name),
                            field_name.span(),
                        );
                        Some(quote! {
                            #[inline]
                            #[must_use]
                            pub fn #method_name(&self) -> &[#field_ty] {
                                &self.#field_name
                            }
                        })
                    } else if type_str.starts_with("Option ") {
                        let method_name = syn::Ident::new(
                            &format!("{}_as_ref", field_name),
                            field_name.span(),
                        );
                        Some(quote! {
                            #[inline]
                            #[must_use]
                            pub fn #method_name(&self) -> Option<&#field_ty> {
                                self.#field_name.as_ref()
                            }
                        })
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            }
            _ => vec![],
        },
        _ => vec![],
    };
    
    let expanded = quote! {
        impl #name {
            #(#methods)*
        }
    };
    
    TokenStream::from(expanded)
}
```

#### 2.4 CI 拦截规则

在 clippy.toml 和 CI 脚本中增加检查，禁止 pub 字段：

```toml
# clippy.toml
# 自定义 lint：禁止结构体公共字段（通过 deny 级别）
# 注：标准 clippy 无此 lint，需配合 cargo-deny 或自定义脚本
```

```yaml
# .github/workflows/ci.yml
- name: Check for pub fields in structs
  run: |
    # 扫描 src/ 目录中 struct 定义的 pub 字段（排除 pub(crate) 和宏生成代码）
    if grep -rn "^\s*pub\s\+\w\+:\s" src/ --include="*.rs" | grep -v "generated" | grep -v "mod.rs"; then
      echo "ERROR: Found pub fields in structs. Use #[derive(Getters, Setters)] instead."
      exit 1
    fi
```

### 三、AI 模型服务商配置

#### 3.1 配置结构（TOML）

支持多服务商配置，用户可切换不同 Provider，每个 Provider 独立管理 API Key、Base URL、模型列表。

```toml
# ~/.config/apex/config.toml

[ui]
theme = "dark"
font_family = "JetBrains Mono"
font_size = 14
line_height = 1.2
cursor_style = "block"

# === AI 模型服务商配置 ===

[[providers]]
name = "anthropic"                    # 内部标识，用于切换
display_name = "Anthropic Claude"     # UI 显示名称
enabled = true
base_url = "https://api.anthropic.com/v1"
api_key = "sk-ant-api03-xxxxxxxx"     # 支持环境变量引用："$ANTHROPIC_API_KEY"
default_model = "claude-opus-4-8"
timeout_seconds = 120
max_retries = 3

[providers.models]
default = "claude-opus-4-8"
available = [
{ id = "claude-opus-4-8", name = "Claude Opus 4.8", max_tokens = 200000, context_window = 200000 },
{ id = "claude-sonnet-4-8", name = "Claude Sonnet 4.8", max_tokens = 200000, context_window = 200000 },
{ id = "claude-haiku-4-8", name = "Claude Haiku 4.8", max_tokens = 8000, context_window = 48000 },
]

[providers.headers]
# 自定义 HTTP 请求头
"x-api-key" = "$ANTHROPIC_API_KEY"

[[providers]]
name = "openai"
display_name = "OpenAI"
enabled = false
base_url = "https://api.openai.com/v1"
api_key = "$OPENAI_API_KEY"
default_model = "gpt-4o"
timeout_seconds = 60
max_retries = 3

[providers.models]
default = "gpt-4o"
available = [
{ id = "gpt-4o", name = "GPT-4o", max_tokens = 128000, context_window = 128000 },
{ id = "gpt-4o-mini", name = "GPT-4o Mini", max_tokens = 128000, context_window = 128000 },
]

[[providers]]
name = "local"
display_name = "本地 Ollama"
enabled = false
base_url = "http://localhost:11434/v1"
api_key = ""                          # 本地服务通常无需 key
default_model = "codellama:34b"
timeout_seconds = 300
max_retries = 1

[providers.models]
default = "codellama:34b"
available = [
{ id = "codellama:34b", name = "CodeLlama 34B", max_tokens = 16000, context_window = 16000 },
{ id = "deepseek-coder:33b", name = "DeepSeek Coder 33B", max_tokens = 16000, context_window = 16000 },
]

# === 全局 Agent 配置 ===

[agent]
active_provider = "anthropic"         # 当前使用的服务商名称
mode = "ask"
temperature = 0.7
top_p = 1.0

[agent.context]
auto_include_git_status = true
max_file_size_kb = 100
excluded_patterns = [".git", "target", "node_modules", "*.lock", ".env"]

[agent.safety]
dangerous_commands = ["rm -rf", "dd if=", "mkfs", ">:dev>null"]
require_confirmation = true
```

#### 3.2 数据模型（严格封装）

```rust
// crates/apex-core/src/config/mod.rs
use apex_macros::{Getters, Setters, Builder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 应用总配置
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct AppConfig {
    ui: UiConfig,
    providers: Vec<ProviderConfig>,
    agent: AgentConfig,
}

impl AppConfig {
    /// 获取当前激活的 Provider
    pub fn active_provider(&self) -> Option<&ProviderConfig> {
        let active_name = self.agent.active_provider();
        self.providers.iter().find(|p| p.name() == active_name)
    }

    /// 切换 Provider
    pub fn switch_provider(&mut self, name: &str) -> Result<(), ConfigError> {
        if !self.providers.iter().any(|p| p.name() == name) {
            return Err(ConfigError::ProviderNotFound(name.to_string()));
        }
        self.agent.set_active_provider(name.to_string());
        Ok(())
    }
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct UiConfig {
    theme: String,
    font_family: String,
    font_size: u16,
    line_height: f32,
    cursor_style: String,
}

/// AI 服务商配置
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct ProviderConfig {
    name: String,                // 内部标识，如 "anthropic"
    display_name: String,        // UI 显示名称
    enabled: bool,
    base_url: String,

    #[setter(skip)]              // API Key 通过专用方法设置，支持加密存储
    api_key: String,

    default_model: String,
    timeout_seconds: u64,
    max_retries: u32,
    models: ModelList,
    headers: HashMap<String, String>,
}

impl ProviderConfig {
    /// 获取解密后的 API Key（从 keyring 或环境变量解析）
    pub fn resolved_api_key(&self) -> Result<String, ConfigError> {
        let key = self.api_key();

        // 支持环境变量引用："$ANTHROPIC_API_KEY"
        if key.starts_with('$') {
            let var_name = &key[1..];
            std::env::var(var_name).map_err(|_| {
                ConfigError::MissingEnvVar(var_name.to_string())
            })
        } else {
            Ok(key.clone())
        }
    }

    /// 安全设置 API Key（可扩展为写入系统 keyring）
    pub fn set_api_key_secure(&mut self, key: String) {
        self.set_api_key(key);
    }
}

/// 模型列表
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct ModelList {
    default: String,
    available: Vec<ModelInfo>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
pub struct ModelInfo {
    id: String,
    name: String,
    max_tokens: u64,
    context_window: u64,
}

/// Agent 行为配置
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct AgentConfig {
    active_provider: String,
    mode: String,
    temperature: f32,
    top_p: f32,
    context: ContextConfig,
    safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct ContextConfig {
    auto_include_git_status: bool,
    max_file_size_kb: u64,
    excluded_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
pub struct SafetyConfig {
    dangerous_commands: Vec<String>,
    require_confirmation: bool,
}

/// 配置错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),
}
```

#### 3.3 配置加载与验证

```rust
// crates/apex-core/src/config/loader.rs
use std::path::PathBuf;

pub struct ConfigLoader;

impl ConfigLoader {
    /// 加载配置，按优先级合并：
    /// 1. 系统默认（编译时嵌入）
    /// 2. ~/.config/apex/config.toml（用户级）
    /// 3. <project>/.apex/config.toml（项目级，覆盖用户级）
    pub fn load(project_path: Option<&PathBuf>) -> Result<AppConfig, ConfigError> {
        let mut config = Self::load_defaults();

        // 加载用户级配置
        if let Some(user_config) = Self::user_config_path() {
            if user_config.exists() {
                let content = std::fs::read_to_string(&user_config)?;
                let user: AppConfig = toml::from_str(&content)?;
                config = Self::merge(config, user);
            }
        }
        
        // 加载项目级配置
        if let Some(path) = project_path {
            let project_config = path.join(".apex").join("config.toml");
            if project_config.exists() {
                let content = std::fs::read_to_string(&project_config)?;
                let project: AppConfig = toml::from_str(&content)?;
                config = Self::merge(config, project);
            }
        }
        
        // 验证：检查 active_provider 是否存在且 enabled
        config.validate()?;
        
        Ok(config)
    }
    
    fn validate(config: &AppConfig) -> Result<(), ConfigError> {
        let active = config.agent().active_provider();
        
        match config.providers().iter().find(|p| p.name() == active) {
            None => return Err(ConfigError::ProviderNotFound(active.clone())),
            Some(p) if !p.enabled() => {
                return Err(ConfigError::ProviderDisabled(active.clone()));
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("apex").join("config.toml"))
    }
    
    fn load_defaults() -> AppConfig {
        // 编译时嵌入默认配置
        let default_toml = include_str!("../../../assets/default-config.toml");
        toml::from_str(default_toml).expect("Default config must be valid")
    }
    
    fn merge(base: AppConfig, override_: AppConfig) -> AppConfig {
        // 深度合并逻辑：override_ 的非空字段覆盖 base
        // 对于 Vec（如 providers），按 name 合并而非完全替换
        // ... 实现略
        base
    }
}
```

#### 3.4 默认配置文件（编译时嵌入）

```toml
# assets/default-config.toml
[ui]
theme = "dark"
font_family = "JetBrains Mono"
font_size = 14
line_height = 1.2
cursor_style = "block"

[[providers]]
name = "anthropic"
display_name = "Anthropic Claude"
enabled = true
base_url = "https://api.anthropic.com/v1"
api_key = ""
default_model = "claude-opus-4-8"
timeout_seconds = 120
max_retries = 3

[providers.models]
default = "claude-opus-4-8"
available = [
{ id = "claude-opus-4-8", name = "Claude Opus 4.8", max_tokens = 200000, context_window = 200000 },
]

[agent]
active_provider = "anthropic"
mode = "ask"
temperature = 0.7
top_p = 1.0

[agent.context]
auto_include_git_status = true
max_file_size_kb = 100
excluded_patterns = [".git", "target", "node_modules", "*.lock"]

[agent.safety]
dangerous_commands = ["rm -rf", "dd if=", "mkfs"]
require_confirmation = true
```

### 四、状态栏更新（展示当前 Provider）

```rust
// src/ui/status_bar.rs
pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let config = app.config();
    let provider = config.active_provider()
        .map(|p| p.display_name().as_str())
        .unwrap_or("No Provider");

    let left = format!("● {}    model {}    provider {}", 
        app.connection_status(),
        app.session().model(),
        provider,
    );
    
    let center = format!("tokens {}/{}    cache {}%",
        format_tokens(app.session().token_usage().input()),
        format_tokens(app.session().token_usage().limit()),
        app.session().token_usage().cache_hit_rate(),
    );
    
    let right = format!("branch {}    mode {}    session {}    workspace {}",
        app.project().map(|p| p.git_branch()).unwrap_or("none"),
        config.agent().mode(),
        app.session().id_short(),
        app.project().map(|p| p.path().display().to_string()).unwrap_or_default(),
    );
    
    // 渲染三列...
}
```

### 五、更新后的 Cargo Workspace 结构

```toml
# Cargo.toml（Workspace Root）
[workspace]
members = [
    "crates/apex-tui",
    "crates/apex-core",
    "crates/apex-macros",
]
resolver = "2"

[workspace.dependencies]
# 共享依赖版本管理
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
ratatui = "0.29"
winit = "0.30"
softbuffer = "0.4"
cosmic-text = "0.12"
portable-pty = "0.9"
reqwest = { version = "0.12", features = ["json", "stream"] }
futures = "0.3"
dirs = "5.0"

# 内部 crate
apex-core = { path = "crates/apex-core" }
apex-macros = { path = "crates/apex-macros" }
```

### 六、编码规范检查清单

| 检查项                              | 工具/方法                 | 频率    |
| -------------------------------- | --------------------- | ----- |
| 无 `pub` 字段                       | `grep` + CI 脚本        | 每次 PR |
| 使用 `#[derive(Getters, Setters)]` | Code Review           | 每次 PR |
| 无 `unwrap()` / `expect()`（生产代码）  | `clippy::unwrap_used` | 每次构建  |
| 所有错误类型实现 `thiserror::Error`      | Code Review           | 每次 PR |
| 所有公开 API 有文档注释                   | `cargo doc` + CI      | 每次 PR |
| 通过 `cargo fmt`                   | `rustfmt`             | 每次提交  |
| 通过 `cargo clippy`                | clippy                | 每次提交  |

以上补充完整覆盖了：

1. 严格封装规范：通过 apex-macros crate 的 Derive 宏自动生成 Getter/Setter，零 pub 字段
2. AI 模型配置：多 Provider 支持、环境变量引用 API Key、模型元数据、安全策略
3. 配置加载：三层合并（默认 → 用户级 → 项目级）+ 运行时验证
4. CI 拦截：通过脚本禁止 pub 字段提交到仓库

## 补充设计：Lombok 风格复合宏系统

### 一、设计目标

| 目标            | 说明                                                                            |
|-----------------|---------------------------------------------------------------------------------|
| **复合宏**      | `#[derive(Data)]` 一键生成 getter + setter + builder + to\_string + constructor |
| **细粒度控制**  | 通过结构体属性 `#[data(...)]` 精确控制生成内容                                  |
| **字段级控制**  | 通过字段属性 `#[data(skip_setter)]` 等控制单个字段                              |
| **零 pub 字段** | 所有字段私有，完全通过方法访问                                                  |
| **Rust 惯用法** | 遵循 Rust 命名约定和所有权语义                                                  |

### 二、宏功能矩阵

#### 2.1 单独 Derive 宏

| Derive 宏     | 生成内容                               | 等价 Lombok           |
|---------------|----------------------------------------|-----------------------|
| `Getters`     | 所有字段的不可变 getter                | `@Getter`             |
| `Setters`     | 所有字段的可变 setter                  | `@Setter`             |
| `Builder`     | `with_xxx(self, val) -> Self` 链式构造 | `@Builder` (简化版)   |
| `Constructor` | `new(field1, field2, ...)` 全字段构造  | `@AllArgsConstructor` |
| `ToString`    | `to_string() -> String` 格式化输出     | `@ToString`           |
| `Equals`      | `eq(&self, other) -> bool` + `hash()`  | `@EqualsAndHashCode`  |
| `Data`        | **以上全部**                           | `@Data`               |

#### 2.2 复合宏 Data 的默认行为

```rust
#[derive(Data)]
#[data(
    getters = true,      // 生成 getter
    setters = true,      // 生成 setter
    builder = true,      // 生成 builder (with_xxx)
    constructor = true,  // 生成 new()
    to_string = true,    // 生成 to_string()
    equals = true,       // 生成 PartialEq + Hash
)]
struct Message {
    id: String,
    content: String,
}
```

自动生成代码等价于：

```rust
impl Message {
    // === Constructor ===
    pub fn new(id: String, content: String) -> Self {
        Self { id, content }
    }

    // === Getters ===
    #[inline] #[must_use]
    pub fn id(&self) -> &String { &self.id }

    #[inline] #[must_use]
    pub fn content(&self) -> &String { &self.content }

    // === Setters ===
    #[inline]
    pub fn set_id(&mut self, val: String) -> &mut Self {
        self.id = val;
        self
    }

    #[inline]
    pub fn set_content(&mut self, val: String) -> &mut Self {
        self.content = val;
        self
    }

    // === Builder ===
    #[inline] #[must_use]
    pub fn with_id(mut self, val: String) -> Self {
        self.id = val;
        self
    }

    #[inline] #[must_use]
    pub fn with_content(mut self, val: String) -> Self {
        self.content = val;
        self
    }
}

// === ToString ===
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message(id={}, content={})", self.id, self.content)
    }
}

// === Equals ===
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.content == other.content
    }
}
impl Eq for Message {}
impl std::hash::Hash for Message {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.content.hash(state);
    }
}
```

### 三、完整宏实现

#### 3.1 apex-macros crate 结构

```plain
crates/apex-macros/
├── Cargo.toml
└── src/
    ├── lib.rs              # 导出所有 derive 宏
    ├── data.rs             # #[derive(Data)] 复合宏
    ├── getters.rs          # #[derive(Getters)]
    ├── setters.rs          # #[derive(Setters)]
    ├── builder.rs          # #[derive(Builder)]
    ├── constructor.rs      # #[derive(Constructor)]
    ├── to_string.rs        # #[derive(ToString)]
    ├── equals.rs           # #[derive(Equals)]
    └── utils.rs            # 共享工具函数
```

#### 3.2 lib.rs — 宏入口

```rust
// crates/apex-macros/src/lib.rs
use proc_macro::TokenStream;

mod builder;
mod constructor;
mod data;
mod equals;
mod getters;
mod setters;
mod to_string;
mod utils;

/// 复合宏：一键生成 getter + setter + builder + constructor + to_string + equals
///
/// # 示例
/// ```
/// #[derive(Data)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let user = User::new("Alice".into(), 30);
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.to_string(), "User(name=Alice, age=30)");
/// ```
#[proc_macro_derive(Data, attributes(data))]
pub fn derive_data(input: TokenStream) -> TokenStream {
    data::derive(input)
}

/// 单独生成所有字段的不可变 getter
#[proc_macro_derive(Getters, attributes(getter))]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    getters::derive(input)
}

/// 单独生成所有字段的可变 setter（返回 &mut Self 支持链式调用）
#[proc_macro_derive(Setters, attributes(setter))]
pub fn derive_setters(input: TokenStream) -> TokenStream {
    setters::derive(input)
}

/// 生成 builder 风格的 with_xxx(self, val) -> Self 方法
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    builder::derive(input)
}

/// 生成全字段构造函数 new(field1, field2, ...)
#[proc_macro_derive(Constructor, attributes(constructor))]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    constructor::derive(input)
}

/// 生成 Display 实现（to_string）
#[proc_macro_derive(ToString, attributes(to_string))]
pub fn derive_to_string(input: TokenStream) -> TokenStream {
    to_string::derive(input)
}

/// 生成 PartialEq + Eq + Hash 实现
#[proc_macro_derive(Equals, attributes(equals))]
pub fn derive_equals(input: TokenStream) -> TokenStream {
    equals::derive(input)
}
```

#### 3.3 data.rs — 复合宏核心

```rust
// crates/apex-macros/src/data.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

/// Data 宏的属性配置
#[derive(Debug, Default)]
struct DataConfig {
    getters: bool,
    setters: bool,
    builder: bool,
    constructor: bool,
    to_string: bool,
    equals: bool,
}

impl DataConfig {
    fn all_enabled() -> Self {
        Self {
            getters: true,
            setters: true,
            builder: true,
            constructor: true,
            to_string: true,
            equals: true,
        }
    }
}

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析结构体级别的 #[data(...)] 属性
    let mut config = DataConfig::all_enabled();
    
    for attr in &input.attrs {
        if attr.path().is_ident("data") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
            ) {
                for meta in nested {
                    match meta {
                        Meta::Path(path) if path.is_ident("no_getters") => config.getters = false,
                        Meta::Path(path) if path.is_ident("no_setters") => config.setters = false,
                        Meta::Path(path) if path.is_ident("no_builder") => config.builder = false,
                        Meta::Path(path) if path.is_ident("no_constructor") => config.constructor = false,
                        Meta::Path(path) if path.is_ident("no_to_string") => config.to_string = false,
                        Meta::Path(path) if path.is_ident("no_equals") => config.equals = false,
                        Meta::NameValue(nv) if nv.path.is_ident("getters") => {
                            config.getters = parse_bool(&nv.value);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("setters") => {
                            config.setters = parse_bool(&nv.value);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("builder") => {
                            config.builder = parse_bool(&nv.value);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("constructor") => {
                            config.constructor = parse_bool(&nv.value);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("to_string") => {
                            config.to_string = parse_bool(&nv.value);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("equals") => {
                            config.equals = parse_bool(&nv.value);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Data derive only supports named fields"),
        },
        _ => panic!("Data derive only supports structs"),
    };
    
    // 收集字段信息（过滤掉被标记 skip 的）
    let field_infos: Vec<_> = fields.iter().filter_map(|f| {
        let name = f.ident.as_ref()?;
        let ty = &f.ty;
        
        // 解析字段级属性
        let mut skip_getter = false;
        let mut skip_setter = false;
        let mut skip_builder = false;
        let mut skip_to_string = false;
        let mut skip_equals = false;
        let mut getter_return_type: Option<syn::Type> = None;
        
        for attr in &f.attrs {
            if attr.path().is_ident("data") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
                ) {
                    for meta in nested {
                        match meta {
                            Meta::Path(path) if path.is_ident("skip") => {
                                skip_getter = true;
                                skip_setter = true;
                                skip_builder = true;
                                skip_to_string = true;
                                skip_equals = true;
                            }
                            Meta::Path(path) if path.is_ident("skip_getter") => skip_getter = true,
                            Meta::Path(path) if path.is_ident("skip_setter") => skip_setter = true,
                            Meta::Path(path) if path.is_ident("skip_builder") => skip_builder = true,
                            Meta::Path(path) if path.is_ident("skip_to_string") => skip_to_string = true,
                            Meta::Path(path) if path.is_ident("skip_equals") => skip_equals = true,
                            Meta::NameValue(nv) if nv.path.is_ident("getter_type") => {
                                // 自定义 getter 返回类型
                                if let syn::Expr::Path(expr_path) = &nv.value {
                                    // 解析类型...
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        Some(FieldInfo {
            name: name.clone(),
            ty: ty.clone(),
            skip_getter,
            skip_setter,
            skip_builder,
            skip_to_string,
            skip_equals,
            getter_return_type,
        })
    }).collect();
    
    // === 生成 Constructor ===
    let constructor_impl = if config.constructor {
        let ctor_params = field_infos.iter().map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            quote! { #name: #ty }
        });
        let ctor_fields = field_infos.iter().map(|f| {
            let name = &f.name;
            quote! { #name }
        });
        
        Some(quote! {
            #[allow(clippy::too_many_arguments)]
            pub fn new(#(#ctor_params),*) -> Self {
                Self { #(#ctor_fields),* }
            }
        })
    } else {
        None
    };
    
    // === 生成 Getters ===
    let getter_impls = if config.getters {
        field_infos.iter().filter(|f| !f.skip_getter).map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            
            // 智能返回类型：Option<T> -> Option<&T>, Vec<T> -> &[T]
            let return_ty = if is_option_type(ty) {
                quote! { Option<&#ty> }
            } else if is_vec_type(ty) {
                quote! { &[#ty] }
            } else {
                quote! { &#ty }
            };
            
            quote! {
                #[inline]
                #[must_use]
                pub fn #name(&self) -> #return_ty {
                    &self.#name
                }
            }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };
    
    // === 生成 Setters ===
    let setter_impls = if config.setters {
        field_infos.iter().filter(|f| !f.skip_setter).map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            let setter_name = syn::Ident::new(&format!("set_{}", name), name.span());
            
            quote! {
                #[inline]
                pub fn #setter_name(&mut self, val: #ty) -> &mut Self {
                    self.#name = val;
                    self
                }
            }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };
    
    // === 生成 Builder (with_xxx) ===
    let builder_impls = if config.builder {
        field_infos.iter().filter(|f| !f.skip_builder).map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            let with_name = syn::Ident::new(&format!("with_{}", name), name.span());
            
            quote! {
                #[inline]
                #[must_use]
                pub fn #with_name(mut self, val: #ty) -> Self {
                    self.#name = val;
                    self
                }
            }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };
    
    // === 生成 ToString ===
    let to_string_impl = if config.to_string {
        let to_string_fields = field_infos.iter().filter(|f| !f.skip_to_string).map(|f| {
            let name = &f.name;
            let name_str = name.to_string();
            quote! { .field(#name_str, &self.#name) }
        });
        
        Some(quote! {
            impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!(#name))
                        #(#to_string_fields)*
                        .finish()
                }
            }
        })
    } else {
        None
    };
    
    // === 生成 Equals ===
    let equals_impl = if config.equals {
        let eq_fields = field_infos.iter().filter(|f| !f.skip_equals).map(|f| {
            let name = &f.name;
            quote! { self.#name == other.#name }
        });
        let hash_fields = field_infos.iter().filter(|f| !f.skip_equals).map(|f| {
            let name = &f.name;
            quote! { self.#name.hash(state); }
        });
        
        Some(quote! {
            impl #impl_generics PartialEq for #name #ty_generics #where_clause {
                fn eq(&self, other: &Self) -> bool {
                    true #(&& #eq_fields)*
                }
            }
            impl #impl_generics Eq for #name #ty_generics #where_clause {}
            impl #impl_generics std::hash::Hash for #name #ty_generics #where_clause {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    #(#hash_fields)*
                }
            }
        })
    } else {
        None
    };
    
    // 组合所有实现
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #constructor_impl
            #(#getter_impls)*
            #(#setter_impls)*
            #(#builder_impls)*
        }
        
        #to_string_impl
        #equals_impl
    };
    
    TokenStream::from(expanded)
}

struct FieldInfo {
    name: syn::Ident,
    ty: syn::Type,
    skip_getter: bool,
    skip_setter: bool,
    skip_builder: bool,
    skip_to_string: bool,
    skip_equals: bool,
    getter_return_type: Option<syn::Type>,
}

fn parse_bool(expr: &syn::Expr) -> bool {
    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) = expr {
        b.value()
    } else {
        true
    }
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "Option";
        }
    }
    false
}

fn is_vec_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "Vec";
        }
    }
    false
}
```

#### 3.4 其他单独宏（简化版）

```rust
// crates/apex-macros/src/getters.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let getters = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields.named.iter().filter_map(|f| {
                    let name = f.ident.as_ref()?;
                    let ty = &f.ty;
                    
                    // 检查 #[getter(skip)] 属性
                    let skip = f.attrs.iter().any(|attr| {
                        attr.path().is_ident("getter") &&
                        attr.parse_args::<syn::Ident>()
                            .map(|i| i == "skip")
                            .unwrap_or(false)
                    });
                    
                    if skip { return None; }
                    
                    Some(quote! {
                        #[inline]
                        #[must_use]
                        pub fn #name(&self) -> &#ty {
                            &self.#name
                        }
                    })
                }).collect::<Vec<_>>()
            }
            _ => panic!("Getters only supports named fields"),
        },
        _ => panic!("Getters only supports structs"),
    };
    
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#getters)*
        }
    };
    
    TokenStream::from(expanded)
}
```

```rust
// crates/apex-macros/src/setters.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let setters = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields.named.iter().filter_map(|f| {
                    let name = f.ident.as_ref()?;
                    let ty = &f.ty;
                    let setter_name = syn::Ident::new(&format!("set_{}", name), name.span());
                    
                    let skip = f.attrs.iter().any(|attr| {
                        attr.path().is_ident("setter") &&
                        attr.parse_args::<syn::Ident>()
                            .map(|i| i == "skip")
                            .unwrap_or(false)
                    });
                    
                    if skip { return None; }
                    
                    Some(quote! {
                        #[inline]
                        pub fn #setter_name(&mut self, val: #ty) -> &mut Self {
                            self.#name = val;
                            self
                        }
                    })
                }).collect::<Vec<_>>()
            }
            _ => panic!("Setters only supports named fields"),
        },
        _ => panic!("Setters only supports structs"),
    };
    
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*
        }
    };
    
    TokenStream::from(expanded)
}
```

```rust
// crates/apex-macros/src/to_string.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ToString only supports named fields"),
        },
        _ => panic!("ToString only supports structs"),
    };
    
    let field_strs = fields.iter().filter_map(|f| {
        let name = f.ident.as_ref()?;
        let skip = f.attrs.iter().any(|attr| {
            attr.path().is_ident("to_string") &&
            attr.parse_args::<syn::Ident>()
                .map(|i| i == "skip")
                .unwrap_or(false)
        });
        if skip { return None; }
        
        let name_str = name.to_string();
        Some(quote! {
            .field(#name_str, &self.#name)
        })
    });
    
    let expanded = quote! {
        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#field_strs)*
                    .finish()
            }
        }
    };
    
    TokenStream::from(expanded)
}
```

### 四、使用示例大全

#### 4.1 基础用法：#[derive(Data)]

```rust
// crates/apex-core/src/models/message.rs
use apex_macros::Data;

/// 消息实体 — 一键生成所有方法
#[derive(Debug, Clone, Data)]
struct Message {
    id: String,
    turn_id: String,
    role: MessageRole,
    timestamp: chrono::DateTime<chrono::Local>,
    blocks: Vec<MessageBlock>,
    status: MessageStatus,
}

// 使用
let msg = Message::new(
    "msg-001".into(),
    "T-12".into(),
    MessageRole::User,
    chrono::Local::now(),
    vec![],
    MessageStatus::Complete,
);

assert_eq!(msg.id(), "msg-001");
assert_eq!(msg.to_string(), r#"Message(id="msg-001", turn_id="T-12", ...)"#);

let msg2 = Message::new("msg-002".into(), "T-13".into(), MessageRole::Apex, chrono::Local::now(), vec![], MessageStatus::Sending)
    .with_status(MessageStatus::Complete); // builder 风格
```

#### 4.2 细粒度控制：结构体级别

```rust
// 只生成 getter 和 to_string，不生成 setter（不可变对象）
#[derive(Data)]
#[data(setters = false, builder = false, constructor = false, equals = false)]
struct ImmutableConfig {
    name: String,
    version: String,
}

// 使用
let cfg = ImmutableConfig::new("apex".into(), "1.0.0".into());
// cfg.set_name(...) // ❌ 编译错误：方法不存在
```

#### 4.3 细粒度控制：字段级别

```rust
#[derive(Data)]
struct ProviderConfig {
    name: String,
    display_name: String,
    enabled: bool,

    // API Key 不生成 to_string（避免日志泄露）
    #[data(skip_to_string)]
    api_key: String,

    // created_at 不生成 setter（由系统维护）
    #[data(skip_setter, skip_builder)]
    created_at: chrono::DateTime<chrono::Utc>,

    // 内部字段，完全不暴露
    #[data(skip)]
    internal_id: u64,
}

// 使用
let provider = ProviderConfig::new(
    "anthropic".into(),
    "Anthropic Claude".into(),
    true,
    "sk-xxx".into(),
    chrono::Utc::now(),
    42,
);

println!("{}", provider);  
// 输出：ProviderConfig(name="anthropic", display_name="Anthropic Claude", enabled=true, ...)
// api_key 被 skip_to_string，不会出现在输出中

// provider.set_api_key("new-key".into()); // ✅ 可以修改
// provider.set_created_at(...);           // ❌ 编译错误
// provider.internal_id();                 // ❌ 编译错误
```

#### 4.4 单独使用某个宏

```rust
use apex_macros::{Getters, ToString};

// 只需要 getter 和 to_string
#[derive(Debug, Getters, ToString)]
struct ModelInfo {
    id: String,
    name: String,
    max_tokens: u64,
}

let model = ModelInfo { id: "gpt-4o".into(), name: "GPT-4o".into(), max_tokens: 128000 };
assert_eq!(model.id(), "gpt-4o");
assert_eq!(model.to_string(), r#"ModelInfo(id="gpt-4o", name="GPT-4o", max_tokens=128000)"#);
```

#### 4.5 AI 模型配置实体（完整示例）

```rust
// crates/apex-core/src/config/mod.rs
use apex_macros::Data;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 应用总配置
#[derive(Debug, Clone, Serialize, Deserialize, Data)]
#[data(equals = false)]  // 配置对象通常不需要比较相等性
pub struct AppConfig {
    ui: UiConfig,
    providers: Vec<ProviderConfig>,
    agent: AgentConfig,
}

impl AppConfig {
    pub fn active_provider(&self) -> Option<&ProviderConfig> {
        let active_name = self.agent().active_provider();
        self.providers().iter().find(|p| p.name() == active_name)
    }
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize, Data)]
pub struct UiConfig {
    theme: String,
    font_family: String,
    font_size: u16,
    line_height: f32,
    cursor_style: String,
}

/// AI 服务商配置
#[derive(Debug, Clone, Serialize, Deserialize, Data)]
pub struct ProviderConfig {
    name: String,
    display_name: String,
    enabled: bool,
    base_url: String,

    #[data(skip_to_string)]  // 安全：to_string 不暴露 API Key
    api_key: String,

    default_model: String,
    timeout_seconds: u64,
    max_retries: u32,
    models: Vec<ModelInfo>,
    headers: HashMap<String, String>,
}

impl ProviderConfig {
    /// 获取解析后的 API Key（支持环境变量引用）
    pub fn resolved_api_key(&self) -> Result<String, ConfigError> {
        let key = self.api_key();
        if key.starts_with('$') {
            std::env::var(&key[1..]).map_err(|e| ConfigError::MissingEnvVar(key[1..].to_string()))
        } else {
            Ok(key.clone())
        }
    }
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize, Data)]
pub struct ModelInfo {
    id: String,
    name: String,
    max_tokens: u64,
    context_window: u64,
}

/// Agent 行为配置
#[derive(Debug, Clone, Serialize, Deserialize, Data)]
pub struct AgentConfig {
    active_provider: String,
    mode: String,
    temperature: f32,
    top_p: f32,
}

/// 使用示例
fn demo() {
    let provider = ProviderConfig::new(
        "anthropic".into(),
        "Anthropic Claude".into(),
        true,
        "https://api.anthropic.com/v1".into(),
        "$ANTHROPIC_API_KEY".into(),  // 环境变量引用
        "claude-opus-4-8".into(),
        120,
        3,
        vec![
            ModelInfo::new("claude-opus-4-8".into(), "Claude Opus 4.8".into(), 200000, 200000),
        ],
        HashMap::new(),
    );

    println!("Provider: {}", provider.name());           // "anthropic"
    println!("Display: {}", provider.display_name());    // "Anthropic Claude"
    println!("API Key: {}", provider.api_key());         // "$ANTHROPIC_API_KEY"
    println!("ToString: {}", provider);                   // 不含 api_key 字段

    // Builder 风格修改
    let provider2 = provider
        .with_enabled(false)
        .with_timeout_seconds(60);
}
```

### 五、属性速查表

#### 5.1 结构体属性

| 属性                        | 示例                 | 说明                   |
|-----------------------------|----------------------|------------------------|
| `#[data]`                   | `#[derive(Data)]`    | 全部功能启用（默认）   |
| `#[data(getters = false)]`  | 禁用 getter          | 结构体级别关闭         |
| `#[data(setters = false)]`  | 禁用 setter          | 结构体级别关闭         |
| `#[data(builder = false)]`  | 禁用 builder         | 结构体级别关闭         |
| `#[data(constructor = false)]` | 禁用 new()         | 结构体级别关闭         |
| `#[data(to_string = false)]` | 禁用 Display        | 结构体级别关闭         |
| `#[data(equals = false)]`   | 禁用 PartialEq/Hash  | 结构体级别关闭         |
| `#[data(no_getters)]`       | 快捷禁用             | 等价于 getters = false |

#### 5.2 字段属性

| 属性                     | 示例                        | 说明              |
|--------------------------|-----------------------------|-------------------|
| `#[data(skip)]`          | 跳过该字段的所有方法生成    | 完全隐藏字段      |
| `#[data(skip_getter)]`   | 不生成该字段的 getter       | 只写字段          |
| `#[data(skip_setter)]`   | 不生成该字段的 setter       | 只读/系统维护字段 |
| `#[data(skip_builder)]`  | 不生成该字段的 with_xxx     | 构造时固定值      |
| `#[data(skip_to_string)]` | to_string 中跳过该字段     | 敏感信息保护      |
| `#[data(skip_equals)]`   | equals/hash 中跳过该字段    | 不参与比较        |

### 六、CI 编码规范检查

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used
      
      - name: Check for pub fields in structs
        run: |
          # 查找所有包含 pub 字段的 struct 定义（排除宏生成和测试代码）
          OFFENDERS=$(grep -rn "^\s*pub\s\+\w\+:\s" crates/ --include="*.rs" \
            | grep -v "generated" \
            | grep -v "#\[derive" \
            | grep -v "mod.rs" \
            | grep -v "tests/" || true)
          
          if [ -n "$OFFENDERS" ]; then
            echo "❌ ERROR: Found pub fields in structs. Use #[derive(Data)] instead."
            echo "$OFFENDERS"
            exit 1
          fi
          echo "✅ No pub fields found"
      
      - name: Check Data derive usage
        run: |
          # 检查所有 struct 是否都使用了 Data 或至少一个 derive 宏
          # （允许 enum 和 unit struct 例外）
          MISSING=$(grep -rn "^pub struct\s\+\w\+\s*{" crates/ --include="*.rs" \
            | grep -v "#\[derive" || true)
          
          if [ -n "$MISSING" ]; then
            echo "⚠️ WARNING: Some structs may be missing derive macros:"
            echo "$MISSING"
          fi
      
      - name: Build
        run: cargo build --all-features
      
      - name: Test
        run: cargo test --all-features
```

### 七、更新后的 Workspace 结构

```plain
apex-workspace/
├── Cargo.toml                    # Workspace 定义
├── crates/
│   ├── apex-tui/                 # 主应用（winit + softbuffer + ratatui）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       ├── backend/          # PixelBackend
│   │       ├── ui/               # 界面渲染
│   │       └── platform/         # 平台适配
│   │
│   ├── apex-core/                # 业务逻辑 + 配置
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config/           # 配置加载
│   │       │   ├── mod.rs
│   │       │   ├── loader.rs
│   │       │   └── models.rs     # 配置实体（使用 #[derive(Data)]）
│   │       ├── models/           # 业务实体
│   │       │   ├── message.rs
│   │       │   ├── session.rs
│   │       │   └── project.rs
│   │       ├── agent/            # Agent API 客户端
│   │       └── spec/             # Spec 流水线
│   │
│   └── apex-macros/              # 过程宏（Lombok 风格）
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs            # 宏导出
│           ├── data.rs           # #[derive(Data)] 复合宏
│           ├── getters.rs        # #[derive(Getters)]
│           ├── setters.rs        # #[derive(Setters)]
│           ├── builder.rs        # #[derive(Builder)]
│           ├── constructor.rs    # #[derive(Constructor)]
│           ├── to_string.rs      # #[derive(ToString)]
│           ├── equals.rs         # #[derive(Equals)]
│           └── utils.rs          # 共享工具
│
├── assets/
│   └── default-config.toml       # 默认配置模板
└── .github/
    └── workflows/
        └── ci.yml                # CI 编码规范检查
```

### 八、关键设计决策说明

| 决策                        | 理由                                                                   |
|-----------------------------|------------------------------------------------------------------------|
| 复合宏 Data + 单独宏并存    | Data 覆盖 90% 场景；单独宏用于特殊需求（如只需要 getter 的不可变对象） |
| setter 返回 &mut Self       | 支持链式调用：obj.set_a(1).set_b(2)，符合 Rust builder 惯用法          |
| builder 使用 with_xxx(self, val) | 消费 self 而非 &mut self，支持不可变链式构造，与 set_xxx 区分语义 |
| to_string 使用 DebugStruct 格式 | 自动处理引号、转义、嵌套结构，比手写 Display 更可靠                 |
| skip_to_string 保护敏感字段 | API Key、密码等字段默认不出现在日志中，安全优先                        |
| 字段级 skip 覆盖结构体级配置 | 精细控制：结构体开 setter，但某个字段禁止修改                         |
| CI 拦截 pub 字段            | 强制规范落地，避免代码审查遗漏                                         |

这套宏系统完整覆盖了 Java Lombok @Data 的核心能力，同时针对 Rust 的所有权系统和安全需求做了适配。开发者只需 #[derive(Data)] 一行代码，即可获得完整的封装、构造、访问、比较、格式化能力，且全程零 pub 字段。
