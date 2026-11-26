# ADR-0002：v0.1 显式不支持的能力

- 状态：Accepted
- 日期：2026-11-26
- 决策者：Composition Designer v0.1
- 关联：[`composition-schema-v1.md`](../spec/composition-schema-v1.md) §12、[`error-codes-v1.md`](../spec/error-codes-v1.md)

## 背景

设计基线要求“未冻结的能力应显式标记 unsupported，而不是以隐式行为代替尚未决定的规则”。
M0 冻结 Schema 时必须同时列出不做的事，否则实现者容易用“尽量兼容”补齐，破坏 A18（无隐式兼容）。

## 决策

v0.1 明确不支持以下能力；遇到时给出稳定诊断或直接忽略，不提供近似行为：

| 能力 | 行为 | 诊断/说明 |
|---|---|---|
| 多文件 Composition、include/import、`parts/` 合并 | 只读单入口 | 未识别文件不参与解析 |
| `object` 属性类型 | 拒绝 | `schema-property-type-unsupported` |
| 数组嵌套数组、数组元素为 object | 拒绝 | `schema-invalid-value` / `schema-property-type-unsupported` |
| 预发布版本、版本范围、通配、`latest` | 拒绝 | `schema-invalid-version` |
| 符号链接目录/文件 | 不跟随；越界按路径错误 | `file-path-escape` |
| 能力依赖（按 capability 而非模块 ID） | 只支持模块 ID 依赖 | 依赖目标必须模块 ID |
| Port 协议协商、自动类型转换 | 只做严格相等 | `connection-protocol-mismatch` / `connection-type-mismatch` |
| 多父、共享实例、无根项目、多 Root | 结构错误或不可表达 | 单父 + 单 `root` 字段 |
| 模块定义编辑 | 只读 | M3 不提供编辑入口 |
| 运行时执行、构建、部署、代码生成 | 不属于 v0.1 | 设计基线 §1.2 / §14 |
| 在线模块市场、复杂版本求解、自动 Adapter | 不做 | 设计基线 §1.2 |
| 多 Agent 调度与长时自主执行 | 不做 | 设计基线 §1.2 |

## 备选方案

| 方案 | 优点 | 缺点 | 未采用原因 |
|---|---|---|---|
| 用“宽松解析 + 忽略未知”换取兼容 | 老文件更容易打开 | 隐式行为不可解释，违反 A18 | 明确拒绝更安全 |
| 先支持 object 属性 | 配置表达力更强 | 嵌套 Schema、默认值、UI 编辑复杂度上升 | 先把标量/枚举/数组做稳 |
| 先支持多文件 Composition | 大型项目更自然 | include/循环/覆盖规则未定 | M0 不应冻结半成品语义 |
| 允许按 capability 依赖 | 更解耦 | 提供者解析与歧义规则未定 | 模块 ID 依赖已足够 v0.1 |
| 静默跳过符号链接 | 扫描更简单 | 用户难以发现模块“消失” | 显式文档化，路径越界报错 |

## 后果

- 正面：实现边界清晰；错误代码可机器判断；不会积累兼容补丁。
- 负面/成本：真实项目可能需要拆分目录或改用标量配置；后续版本需要 `schema_version` 升级。
- 需要配套的验证：M1 为每个不支持项提供负例测试；M5 检查没有隐式 fallback。
- 复审条件：用户提出具体用例且影响验收时，先新增 ADR、更新 Schema 与错误码，再实现。

## 关联

- 设计基线：§1.2、§13、§14、§15
- 相关 ADR：[ADR-0001](./0001-domain-and-schema-freeze.md)
- 相关规范：`docs/spec/composition-schema-v1.md` §12
