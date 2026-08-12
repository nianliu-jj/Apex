# Apex 核心 Trait 接口契约

## 1. 契约约定

本文件是应用 Port 的权威设计。代码块表达首版稳定边界，省略 import、生命周期和序列化细节；实现阶段可以在不改变语义的前提下选择 `async_trait`、关联 Future 或生成式 facade。

统一约定：

```rust
type ApexResult<T> = Result<T, ApexError>;

struct CommandContext {
    actor: ActorRef,
    trace_id: TraceId,
    span_id: SpanId,
    idempotency_key: IdempotencyKey,
    client_id: ClientId,
    protocol_version: ProtocolVersion,
}

enum Durability { Normal, Critical }
enum Consistency { Eventual, ReadYourWrites, StrongLocal }
```

- 所有改变状态的方法必须接收 `CommandContext` 或从父调用显式派生，禁止在底层临时生成无关联 trace。
- 命令必须幂等；重复 `idempotency_key` 返回原结果或稳定冲突，不重复副作用。
- `Critical` 对应 SQLite 临时 `synchronous=FULL` 和必要的文件 `fsync`/目录同步。
- Trait 返回领域类型或 Port DTO，不返回 SQLx、Tonic、Actix、SDK 或平台句柄。
- 所有 Stream 都必须定义取消、背压、重连和末端错误语义。

## 2. 基础 Port

```rust
trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}

trait IdGenerator: Send + Sync {
    fn uuid_v7(&self) -> Uuid;
    fn trace_id(&self) -> TraceId;
    fn span_id(&self) -> SpanId;
}

trait SecretResolver: Send + Sync {
    async fn resolve_provider_key(
        &self,
        profile: ProviderProfileId,
    ) -> ApexResult<SecretString>;
}

trait UnitOfWork: Send {
    async fn commit(self: Box<Self>, durability: Durability) -> ApexResult<()>;
    async fn rollback(self: Box<Self>) -> ApexResult<()>;
}
```

`SecretResolver` 的结果只能传给 Provider/MCP credential adapter，类型不得实现 `Serialize`、`Display` 或 `Debug` 明文输出。

## 3. 事件、投影与查询

```rust
trait EventStore: Send + Sync {
    async fn append(
        &self,
        ctx: &CommandContext,
        aggregate: AggregateRef,
        expected_version: u64,
        events: Vec<NewEvent>,
        durability: Durability,
    ) -> ApexResult<AppendReceipt>;

    async fn load_aggregate(
        &self,
        aggregate: AggregateRef,
        after_version: u64,
    ) -> ApexResult<Vec<EventEnvelope>>;

    async fn load_session_events(
        &self,
        session_id: SessionId,
        after_seq: u64,
        limit: usize,
    ) -> ApexResult<EventPage>;

    fn subscribe_session(
        &self,
        session_id: SessionId,
        after_seq: u64,
    ) -> BoxStream<'static, ApexResult<EventEnvelope>>;
}

trait ProjectionStore: Send + Sync {
    async fn session_snapshot(
        &self,
        session_id: SessionId,
        consistency: Consistency,
    ) -> ApexResult<SessionSnapshot>;

    async fn list_sessions(&self, query: SessionQuery) -> ApexResult<SessionPage>;
    async fn apply_batch(&self, batch: ProjectionBatch) -> ApexResult<()>;
    async fn cursor(&self, projector: ProjectorId) -> ApexResult<ProjectionCursor>;
}

trait Projector: Send + Sync {
    fn id(&self) -> ProjectorId;
    fn supports(&self, event_type: &str, schema_version: u16) -> bool;
    async fn reduce(&self, event: &EventEnvelope, tx: &mut dyn ProjectionTx)
        -> ApexResult<()>;
}
```

`append` 必须在一个 SQLite 事务中完成事件追加、聚合版本、session sequence 和 outbox/必要同步投影。未知事件由不支持它的 Projector 跳过并保留，不能当作损坏删除。

## 4. 文件事实与内容寻址存储

