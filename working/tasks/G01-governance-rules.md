# G01 — 授权规范与未实现披露制度

- 状态：todo（**等待 Rearchitect 授权**，未开始）
- 目标：把授权从单值等级改为可组合权限集合，并建立未实现工作的登记、覆盖状态与汇报规则；只改规则和模板，不改产品代码。
- 授权：Rearchitect + Modify（`AGENTS.md`、任务卡模板、Manifest 模板、汇报规范）
- 来源：`docs/reviews/2026-11-27-authorization-audit.md`、`docs/reviews/2026-11-27-unimplemented-work-transparency.md`
- 变更声明：`changes/G01-governance-rules.yaml`（待授权后创建）

## 背景

两份评审指出同一类系统性问题：

- 权限是权限集合，不是单一阶梯；现有“取所需最低一项”无法表达 Extend + Modify、
  Extend + Rearchitect 等组合；
- 工程配置、文档修改、Compose 路径、外部副作用缺少明确边界；
- 未实现事项通常只在结果末尾用一句话带过，任务 `done` 可能掩盖未完成的能力；
- 当前没有机器门禁，Manifest 只是书面契约。

本卡只修订规则与模板。**不**修改领域模型、TOML Schema、命令实现、Designer 代码，
也**不**追溯改写给历史任务卡补授权。

## 范围

### 授权模型

- `AGENTS.md` §1 改为权限集合：`required` 列出实际需要的全部模式，
  `approval` 记录每个模式的授权状态与依据；
- 明确 Modify / Compose / Extend / Rearchitect 的边界与组合规则；
- 工程配置边界：注册 workspace member vs 改变 workspace 拓扑；CI 测试覆盖 vs 强制门禁；
- 文档权限：任务结果/README/plan/architecture 现状可随任务授权更新；修改 `AGENTS.md`
  规则、设计基线、领域/Schema/协议语义需要更高授权；
- Compose 范围增加 `working/pilots/**`（只允许声明式试点，不允许修改生产代码/模块实现）；
- 新依赖、Shell、文件删除、安装、远端服务、push/发布等副作用独立判定，
  不随 Extend/Rearchitect 自动获得。

### 任务卡与 Manifest 模板

- `working/tasks/000-template.md`：
  - `authorization.required` + `approval` 状态；
  - `status` / `coverage_status` / `open_items`；
  - 原始目标覆盖表；
  - 未实现/被搁置事项表；
  - 范围调整记录；
  - 完成判定清单。
- `changes/000-template.yaml`：
  - `authorization.required/approved/evidence`；
  - `operations` / `targets` / `scope` 分离；
  - `unresolved_items`；
  - 明确“声明覆盖实际 diff”的核对要求。
- 未决工作登记册格式：
  - `working/decisions/unresolved-work.md` 稳定 UW ID、状态机、关闭规则；
  - `working/decisions/decision-history.md` 记录用户决策与授权依据。

### 汇报规范

- 任务结束报告必须包含独立“未实现 / 被搁置功能”章节；
- 每项说明：用户缺失的操作、当前替代行为、原因、影响、所需授权、最小方案、
  决策选项、UW ID；
- 没有未实现项时明确写“无”；
- `status: done` 不得单独用于宣称更大功能目标完成。

## 非目标

- 不重写 001–025 历史任务卡的授权字段；
- 不建立机器门禁（G04）；
- 不推进 026 Change Review；
- 不改 `docs/design/**`、`docs/spec/**` 的语义；
- 不实现任何新的产品能力。

## 清单

- [ ] 获得用户对 Rearchitect 范围的明确批准
- [ ] 修订 `AGENTS.md` §1–§3、§6、§8
- [ ] 修订 `working/tasks/000-template.md`
- [ ] 修订 `changes/000-template.yaml`
- [ ] 定义 `working/decisions/unresolved-work.md` 状态机与关闭规则
- [ ] 定义 `working/decisions/decision-history.md` 记录格式
- [ ] 定义任务结束汇报模板与阶段结项检查
- [ ] 更新 `working/plan.md` 未决工作总览与统计口径
- [ ] 添加迁移说明：旧卡如何补 `coverage_status`/`open_items`，哪些字段不追溯
- [ ] 创建 `changes/G01-governance-rules.yaml`

## 验收

- `AGENTS.md` 不再要求“取最低一项”，而是要求列出全部所需权限与审批依据；
- 新任务卡模板和 Manifest 模板能表达 Extend + Modify、Compose + Modify 等组合；
- Compose 明确覆盖 `working/pilots/**` 且保留“不得改生产代码”的限制；
- 未实现披露规则可被后续任务卡与汇报直接执行；
- 规则变更没有修改任何产品代码、领域模型、TOML Schema 或命令协议；
- 旧任务卡的授权历史保持不变，迁移说明明确不追溯。

## 风险

- `AGENTS.md` 是每个会话都会读取的规则文件，修改后所有任务的行为基线都会变化；
- 若模板过重，可能降低小任务的执行效率；需要保持最小必填字段；
- 若把历史卡全部强制迁移，会产生大量低价值改稿；本卡只定义迁移规则，不改历史正文。

## 结果 / 遗留问题

- 尚未开始；等待 Rearchitect 授权。
