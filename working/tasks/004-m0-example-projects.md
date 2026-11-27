# 004 — M0 项目样例集（合法 / 非法 / 边界）

- 状态：done
- 目标：在 `examples/` 建立覆盖设计基线 §12.1 十二类场景的项目样例，每个非法样例带预期诊断代码，
  作为 M1 的验收测试数据。
- 授权：Extend（仅 `examples/`）
- 变更声明：无（仅 `examples/`；目录已由 001 建立）

> 授权审计（2026-11-27）：审计建议 **Extend + Compose**（示例类型 + 真实组合声明）；原始授权字段保持不变。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

- coverage_status：complete_with_deferred_items
- open_items：UW-026
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

设计基线 §12.1 要求为 12 类错误与边界准备样例，但仓库里还没有任何项目目录。样例既是
M0 的交付物，也是 M1 解析 / 校验的回归数据；没有样例，冻结的 Schema 无法被证明可执行。
样例必须在 002 / 003 之后建立，避免字段名与错误代码返工。

## 清单

- [x] `minimal-project/`：一个 Root + 一个叶子模块
- [x] `nested-composition/`：四层 Composite 与 Primitive
- [x] 多实例复用：`nested-composition/` 与 `edge-valid/slot-many/` 复用模块定义
- [x] Slot 基数边界：必需 Slot 缺失 / 单 Slot 重复接入 / many Slot 合法接入
- [x] 非法引用：Module / 版本 / Instance / Slot / Port 不存在
- [x] 属性错误：未知属性、类型错误、枚举值错误、缺少必填属性
- [x] 结构错误：多个 Root、孤立实例、多父节点、重复 Binding、环
- [x] 版本错误：模块版本缺失、指定版本不可用
- [x] 文件错误：入口缺失、路径越界、TOML 语法错误
- [x] 编辑错误场景：事务失败、Revision 过期、重复 request ID（`edit-cases/README.md`，M2 补载荷）
- [x] 外部修改场景：合法 / 非法 / 与未保存内容冲突（`external-change-cases/README.md`，M3 补序列）
- [x] 格式保留样例：`edge-valid/format-preserved/`（注释、空行、乱序字段）
- [x] 每个样例目录的 README：目的、预期诊断、M1 起验证命令
- [x] 根 README 的覆盖清单与设计基线 §12.1 映射

## 验收

- 12 类每类至少 1 例，目录与命名可被测试扫描：共 26 个 `expected.json` 数据集，
  其中 25 个完整项目 + 1 个入口缺失目录。
- 每个非法样例声明预期错误代码，且与 003 的错误代码表一致（21 个非法数据集）。
- 合法样例满足 003 的必填与约束：临时校验器对 5 个合法样例返回空诊断。
- 不包含 `.designer/` 布局；A15 由 M3 的专门测试覆盖。
- 样例目录结构与设计基线 §4.1 一致，入口固定为 `project.toml`。
- 临时校验器对 26/26 数据集的代码集合与 `expected.json` 完全一致。

## 结果 / 遗留问题

- 结果：5 个合法数据集（minimal、nested、slot-many、dependency-provider、format-preserved）、
  21 个非法数据集（slot 2、references 5、properties 4、structure 5、versions 1、files 3、connections 1）；
  每个都带 `README.md` 与 `expected.json`。
- 校验：临时 Python 校验器镜像了 L1–L7 的主要规则，26/26 匹配；它不提交，不作为产品代码。
- 遗留：`edit-cases/` 与 `external-change-cases/` 只有场景说明；具体请求载荷/操作序列分别在
  009（命令 Schema）与 015（外部同步）冻结后补。连接错误样例超出 §12.1 的 12 类，但覆盖了
  L6 的方向/类型/重复端点代码。
- 下一步：M1 005（Rust 工程与领域类型），进入 M1 时创建任务卡。