```rust
trait FileFactStore: Send + Sync {
    async fn read(&self, key: FactKey) -> ApexResult<Option<FactDocument>>;
    async fn atomic_write(
        &self,
        ctx: &CommandContext,
        write: FactWrite,
        expected: ExpectedGeneration,
        durability: Durability,
    ) -> ApexResult<FactCommit>;
    async fn reload(&self, key: FactKey) -> ApexResult<ReconcileOutcome>;
    fn watch(&self, scope: FactScope) -> BoxStream<'static, FactChange>;
}

trait MarkdownReconciler: Send + Sync {
    async fn reconcile(
        &self,
        base: FactDocument,
        local: FactDocument,
        external: FactDocument,
    ) -> ApexResult<MergeOutcome>;
}

trait ContentStore: Send + Sync {
    async fn put(&self, kind: ContentKind, bytes: ByteStream) -> ApexResult<ContentRef>;
    async fn open(&self, reference: &ContentRef) -> ApexResult<ByteStream>;
    async fn verify(&self, reference: &ContentRef) -> ApexResult<VerificationStatus>;
    async fn mark_and_sweep(&self, roots: Vec<ContentRef>) -> ApexResult<GcReport>;
}
```

`atomic_write` 使用同目录临时文件、权限继承、flush、原子 rename 和必要的目录 sync；若平台无法保证原子替换，必须返回能力错误并使用恢复 journal，不能假装成功。

## 5. Session、Admission 与租约

```rust
trait SessionService: Send + Sync {
    async fn create_session(
        &self,
        ctx: CommandContext,
        request: CreateSession,
    ) -> ApexResult<SessionSnapshot>;

    async fn submit_prompt(
        &self,
        ctx: CommandContext,
        request: SubmitPrompt,
    ) -> ApexResult<AdmissionReceipt>;

    async fn request_pause(
        &self,
        ctx: CommandContext,
        session_id: SessionId,
        reason: PauseReason,
    ) -> ApexResult<PauseReceipt>;

    async fn resume(&self, ctx: CommandContext, request: ResumeSession)
        -> ApexResult<RunSnapshot>;

    async fn cancel_run(&self, ctx: CommandContext, run_id: RunId)
        -> ApexResult<CancelReceipt>;
}

trait SessionRuntime: Send + Sync {
    async fn wake(&self, session_id: SessionId) -> ApexResult<()>;
    async fn recover_all(&self) -> ApexResult<RecoveryReport>;
    async fn reach_safe_point(
        &self,
        session_id: SessionId,
        deadline: Instant,
    ) -> ApexResult<SafePointReceipt>;
}

trait ControlLeaseService: Send + Sync {
    async fn acquire(&self, ctx: CommandContext, request: AcquireControl)
        -> ApexResult<ControlLease>;
    async fn renew(&self, ctx: CommandContext, lease: ControlLeaseToken)
        -> ApexResult<ControlLease>;
    async fn force_takeover(&self, ctx: CommandContext, request: ForceTakeover)
        -> ApexResult<ControlLease>;
    async fn release(&self, ctx: CommandContext, lease: ControlLeaseToken)
        -> ApexResult<()>;
}

trait WebEnableLeaseService: Send + Sync {
    async fn acquire_from_tui(&self, ctx: CommandContext, ttl: Duration)
        -> ApexResult<WebEnableLease>;
    async fn renew_from_tui(&self, ctx: CommandContext, token: WebLeaseToken)
        -> ApexResult<WebEnableLease>;
    async fn current_listener(&self) -> ApexResult<Option<WebListenerInfo>>;
}
```

Prompt Admission 只表示已持久化，不表示已开始执行。控制租约宽限固定 30 秒；force takeover 必须产生 Durable Event。Web lease 只接受通过 TUI 客户端身份握手的调用。

## 6. Spec、审批、Rules 与验证

