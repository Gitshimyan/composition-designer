# G02 — 历史授权审计记录与 Manifest 修正

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：把 001–025 的授权审计结论固化为可追溯记录，并修正 Manifest 的事实性覆盖遗漏；不伪造历史授权。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：`docs/reviews/2026-11-27-authorization-audit.md`
- 变更声明：`changes/G02-historical-audit.yaml`（已创建）

## 范围

1. 新建 `docs/reviews/2026-11-27-authorization-audit-record.md`：
   - 每张 001–025 卡的原始授权、审计建议、证据状态、Manifest 差异；
   - 明确标注“审计建议”与“当时授权证据不足”，不修改旧卡授权字段；
   - 对 002/003/009/010/016 的升级建议、005/012/020 的条件性判断、
     004 的 Compose 补充、021 的 `working/pilots/**` 路径问题逐项记录。
2. 修正已确认的事实性 Manifest 问题：
   - 025：补 `crates/composition-protocol` target 与 `protocol_transaction_test`（已在本轮完成）；
   - 020：`expected_command_changes` 改为 `false` 并说明生产 IPC 未变（已在本轮完成）；
   - 010：记录 `authorization: modify` 与 `expected_command_changes: true` 的不一致，
     不追溯修改授权字段；
   - 021：记录 Compose 路径缺口，路径规则由 G01 修订。
3. 在相关任务卡追加一行审计备注（链接到审计记录），不改原始范围与验收。

## 清单

- [x] 001–025 历史授权审计记录
- [x] 025/020 Manifest 事实性修正（本轮前完成）
- [x] 010/021 Manifest 差异记录（不改授权字段）
- [x] 13 张相关任务卡追加审计备注
- [x] `docs/reviews/README.md` 链接审计记录
- [x] 创建 `changes/G02-historical-audit.yaml`

## 非目标

- 不为 002–004 伪造不存在的 Manifest；
- 不把审计建议写成“当时已获授权”；
- 不修改产品代码、Schema 或命令协议；
- 不修改 G01 已生效的规则。

## 验收

- 001–025 每张卡在审计记录中都有分类与证据状态；
- 025/020 的 Manifest 事实性遗漏已修正，且与实际 diff 一致；
- 010/021 的授权/路径问题已记录为审计建议，不追溯改写历史授权；
- 未确认的历史授权保持“审计建议/证据不足”，没有倒填；
- `docs/reviews/README.md` 与新审计记录互相链接；
- 不改动任何产品实现文件。

## 结果 / 遗留问题

- 结果：
  - 新建 `docs/reviews/2026-11-27-authorization-audit-record.md`，覆盖 001–025；
  - 002/003/004/005/009/010/012/014/015/016/020/021/025 任务卡追加审计备注；
  - 025 Manifest 补 Protocol target；020 `expected_command_changes` 已修正；
  - 010 的授权/影响不一致与 021 的 Compose 路径缺口已记录，不追溯改写；
  - `docs/reviews/README.md`、审计响应与计划已更新；
  - `changes/G02-historical-audit.yaml` 记录本次 Modify 范围。
- 验证：
  - 实际执行：文档相对链接检查、`git diff --check`、确认 `crates/` 与 `apps/` 无改动；
  - 未执行：`cargo test` / `npm run test:ui`（仅文档/记录变更）；
  - 未执行机器 YAML 解析（环境无解析器；G02 Manifest 结构人工核对）。
- 遗留：
  - G03 历史未实现工作回溯已在 2026-11-27 完成，见 [回溯报告](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)；
  - 010/021 的授权分类保留为审计建议，不追溯补授权。
- 下一步：G03 — 历史未实现工作回溯。
