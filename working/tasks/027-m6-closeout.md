# 027 — M6 收口验收

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-029
- 目标：按 M6 完成标准做一次可重复的真实项目全链路验收，并核实 CI PR/合并门禁；只补充验证与记录，不实现新能力。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：`working/plan.md` §6 M6 总验收链路；020 / 021 / 022–026 结果
- 变更声明：`changes/027-m6-closeout.yaml`

## 背景

020–026 已让 Instance/Binding/Connection/Dependency 编辑与完整 Change Review 全部落地，
但 M6 总验收还有两项未证明：

1. **真实项目全链路**：现有 UI 自动化使用 IPC fixture，host 编辑测试使用示例项目；
   Glide 试点（021）只做过只读 validate。尚未在 Glide 试点副本上完成
   “创建 → Binding → Config → Connection / Provider → Review → 写回 → 重开”的可重复链路。
2. **CI PR 门禁**：workflow 含 `pull_request` 触发，但尚未观察到 PR 运行，
   也未核实分支保护/失败合并阻止设置。

本卡只做验收、集成测试与记录，不新增 Core 命令、Schema 字段或产品能力。

## 范围与关键决策

1. 复制 `working/pilots/glide-task-terminal` 到临时目录，用真实 `ProjectSession`
   做一条 M6 编辑链：创建实例、Binding、配置、Connection、Dependency Provider，validate，
   重新打开并比对。
2. 用 CLI `validate` / `query` 或 Protocol/MCP 事务验证同一结果的入口一致性；
   至少覆盖 Core 事务 + 一个 Agent/CLI 入口。
3. 检查 GitHub Actions `pull_request` 运行与 main 分支保护；若仓库设置不可读/不可改，
   记录为外部阻塞，不伪称已强制。
4. 将结果写入 `docs/reviews/2026-11-27-m6-closeout.md`；更新 plan 的 M6 状态与验收结论。
5. 不把“本地测试通过”等同为“真实项目全链路已验收”。

## 清单

- [x] 建立 Glide 试点副本的 M6 全链路集成测试
- [x] 验证写入后可重开、validate 通过、Revision/Digest 刷新
- [x] 验证 CLI/Protocol 至少一个入口与 Core 结果一致
- [x] 检查 CI PR 触发与分支保护设置，记录外部阻塞
- [x] 写 `docs/reviews/2026-11-27-m6-closeout.md`
- [x] 更新 plan / README / architecture-entry 的 M6 状态
- [x] 创建 `changes/027-m6-closeout.yaml`

## 验收

- 在 Glide 试点副本上完成一次可重复的 M6 编辑链，测试可单独重跑；
- 写回后重开项目仍 valid，Revision/Digest 与 CLI 输出一致；
- PR 门禁有明确结论：已启用并验证，或记录为外部阻塞（不伪装通过）；
- M6 状态从“功能卡完成”更新为“总验收通过”或“仍有明确未决项”；
- 不新增产品命令/字段，不修改设计基线。

## 结果 / 遗留问题

- 结果：
  - 新增 `crates/composition-cli/tests/m6_closeout.rs`：在 Glide 试点副本上通过 CLI/Protocol
    完成创建 Instance、Binding、配置、Connection、Dependency Provider、写回、重开与 validate；
  - 验证实例 9、Binding 8、Connection 7，Digest 与 Core 一致；
  - UI 28 条、Tauri host 15 条、workspace 155 条测试通过；
  - 新增 `docs/reviews/2026-11-27-m6-closeout.md` 记录验收结论。
- 未完成 / 外部阻塞：
  - `main` 未启用分支保护（API 404），无 PR 运行记录；
  - 无法证明失败合并被阻止；UW-029 保持 pending_decision，需仓库管理员设置
    required check `Windows build and UI tests` 并用 PR 验证。
- 验证边界：
  - UI 自动化使用 IPC fixture；未做同一 Glide 副本上的 GUI + MCP 并发验证；
  - 跨入口一致性由共享 Core 与分别的 UI/CLI/Protocol 测试支持。
- 下一步：处理 UW-029；M6 不标记为完全通过。G04 机器门禁仍需 Extend + Rearchitect。