```rust
trait SpecPipeline: Send + Sync {
    async fn status(&self, scope: SpecScope) -> ApexResult<SpecPipelineSnapshot>;
    async fn evaluate_gate(
        &self,
        request: GateRequest,
    ) -> ApexResult<SpecGateDecision>;
    async fn approve(
        &self,
        ctx: CommandContext,
        request: ApproveSpec,
    ) -> ApexResult<ApprovalRecord>;
    async fn invalidate_from_change(
        &self,
        ctx: CommandContext,
        change: SpecChange,
    ) -> ApexResult<InvalidationPlan>;
    async fn grant_skip(
        &self,
        ctx: CommandContext,
        request: GrantSkip,
    ) -> ApexResult<SkipGrant>;
    async fn accept_verification(
        &self,
        ctx: CommandContext,
        request: AcceptVerification,
    ) -> ApexResult<CompletionDecision>;
}

trait RuleEngine: Send + Sync {
    async fn lightweight_check(&self, request: RuleCheckRequest)
        -> ApexResult<RuleReport>;
    async fn incremental_check(&self, request: RuleBatchRequest)
        -> ApexResult<RuleReport>;
    async fn completion_check(&self, request: CompletionCheckRequest)
        -> ApexResult<VerificationEvidence>;
    async fn plan_repair(&self, report: RuleReport, budget: RepairBudget)
        -> ApexResult<RepairPlan>;
}

trait VerificationWriter: Send + Sync {
    async fn render_and_commit(
        &self,
        ctx: CommandContext,
        evidence: VerificationEvidence,
        expected: ExpectedGeneration,
    ) -> ApexResult<FactCommit>;
}
```

`evaluate_gate` 是纯状态/策略判断，不执行 Tool。`plan_repair` 返回的路径集合必须是原任务 `write_paths` 的子集；调用方再次经过 Permission 与 Claim。

## 7. 命令分析与权限

```rust
trait CommandAnalyzer: Send + Sync {
    fn parse(&self, dialect: ShellDialect, source: &str) -> ParseOutcome<CommandAst>;
    fn analyze(&self, ast: &CommandAst, environment: &AnalysisEnvironment)
        -> AnalysisOutcome<CommandSemantics>;
}

trait PermissionEngine: Send + Sync {
    async fn decide(&self, request: PermissionEvaluation)
        -> ApexResult<PermissionVerdict>;
    async fn record_grant(&self, ctx: CommandContext, grant: PermissionGrant)
        -> ApexResult<PermissionGrant>;
    async fn resolve_request(
        &self,
        ctx: CommandContext,
        request_id: PermissionRequestId,
        resolution: PermissionResolution,
    ) -> ApexResult<PermissionVerdict>;
    async fn revoke_project_grants(&self, ctx: CommandContext, project: ProjectId)
        -> ApexResult<usize>;
}
```

`PermissionVerdict` 必须包含最终决策、命中的不可放宽规则、规范资源 key、分析证据和可选询问模型。实现 crate 的依赖图中不得出现 Provider crate；CI 以依赖检查守护零 Token 不变量。

## 8. Tool 与终端

```rust
trait Tool: Send + Sync {
    fn descriptor(&self) -> ToolDescriptor;
    async fn prepare(&self, input: RawJson, ctx: &ToolContext)
        -> ApexResult<PreparedToolCall>;
    async fn execute(&self, prepared: PreparedToolCall, io: ToolIo)
        -> ApexResult<ToolExecution>;
}

trait ToolGateway: Send + Sync {
    async fn invoke(
        &self,
        ctx: CommandContext,
        request: ToolInvocation,
    ) -> ApexResult<ToolOutcome>;
    async fn resume_after_permission(
        &self,
        ctx: CommandContext,
        request_id: PermissionRequestId,
    ) -> ApexResult<ToolOutcome>;
    async fn recover_interrupted(&self) -> ApexResult<ToolRecoveryReport>;
}

trait TerminalManager: Send + Sync {
    async fn open_persistent(
        &self,
        ctx: CommandContext,
        spec: TerminalSpec,
    ) -> ApexResult<TerminalHandle>;
    async fn run_once(
        &self,
        ctx: CommandContext,
        spec: CommandSpec,
    ) -> ApexResult<CommandHandle>;
    async fn write(&self, terminal: TerminalId, bytes: Bytes) -> ApexResult<()>;
    fn subscribe(&self, terminal: TerminalId, after_seq: u64)
        -> BoxStream<'static, ApexResult<TerminalFrame>>;
    async fn terminate_tree(&self, ctx: CommandContext, target: ProcessTarget)
        -> ApexResult<TerminationReport>;
}
```

