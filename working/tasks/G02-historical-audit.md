# G02 — 历史授权审计记录与 Manifest 修正

- 状态：proposed（G01 之后执行；可先做记录不改规则）
- 目标：把 001–025 的授权审计结论固化为可追溯记录，并修正 Manifest 的事实性覆盖遗漏；不伪造历史授权。
- 授权：Modify（`docs/reviews/`、`changes/`、`working/`）
- 来源：`docs/reviews/2026-11-27-authorization-audit.md`
- 变更声明：`changes/G02-historical-audit.yaml`（待开始后创建）

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

## 非目标

- 不为 002–004 伪造不存在的 Manifest；
- 不把审计建议写成“当时已获授权”；
- 不修改产品代码、Schema 或命令协议；
- 不启动 G01 的规则修订（G01 未批准时本卡只做记录）。

## 验收

- 001–025 每张卡在审计记录中都有分类与证据状态；
- 已确认的 Manifest 文件/语义遗漏被修正，且与实际 diff 一致；
- 未确认的历史授权保持“审计建议/证据不足”，没有倒填；
- `docs/reviews/README.md` 与新审计记录互相链接；
- 不改动任何产品实现文件。

## 结果 / 遗留问题

- 025 Manifest 的 Protocol target 与 020 的 `expected_command_changes` 已在本轮计划整理中修正；
  其余记录待本卡执行。
