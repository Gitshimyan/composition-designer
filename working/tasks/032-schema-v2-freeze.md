# 032 — Schema v2.0 ADR 批准与接口/服务语义冻结

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-002、UW-003、UW-032
- 目标：冻结 Schema v2.0 的接口身份、依赖目标、实例 scope、诊断代码、命令 scope 与 v1 兼容矩阵，产出 ADR-0006 与 v2.0 规范，不实现代码。
- authorization：
  - required：modify、extend、rearchitect
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28）
    - rearchitect：granted（用户 2026-11-28）
- 来源：`working/plan.md` S1；ADR-0004 Accepted；用户明确授权先文档后 032
- 变更声明：`changes/032-schema-v2-freeze.yaml`

## 背景

ADR-0004 只确定了 v2 的方向（v2.0 = G1 接口身份 + G2 service scope），具体字段、校验顺序、
诊断代码与命令形态尚未冻结。033–035 若直接实现，会在 Loader/Validation/Commands 中分别发明
规则。本卡把 v2.0 语义固化为可实现的规范，并明确 v2.1（Profile/多入口）与接口方法形状
不在 v2.0 范围。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | v2.0 `schema_version` 与文件兼容矩阵 | implemented | `composition-schema-v2.md` §1、§11 |
| REQ-02 | 接口声明字段、唯一性与对象标识 | implemented | §2、§3.1、§5 |
| REQ-03 | 依赖目标 module/interface 二选一与解析规则 | implemented | §3.2、§6 |
| REQ-04 | 实例 `scope = structure\|service` 与结构规则 | implemented | §4、§7 |
| REQ-05 | 新增诊断代码与级别 | implemented | `error-codes-v2.md`、§5 |
| REQ-06 | 命令增量（`create_instance.scope`）与 v1 行为 | implemented | §8、`command-schema-v1.md` 指针 |
| REQ-07 | 规范化输出增量与确定性 | implemented | §9 |
| REQ-08 | v1 项目不破坏、不自动迁移 | implemented | §11 |
| REQ-09 | v2.1 与接口方法形状明确排除 | implemented | §12；UW-032 登记 |
| REQ-10 | ADR 与规范索引可追溯 | implemented | ADR-0006、`docs/spec/README.md` |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-002 | 接口身份实现 | Rearchitect | 032 只冻结语义；033–035 实现 | Rearchitect + Extend | 依赖仍按模块 ID | 033–035 |
| UW-003 | service scope 实现 | Rearchitect | 同上 | Rearchitect + Extend | 服务仍塞进结构树 | 033–035 |
| UW-032 | 接口方法/Port 形状、service 内部组合 | Rearchitect | v2.0 明确只做身份标记；service 必须 primitive | Rearchitect | 复杂接口契约与组合服务尚不可表达 | 出现真实需求时评估 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 冻结 v2.0 | 只冻结 v2.0；v2.1/接口形状排除 | ADR-0004 分阶段；避免把未决语义绑进 033 | ADR-0006 |

## 清单

- [x] 冻结版本矩阵与 v1 兼容规则
- [x] 冻结接口声明、依赖目标与解析语义
- [x] 冻结实例 scope、service 约束与结构规则
- [x] 冻结新增诊断代码与对象标识
- [x] 冻结命令增量与规范化输出
- [x] 写 ADR-0006 并更新 ADR-0004 冻结记录
- [x] 更新 spec 索引、v1 指针、plan、UW 与架构入口

## 验收

- v2.0 的字段、默认值、约束、校验层与错误代码可逐项实现；
- 每个新代码有触发条件、对象与级别；
- v1 项目行为不变，v2 功能只在 `schema_version = 2` 启用；
- 命令增量不引入新命令，v1 上显式 `scope` 被拒绝；
- 规范之间无互相矛盾，索引与 ADR 可追溯；
- 无产品代码/Schema 实现/示例改动；
- 变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - 新增 [ADR-0006](../../docs/adr/0006-schema-v2-freeze.md)：9 条决策（版本矩阵、接口标记、
    依赖目标、解析、scope、primitive 约束、命令、输出、v2.1 排除）、备选方案与复审条件；
  - 新增 [composition-schema-v2.md](../../docs/spec/composition-schema-v2.md) §1–§13：
    版本/兼容矩阵、接口标识、module 与 composition 增量、不变量、依赖与结构规则、
    命令、规范化输出、校验顺序、v1 兼容、非目标、035 样例计划；
  - 新增 [error-codes-v2.md](../../docs/spec/error-codes-v2.md)：8 个新代码
    （`schema-version-mismatch`、`id-duplicate-interface`、`ref-interface-not-found`、
    `contract-service-module-invalid`、`structure-service-in-binding`、
    `structure-root-scope-invalid`、`dependency-interface-mismatch`、`service-unused`）；
  - v1 规范加指针（`composition-schema-v1.md` 顶部/§12、`domain-model-v1.md` 顶部、
    `command-schema-v1.md` §2 v2.0 注记）；
  - 更新 ADR-0004 冻结记录、spec 索引、README、架构入口、plan、UW 与 D-022。
- 验证：
  - 相对链接检查：全部可解析；
  - v2 代码与规范交叉检查：8 个新代码均出现在 schema v2；spec 中其余代码均为 v1 代码；
  - UW 行数 32，状态分布 11 / 5 / 1 / 2 / 13，与 plan §8 一致；
  - `git diff --check` 通过；无 `crates/`、`apps/designer/src`、`examples/` 改动；
  - `composition-change-gate --mode block`（对工作树临时提交 `ad74383`，base HEAD）：
    17 个文件、39 个 Manifest，**0 error / 0 warning**。
- 未执行：`cargo test`（本卡无代码变更）；CI（尚未推送/建 PR）；033–035 的实现与 fixtures。
- 结果：v2.0 语义可逐项实现；v1 行为与输出不变；v2.1、接口形状、service 组合明确排除。
  `coverage_status = complete_with_deferred_items`：冻结范围完成，UW-002/UW-003 的实施在
  033–035，UW-032 为 v2.0 明确排除项。
- 遗留 / 后续：033 需另行取得 Rearchitect + Modify 授权；035 按 schema v2 §13 建数据集。