Tool 生命周期固定为：prepare → spec gate → permission → claim → pre-write checkpoint/snapshot → execute → lightweight PostToolUse → durable result。任一阶段失败都不能跳到后续阶段。

## 9. Agent、DAG 与 Claim

```rust
trait AgentRuntime: Send + Sync {
    async fn start(&self, ctx: CommandContext, request: StartAgent)
        -> ApexResult<AgentExecutionSnapshot>;
    async fn steer(&self, ctx: CommandContext, request: SteerAgent)
        -> ApexResult<AdmissionReceipt>;
    async fn recover(&self, execution: AgentExecutionId)
        -> ApexResult<RecoveryDecision>;
}

trait DagScheduler: Send + Sync {
    async fn compile(&self, source: WorkflowSource) -> ApexResult<VersionedDagIr>;
    async fn start(&self, ctx: CommandContext, request: StartDag)
        -> ApexResult<DagRunSnapshot>;
    async fn tick(&self, dag_run: DagRunId) -> ApexResult<SchedulingBatch>;
    async fn pause(&self, ctx: CommandContext, dag_run: DagRunId)
        -> ApexResult<PauseReceipt>;
    async fn resume(&self, ctx: CommandContext, dag_run: DagRunId)
        -> ApexResult<DagRunSnapshot>;
}

trait WriteClaimService: Send + Sync {
    async fn normalize(&self, scope: RawPathScope) -> ApexResult<CanonicalPathScope>;
    async fn acquire(&self, ctx: CommandContext, request: ClaimRequest)
        -> ApexResult<ClaimOutcome>;
    async fn renew(&self, ctx: CommandContext, lease: ClaimLeaseToken)
        -> ApexResult<ClaimLease>;
    async fn release(&self, ctx: CommandContext, lease: ClaimLeaseToken)
        -> ApexResult<()>;
}

trait AgentMailbox: Send + Sync {
    async fn send(&self, ctx: CommandContext, edge: CommunicationEdge, msg: AgentMailboxMessage)
        -> ApexResult<MailboxReceipt>;
    async fn receive(&self, node: NodeRunId, after_seq: u64)
        -> ApexResult<Vec<AgentMailboxMessage>>;
}
```

只有 `VersionedDagIr` 内显式声明的通信边能调用持久邮箱。父/子默认通过结构化完成结果汇聚，不获得任意互发通道。

## 10. Snapshot、Checkpoint、Context、Memory 与重放

