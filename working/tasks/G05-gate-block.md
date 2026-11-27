# G05 — change-gate block 模式

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：在规则审计后把 CI 的 change-gate 从 report 切换为 block，并修复重叠 Manifest 导致的误报。
- authorization：
  - required：modify, extend, rearchitect
  - approval：
    - modify：granted（用户 2026-11-27 明确要求进行 UW-030）
    - extend：granted（CI workflow 变更；用户 2026-11-27 明确要求进行 UW-030）
    - rearchitect：granted（用户 2026-11-27 明确要求进行 UW-030；启用强制门禁策略）
- 来源：UW-030（G04 遗留）
- 变更声明：`changes/G05-gate-block.yaml`

## 范围与关键决策

1. CI `Change gate` 步骤改为 `--mode block`；声明/授权/覆盖不一致会让 job 失败。
2. 修复 Manifest 重叠误报：一个文件被多个 `targets` 覆盖时，只要**任一**覆盖
   Manifest 满足该文件所需的授权/operation/forbidden 约束，即视为通过；否则报告
   最具体 Manifest 的失败。
3. 把 `changes/000-template.yaml` 加入敏感路径（修改需 Rearchitect）。
4. 更新 AGENTS、工具 README、验收记录。
5. 不新增产品命令、不修改 Core/Schema/设计基线。

## 清单

- [x] 审计 report 模式与 Manifest 重叠行为
- [x] 实现“任一覆盖 Manifest 授权即可”的选择规则
- [x] 增加重叠 Manifest 回归测试
- [x] CI 切换 `--mode block`
- [x] 更新 AGENTS / 工具 README / 机器门禁记录
- [x] 创建 `changes/G05-gate-block.yaml`

## 验收

- block 模式在当前仓库 diff 上 0 error，CI job 通过；
- 一个文件同时被严格 Manifest 与普通 Modify Manifest 覆盖时，普通 Manifest 可授权；
- 未声明路径、缺少授权、敏感路径、新 crate、Compose、coverage 不一致仍会失败；
- 不新增产品能力；
- 变更范围与 Manifest 一致。

## 结果 / 遗留问题

- 结果：
  - CI 改为 `--mode block`；
  - 重叠 Manifest 选择规则改为“任一覆盖 Manifest 授权”；
  - 新增 `accepts_any_authorized_covering_manifest` 回归测试；
  - `changes/000-template.yaml` 纳入敏感路径。
- 验证（本地）：
  - 8 个 change-gate 单元测试 + 1 个真实仓库集成测试；
  - workspace **164 passed**；
  - clippy / fmt 通过。
- 待 CI 验证：block 模式在 PR 上通过后合并；详见 `docs/reviews/2026-11-27-machine-gate.md`。
- 下一步：M7 Source Authoring（需重新立卡与授权）。
