# 033 — Domain/Loader/Validation：接口标记与 service scope

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-002、UW-003
- 目标：按 032 冻结的 v2.0 语义实现领域类型、TOML 解码与 L2–L7 校验：接口声明/接口依赖、实例 scope、8 个新诊断、版本矩阵与 v1 回归。
- authorization：
  - required：modify、rearchitect
  - approval：
    - modify：default
    - rearchitect：granted（用户 2026-11-28）
- 来源：`working/plan.md` S1；032 冻结语义（ADR-0006）
- 变更声明：`changes/033-v2-domain-validation.yaml`

## 背景

032 已冻结 v2.0：`schema_version` 矩阵、`[[contract.interfaces]]`、依赖目标
`module`/`interface` 二选一、实例 `scope`、8 个诊断与规范化输出增量。033 只实现
Domain/Loader/Validation，不新增命令、不改 UI/示例。v1 行为与规范化输出必须保持不变。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | 领域类型：接口、依赖目标、scope、输出 target | implemented | `composition-domain` |
| REQ-02 | 错误代码 8 个与层/级别 | implemented | `error_code.rs` |
| REQ-03 | 解码 v1/v2 版本矩阵与 v2 字段 | implemented | `composition-core/src/decode.rs` |
| REQ-04 | L2–L7 v2 校验与级联 | implemented | `composition-core/src/validate.rs` |
| REQ-05 | v1 项目行为/输出不变 | implemented | 既有 173 + 新增测试 |
| REQ-06 | 接口解析、service 结构规则、service-unused | implemented | 单元/集成测试 |
| REQ-07 | 确定性输出 | implemented | 排序与 JSON 测试 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-002 | 接口身份的完整入口（命令/UI/验收） | Rearchitect | 034/035 实现 | Extend + Modify | CLI/GUI 尚不可创建接口/scope | 034–035 |
| UW-003 | service scope 的完整入口 | Rearchitect | 034/035 实现 | Extend + Modify | UI 尚无 Services 视图 | 034–035 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| — | 033 Domain/Loader/Validation | 不做命令/UI/示例 | plan 卡范围；034/035 承接 | plan.md S1 |

## 清单

- [x] Domain：InterfaceId/InterfaceDeclaration/DependencyTarget/InstanceScope
- [x] Domain：Contract 接口与序列化、DependencyResolution.target
- [x] Domain：8 个错误代码、ObjectId::interface
- [x] Loader：接受 project schema_version 2
- [x] Decode：版本矩阵、interfaces、interface 依赖、scope
- [x] Validate：接口注册/解析、service 规则、service-unused、v1 回归
- [x] 测试：新增 v2 正反例；workspace/clippy/fmt 通过
- [x] 回写 plan/UW/架构入口

## 验收

- v2 合法项目可解析并通过校验；
- 032 §13 的正反例诊断按代码产生（035 建正式数据集前由单元测试覆盖）；
- v1 项目诊断与 JSON 输出不变；
- `cargo test --workspace` / `clippy` / `fmt` 通过；
- 无命令、UI、示例、设计基线改动；
- 变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - Domain：`InterfaceId`、`InterfaceDeclaration`、`DependencyTarget`（module/interface）、
    `InstanceScope`、`Instance.scope`、`Contract.interfaces`（v2 才序列化）、
    `DependencyResolution.target`、`ObjectId::interface`；8 个新错误代码与层/级别；
  - Loader：接受 `project.schema_version = 2`，其余值仍 `schema-unsupported-version`；
  - Decode：版本矩阵由校验层统一裁决；v2 模块的 `interfaces`/`interface` 依赖、
    v2 Composition 的 `scope`；v1 字段名仍按未知字段拒绝；
  - Validate：接口声明/解析（显式与自动、歧义、未声明、provider 不匹配）、
    service 必须 primitive、Root 必须 structure、service 不参与 Binding 与可达性、
    Slot 基数只统计 structure、`service-unused` warning；
  - v2 输出新增 `interfaces`（含空数组）、`scope`、`dependencies[].target`；v1 输出不变。
- 验证：
  - `cargo test --workspace`：**189 passed**（新增 `tests/v2.rs` 14 条 + loader 1 条）；
  - `cargo test --manifest-path apps/designer/src-tauri/Cargo.toml`：**16 passed**
    （API 变更后宿主仍编译，行为不变）；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - `composition-change-gate --mode block`（工作树临时提交，base HEAD）：19 个文件、
    40 个 Manifest，**0 error / 0 warning**；
  - 覆盖：v2 合法/规范化/确定性、未知接口、重复接口、双目标、service Binding、
    Root service、composite service、版本不匹配、provider 不匹配、service-unused、
    无版本接口歧义、v1 模块混用、v1 项目拒绝 v2 模块、v1 输出无 v2 字段。
- 未执行：Playwright UI（本卡无 `apps/designer/src` 改动）；035 正式 v2 数据集与 Glide 试点验收。
- 结果：032 冻结语义已在 Core 落地；v1 行为与 JSON 输出保持兼容；服务/接口进入
  规范模型，但命令与 UI 入口仍待 034。
- 遗留 / 后续：UW-002/UW-003 保持 in_progress，待 035 验收后关闭；034 处理
  `create_instance.scope`、Services 视图与接口展示。
