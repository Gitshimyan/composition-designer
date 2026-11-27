# 025 — Extend：Dependency Provider 编辑

- 状态：done
- 目标：补齐 v0.1 冻结命令集缺失的 Dependency Provider 编辑能力，让 Designer 的 Dependency 视图可以设置显式 Provider、回退自动解析，并同步 CLI/MCP 事务语义。
- 授权：**Extend + Modify（用户于 2026-11-27 明确授权补齐该权限缺口）**
- 变更声明：`changes/025-dependency-provider.yaml`
- 依赖：024 的 Dependency 只读投影；021 试点结论

## 背景与授权审计

024 在 Dependency 视图中明确记录：v0.1 的 12 条编辑命令没有“设置 Dependency Provider”，
`dependency_providers` 只能被读取、重命名或删除间接改变。用户已明确授权补齐这类
Extend 能力，因此本卡新增两条命令并打通 Core → TOML → Designer → 测试。

同时审计 021–024 的遗留：

| 缺口 | 类别 | 本卡处理 |
|---|---|---|
| Dependency Provider 无编辑命令（024） | Extend | **本卡补齐** |
| G1 接口/契约独立身份（021） | Rearchitect（Schema/领域模型） | 不在本卡；需要单独设计冻结字段与迁移 |
| G2 runtime/service 容器（021） | Rearchitect | 不在本卡；当前用结构 Slot 近似 |
| G3 项目级 Profile（021） | Rearchitect | 不在本卡 |
| G4–G8 技术元数据、kind、Port wire、Profile-scoped task、模板（021） | Rearchitect / 后续 | 不在本卡 |
| ConfigValue 的整数 float 在 JSON 线上无法区分（022） | 命令/协议格式（Extend 级） | 不在本卡；需要独立 wire 设计，避免局部 hack |
| CI 远端执行（020） | 推送权限/流程 | 用户未要求推送，保持本地 |
| Source 编辑 | M7 计划 | 不属于 M6 |

本卡只做 Provider 命令；不修改设计基线、Schema 字段、`schema_version` 或对象关系。

## 范围与关键决策

1. **新增命令**（`command-schema-v1.md` 同步扩展）：
   - `set_dependency_provider { instance, requirement, provider }`
   - `unset_dependency_provider { instance, requirement }`
2. **命令级错误**：目标实例不存在、Unset 时没有显式映射 → `edit-invalid-command`；
   requirement / provider 是否符合契约、是否自引用仍由候选模型 L2–L7 校验
   （`contract-dependency-unknown`、`dependency-provider-invalid`、`dependency-self` 等）。
3. **TOML 形状**：写回 `[instances.dependencies]` 表；Set 写入
   `requirement = "provider"`，Unset 删除键并在表为空时移除表；保留其余格式。
4. **Designer**：Dependency 视图的每条要求提供 `Set provider…` 与（显式映射时）
   `Clear explicit`；候选仅列出目标模块/版本匹配的实例，最终合法性由 Core 判定。
5. **CLI/MCP**：事务信封原样支持新命令，不需要新的 Adapter 语义；补端到端测试。
6. **审计结论**：G1–G3 等是 Rearchitect，不因本次 Extend 授权自动纳入；本卡结卡时
   在结果中再次列出，供下一阶段决策。

## 清单

- [x] 读 024 / command-schema / editor workspace/session / 领域依赖模型
- [x] 任务卡与 Change Manifest（authorization: extend）
- [x] `composition-editor`：新增命令、kind 列表、事务应用
- [x] `ProjectSession`：`[instances.dependencies]` 保留格式写回
- [x] `command-schema-v1.md`：命令表、错误行为、Extend 说明
- [x] Editor/Session 测试：Set、Unset、非法 provider、未知 requirement、缺映射
- [x] Designer `types.ts` + Dependency Provider 对话框 + DependencyView 操作
- [x] App 接线、Review、选择保持
- [x] Fixtures/mock 与 UI 用例（设置/清空/歧义状态）
- [x] Tauri host：Provider Set/Unset 写回与非法拒绝
- [x] 更新 plan（025 插入，Change Review 顺延 026）/ README / architecture-entry / AGENTS
- [x] ADR-0003：记录 Extend 决策、范围与不修改领域模型/TOML Schema
- [x] 回写本卡与 024 的缺口记录

