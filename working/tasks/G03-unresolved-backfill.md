# G03 — 历史未实现工作回溯

- 状态：queued（G02 之后执行）
- coverage_status：partial
- open_items：无
- 目标：对 001–025 做一次未实现工作回溯，填充 `working/decisions/unresolved-work.md`，并给有未决项的卡补 `coverage_status` / `open_items`。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：`docs/reviews/2026-11-27-unimplemented-work-transparency.md` §10
- 变更声明：`changes/G03-unresolved-backfill.yaml`（开始后创建）

## 范围

1. 逐张读取 001–025 的目标、范围、清单、验收、结果，提取：
   - 未实现、只读、暂缓、后续、需要授权、范围外发现的能力；
   - 目标提到但结果没有实现证据的能力；
   - 后续任务声称解决但没有实现证据的能力。
2. 为每个未决事项分配稳定 `UW-*` ID，写入
   `working/decisions/unresolved-work.md`：
   - 类型、状态、原因、所需授权、影响、最小方案、决策选项、首次/最近复核日期；
   - 已实现事项（如 025 Provider）保留关闭证据，不重复列为待决；
   - 无法证明历史意图的事项标 `historical_status_unknown`。
3. 对仍有未决项的卡补充：
   - `coverage_status`、`open_items`；
   - 精简的“需求覆盖与未实现项”表，不重写原范围。
4. 更新 `working/plan.md` 的未决工作总览与计数。

## 非目标

- 不实现任何未决能力；
- 不修改 G1–G3 的 Schema/领域模型；
- 不把“后续任务提到过”当作解决证据；
- 不伪造当时的用户授权或决策。

## 验收

- 001–025 每张卡的所有“遗留/后续/暂缓”线索都有去向：
  `implemented` / `pending_*` / `deferred` / `rejected` / `superseded` /
  `historical_status_unknown`；
- 每个未决事项可在登记册中检索，含影响、授权、后续处理；
- 已解决事项有实现任务与测试/验收证据链接；
- 计划页与登记册一致，未决数量、等待授权数量可核对；
- 历史卡原始范围与验收文本不被删除或改写。

## 结果 / 遗留问题

- 初始登记册与首批 UW 已在本轮计划整理中建立；本卡负责全量回溯与证据核对。
