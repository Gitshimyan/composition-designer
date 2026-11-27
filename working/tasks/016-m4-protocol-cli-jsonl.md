# 016 — M4 领域协议与 CLI/JSONL

- 状态：done
- 目标：建立机器可读 Query/Command 协议，提供 CLI `query` / `apply` / `jsonl`；复用同一
  Core/ProjectSession，不引入第二套模型。
- 授权：Extend（新 protocol crate、CLI 命令）
- 变更声明：`changes/016-protocol-cli-jsonl.yaml`

> 授权审计（2026-11-27）：审计建议 **Extend + Rearchitect**（新 Protocol crate + Agent/CLI 通信契约）；原始授权字段保持不变。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

- coverage_status：complete_with_deferred_items
- open_items：UW-011、UW-015、UW-018
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

M3 的编辑能力只在 Tauri/Rust API 中。M4 要让 Agent 用统一协议执行同样操作：

- 查询：inspect / resolve / validate / source；
- 命令：prepare_transaction / apply_transaction；
- 单次 CLI 与长连接 JSONL；
- 所有写操作带 Revision，外部 digest 变化拒绝覆盖。

## 清单

- [x] 新 crate `composition-protocol`
- [x] Request/Response/ProtocolError
- [x] 查询：inspect / resolve / validate / source
- [x] 命令：prepare_transaction / apply_transaction
- [x] 外部 digest 冲突守卫
- [x] CLI `query` / `apply`（`--dry-run`）/ `jsonl`
- [x] JSONL 逐行请求/响应、空行忽略、非法行不退出
- [x] 协议文档 `docs/spec/protocol-v1.md`
- [x] 测试：协议查询/无效项目/未知 op/apply+外部冲突；CLI JSONL/dry-run
- [x] `cargo test` / clippy / fmt；更新 plan、README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**137 passed**
  （CLI 10 + core 8 + domain 58 + editor 5/18/7 + protocol 4 + toml 12/15）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`；
- `composition-cli query examples/minimal-project validate` 返回 `valid=true`；
- `composition-cli query examples/invalid-projects/files/entry-missing validate` 返回
  `ok=true`、`valid=false`、`file-project-missing` 诊断；
- `composition-cli apply ... --dry-run` 不写文件；
- `jsonl` 两个请求返回两行响应，id/ok 正确；
- 外部修改后再 apply 返回 `edit-revision-conflict`，不覆盖磁盘；
- 协议不返回 GUI 布局，不写 `.designer/**`。

## 结果 / 遗留问题

- 结果：
  - `composition-protocol`：Request/Response、4 查询 + 2 命令、外部冲突守卫；
  - CLI `query` / `apply` / `jsonl`；
  - protocol 4 测试；CLI 10 测试；
  - 协议文档冻结。
- 关键决策：
  - 查询无效项目仍 `ok=true`，valid 在 result；
  - 命令拒绝 `ok=false`，diagnostics 带真实 Core 代码；
  - JSONL 长连接保持 ProjectSession；
  - 协议层也做磁盘 digest 守卫，避免 CLI 覆盖外部修改；
  - 既有 inspect/resolve/validate 格式不破坏。
- 遗留：
  - JSONL 不含 request 幂等持久化；
  - `source` 只读，无 Source 编辑事务；
  - 仅支持单个 composition 入口；
  - 无 MCP Adapter（017）；
  - CLI apply 只接受 transaction 文件，不支持 stdin 批处理。
- 下一步：017 MCP Adapter。
