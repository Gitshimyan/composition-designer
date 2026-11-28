# 039 — Profile/多入口语义冻结与 ADR

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-004、UW-018
- 目标：冻结 Schema v2.1（format 3）：项目级 Profile、多 Composition 入口、选择/Revision/Digest 语义与错误码；不实现代码。
- authorization：
  - required：modify、extend、rearchitect
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 授权 S2）
    - rearchitect：granted（用户 2026-11-28 授权 S2）
- 来源：`working/plan.md` S2 039；ADR-0004 G3
- 变更声明：`changes/039-profile-adr.yaml`

## 背景

ADR-0004 已确认 Profile 采用“每个 profile 一份完整 Composition 入口”，但没有冻结
`schema_version`、`[profiles]` 字段、选择语义、Revision/Digest 范围与错误码。
040/041 若直接实现会各自发明规则。039 只冻结语义，不改 Core/命令/UI。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | format 版本与 v1/v2.0/v3 兼容矩阵 | implemented | ADR-0007、schema-v3 §1 |
| REQ-02 | `[profiles.<name>]` 字段与默认 profile | implemented | §2、§3 |
| REQ-03 | 选择语义（默认/显式/缺失/reserved） | implemented | §4 |
| REQ-04 | Revision/Digest 按所选 profile | implemented | §5 |
| REQ-05 | 事务与 Source 只写所选入口 | implemented | §6 |
| REQ-06 | 规范化输出增量 | implemented | §7 |
| REQ-07 | 3 个新错误码 | implemented | `error-codes-v3.md` |
| REQ-08 | v1/v2.0 不破坏、不自动迁移 | implemented | §9 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-004 | Profile 实现 | Rearchitect | 039 只冻结 | Rearchitect + Extend | 仍单入口 | 040–041 |
| UW-018 | 多入口实现 | Rearchitect | 同上 | Rearchitect + Extend | 仍单 Composition | 040–041 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 冻结 v2.1 | 仅冻结 v2.1；不实现 | 先规范后代码 | plan S2 |

## 清单

- [x] ADR-0007（决策/备选/后果）
- [x] composition-schema-v3（format 3）
- [x] error-codes-v3（3 个新代码）
- [x] v2 规范指针、spec 索引
- [x] plan/UW/决策历史回写

## 验收

- format 3 的文件集合、`[profiles]`、默认与显式选择、reserved 名称可逐项实现；
- Revision/Digest、事务、Source 的 profile 作用域明确；
- 3 个新错误码有触发/对象/级别；
- v1/v2.0 行为不变、不自动迁移；
- 无产品代码/Schema 实现/示例改动；
- 变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - [ADR-0007](../../docs/adr/0007-schema-v21-profiles.md)：format 3、Profile 表、选择、
    Revision/Digest、事务/Source、输出、错误码、兼容与备选方案；
  - [composition-schema-v3.md](../../docs/spec/composition-schema-v3.md)：§1–§12 可实现规范；
  - [error-codes-v3.md](../../docs/spec/error-codes-v3.md)：`profile-not-found`、
    `profile-unsupported`、`profile-reserved-name`；
  - 更新 v2 规范指针、ADR-0004 冻结记录、spec 索引、README、架构入口、plan、UW、D-029。
- 验证：
  - 相对链接检查通过；
  - v3 规范与错误码交叉检查：3 个新代码均有触发/对象/级别；
  - 无 `crates/`、`apps/`、`examples/` 改动；
  - change-gate block（工作树临时提交）：13 个文件、46 个 Manifest，**0 error / 0 warning**。
- 未执行：`cargo test`（无代码变更）；040–041 实现与 fixtures。
- 结果：v2.1 语义可逐项实现；v1/v2.0 不变；Profile overlay/继承/自动迁移明确排除。
  `coverage_status = complete_with_deferred_items`：冻结完成，实现待 040–041。
- 遗留 / 后续：040 需实现 Project/Loader/Core/Session；041 入口与验收；UW-004/018
  在 041 验收后关闭。