## 验收

本地已执行（2026-11-27）：

- `cargo test --workspace`：**154 passed**（原 146 + 命令单测 1 + Editor 4 + Session 2 + Protocol 1）；
- `cargo test --manifest-path apps/designer/src-tauri/Cargo.toml`：**15 passed**
  （新增 Provider Set/Clear 写回与非法拒绝 1 条真实 `ProjectSession` 测试）；
- `cd apps/designer && npm run build`：通过（tsc --noEmit + Vite，90 modules）；
- `cd apps/designer && npm run test:ui`：**24 passed**（新增 Provider 设置/清空 1 条）；
- `cargo clippy --workspace --all-targets -- -D warnings`：通过；
- `cargo fmt --all --check`：通过。

逐项核对：

- `COMMAND_KINDS` 为 14；新命令可序列化/反序列化，事务应用与幂等/Revision 规则复用；
- 合法 Set/Unset 写回 `[instances.dependencies]`、空表移除、注释保留；
  `validate` 通过、Revision/`source_digest` 刷新；
- 非法 provider 目标、自引用、未知 requirement、缺映射分别被 Core/命令错误拒绝，
  文件与工作区不变；
- CLI/MCP/Protocol 共用 `Transaction`，Protocol 新增 Provider 事务测试通过；
- Designer 设置/清空显式 Provider 经 prepare → Review → apply，取消不写入；
  清空后自动解析重新计算（fixture 中两个候选 → `ambiguous`）；
- 除 `docs/spec/command-schema-v1.md` 的命令扩展外，未改设计基线、TOML 字段或领域模型；
- 变更范围与 `changes/025-dependency-provider.yaml` 一致。

## 结果 / 遗留问题

- 结果：
  - `crates/composition-editor/src/command.rs`：2 条新命令与 `COMMAND_KINDS` 14；
  - `workspace.rs`：Set/Unset 显式映射（缺实例/缺映射为命令错误）；
  - `session.rs`：`[instances.dependencies]` 写回/移除/保留格式；
  - `docs/spec/command-schema-v1.md`：命令表与错误行为（标注 v0.2 025 Extend）；
  - `docs/adr/0003-dependency-provider-command.md`：Extend 决策与影响记录；
  - `tests/editor.rs`、`tests/session.rs`、Protocol 测试；
  - `src/relations/DependencyProviderDialog.tsx`、DependencyView 操作按钮；
  - `relations.spec.ts` 新增 Provider 设置/清空链路。
- 关键决策：
  - Provider 合法性不在命令层复制，交给候选模型 L2–L7；
  - Unset 是显式映射的精确回退，不删除实例；
  - 候选列表只列目标模块/版本匹配实例，避免 UI 诱导非法映射。
- 权限审计结论：
  - 本次补齐的是 024 唯一明确标注的 **Extend** 缺口；
  - 2026-11-27 授权审计指出本卡 Manifest 漏列 `crates/composition-protocol` 测试 target，
    已补 `crates/composition-protocol` 与 `protocol_transaction_test`（事实性声明修正，不改产品代码）；
  - G1 接口/契约身份、G2 runtime 容器、G3 项目级 Profile、G4–G8 技术元数据/任务模板
    属于 **Rearchitect**（Schema/领域模型/设计基线），未在本次 Extend 授权下修改；
  - ConfigValue 整数值 float 的 JSON wire 区分是协议格式问题，需要单独设计；
  - CI 远端执行仍等待明确推送指令；Source 编辑属于 M7。
- 下一步：026 — 完整 Change Review。