```rust
trait SnapshotStore: Send + Sync {
    async fn capture(&self, ctx: CommandContext, request: SnapshotRequest)
        -> ApexResult<SnapshotManifest>;
    async fn diff(&self, from: SnapshotId, to: WorkspaceState)
        -> ApexResult<SnapshotDiff>;
    async fn restore(&self, ctx: CommandContext, request: RestoreRequest)
        -> ApexResult<RestoreReport>;
}

trait CheckpointStore: Send + Sync {
    async fn commit(&self, ctx: CommandContext, checkpoint: CheckpointDraft)
        -> ApexResult<CheckpointManifest>;
    async fn latest(&self, session: SessionId) -> ApexResult<Option<CheckpointManifest>>;
    async fn reconstruct(&self, checkpoint: CheckpointId)
        -> ApexResult<ReconstructedSession>;
    async fn pin(&self, ctx: CommandContext, checkpoint: CheckpointId, pinned: bool)
        -> ApexResult<()>;
}

trait ContextManager: Send + Sync {
    async fn build_epoch(&self, request: BuildContext) -> ApexResult<ContextEpoch>;
    async fn observe_usage(&self, observation: ContextUsage)
        -> ApexResult<Vec<ContextAction>>;
    async fn apply_action(&self, ctx: CommandContext, action: ContextAction)
        -> ApexResult<ContextEpoch>;
}

trait MemoryStore: Send + Sync {
    async fn propose_write(&self, request: MemoryWriteProposal)
        -> ApexResult<MemoryWriteDecision>;
    async fn commit(&self, ctx: CommandContext, request: CommitMemory)
        -> ApexResult<MemoryDocument>;
    async fn search(&self, query: MemoryQuery) -> ApexResult<Vec<MemoryHit>>;
    async fn record_recall(&self, ctx: CommandContext, recall: MemoryRecall)
        -> ApexResult<()>;
    async fn delete(&self, ctx: CommandContext, id: MemoryId) -> ApexResult<()>;
    async fn export(&self, request: ExportMemory) -> ApexResult<ExportArtifact>;
}

trait ReplayCoordinator: Send + Sync {
    async fn plan_state_replay(&self, request: StateReplayRequest)
        -> ApexResult<StateReplayPlan>;
    async fn execute_state_replay(&self, ctx: CommandContext, plan: StateReplayPlan)
        -> ApexResult<ReplayReport>;
    async fn plan_reexecution(&self, request: ReexecutionRequest)
        -> ApexResult<ReexecutionPlan>;
    async fn execute_reexecution(&self, ctx: CommandContext, approved: ApprovedReexecution)
        -> ApexResult<RunSnapshot>;
    async fn compensate(&self, ctx: CommandContext, request: CompensationRequest)
        -> ApexResult<CompensationReport>;
}
```

`reconstruct` 必须验证所有内容哈希和附件；缺少任一必需块时返回损坏错误，不生成“尽可能恢复”的伪完整 Checkpoint。

## 11. Provider 与多模态

```rust
trait Provider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;
    async fn resolve_capabilities(&self, model: &ModelRef)
        -> ApexResult<ModelCapabilities>;
    fn stream(
        &self,
        request: ModelRequest,
    ) -> BoxStream<'static, ApexResult<ProviderFrame>>;
    async fn realtime(&self, request: RealtimeRequest)
        -> ApexResult<Box<dyn RealtimeSession>>;
}

trait ProviderRegistry: Send + Sync {
    async fn resolve(&self, profile: ProviderProfileId)
        -> ApexResult<Arc<dyn Provider>>;
    async fn route(&self, request: RouteRequest) -> ApexResult<RoutePlan>;
    async fn health(&self, provider: ProviderProfileId) -> ApexResult<ProviderHealth>;
}

trait AttachmentService: Send + Sync {
    async fn import(&self, ctx: CommandContext, source: AttachmentSource)
        -> ApexResult<ArtifactRef>;
    async fn adapt(&self, artifact: ArtifactRef, target: ModelCapabilities)
        -> ApexResult<AdaptedAttachment>;
}
```

`Provider::stream` 的取消必须传播到厂商请求；usage、reasoning、Tool delta、audio 和 error 都使用有类型 Frame。`realtime` 在不支持时返回 capability error，禁止退化为看似实时的轮询而不告知用户。

## 12. Skills、MCP 与 Plugin

```rust
trait SkillRegistry: Send + Sync {
    async fn scan(&self, roots: Vec<SkillSource>) -> ApexResult<SkillScanReport>;
    async fn resolve(&self, request: SkillResolution) -> ApexResult<ResolvedSkill>;
    async fn trust(&self, ctx: CommandContext, request: TrustSkill)
        -> ApexResult<SkillTrustRecord>;
}

trait McpManager: Send + Sync {
    async fn discover(&self) -> ApexResult<McpDiscoveryReport>;
    async fn set_enabled(&self, ctx: CommandContext, request: SetMcpEnabled)
        -> ApexResult<McpServerSnapshot>;
    async fn start(&self, ctx: CommandContext, server: McpServerId)
        -> ApexResult<McpConnection>;
    async fn stop(&self, ctx: CommandContext, server: McpServerId)
        -> ApexResult<()>;
    async fn sync_to_source(&self, ctx: CommandContext, request: SyncMcpSource)
        -> ApexResult<SourceWriteReceipt>;
}

trait PluginManager: Send + Sync {
    async fn discover(&self) -> ApexResult<PluginDiscoveryReport>;
    async fn verify(&self, plugin: PluginId) -> ApexResult<PluginVerification>;
    async fn activate(&self, ctx: CommandContext, plugin: PluginId)
        -> ApexResult<PluginSession>;
    async fn deactivate(&self, ctx: CommandContext, plugin: PluginId)
        -> ApexResult<()>;
}
```

`McpManager::discover` 无副作用；只有 `start` 才创建连接/进程。`PluginManager::activate` 根据签名自动选择 in-process 或 Host，调用者不能要求第三方绕过隔离。

## 13. 日志、归档与诊断

```rust
trait SessionLogSink: Send + Sync {
    async fn append(&self, record: SessionLogRecord) -> ApexResult<LogPosition>;
    async fn seal_segment(&self, session: SessionId) -> ApexResult<SegmentSignature>;
}

trait SystemLogSink: Send + Sync {
    fn write(&self, level: LogLevel, target: &str, message: SanitizedText);
}

trait LogQueryService: Send + Sync {
    async fn list_session_segments(&self, session: SessionId)
        -> ApexResult<Vec<LogSegmentMeta>>;
    async fn read_session_logs(&self, query: SessionLogQuery)
        -> ApexResult<SessionLogPage>;
    async fn verify_session_logs(&self, session: SessionId)
        -> ApexResult<LogVerificationReport>;
    async fn read_system_logs(&self, query: SystemLogQuery)
        -> ApexResult<SystemLogPage>;
}

trait ArchiveStore: Send + Sync {
    async fn archive_session(&self, ctx: CommandContext, session: SessionId)
        -> ApexResult<ArchiveManifest>;
    async fn mount_read_only(&self, archive: ArchiveId)
        -> ApexResult<ArchiveMount>;
    async fn restore_session(&self, ctx: CommandContext, archive: ArchiveId)
        -> ApexResult<SessionSnapshot>;
    async fn purge_expired(&self, now: OffsetDateTime) -> ApexResult<PurgeReport>;
}
```

TUI 的服务能力表中不暴露 `LogQueryService`；Desktop/Web 可查看会话日志与系统日志，但仍需经过访问控制与脱敏。归档包不包含已经过期的会话日志。

## 14. 关键组合事务

| 用例 | 必须的顺序 |
|---|---|
| Prompt Admission | 校验控制租约 → 幂等检查 → 持久化 inbox/event → 确认客户端 → 唤醒 Actor |
| Spec 审批 | 读取当前内容哈希 → Critical 事务写审批/event → 更新投影 → 确认 |
| 高风险 Tool 写 | Spec Gate → Permission → Claim → Checkpoint → Snapshot → Tool → PostToolUse → event/log |
| Checkpoint | 写内容块 → 写 Manifest 临时文件 → 原子替换/fsync → Critical 索引/event → 完成 |
| 外部 Markdown 变化 | 捕获 external generation → 三方合并 → 文件 commit → 投影/event；失败则 Blocked |
| Web 开启 | 验证 TUI 身份 → 持久 lease → 绑定随机 localhost 端口 → 生成一次性 token |

## 15. 契约测试要求

- 每个 Adapter 必须通过同一 Port contract suite；内存 fake 不能替代 SQLite/真实文件系统的故障测试。
- gRPC 与 REST 对同一 Command 的领域结果、错误码、幂等语义必须一致。
- 三个 Shell Analyzer 使用 golden AST/语义 fixture、模糊测试和已知注入语料。
- 每个 Provider Adapter 对统一消息、Tool、流取消、错误和 capability 降级做录制回放测试。
- FileFact/Snapshot 在 macOS、Windows、Linux 上测试 symlink、大小写、长路径、权限位和崩溃注入。
