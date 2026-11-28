# examples/

v0.1 的项目样例，同时是 M0 的契约测试数据集。每个样例是完整项目目录，入口固定为
`project.toml`；`expected.json` 是机器可读的预期结果。

## 1. `expected.json`

```json
{
  "valid": false,
  "diagnostics": ["structure-slot-cardinality"],
  "notes": "…"
}
```

- `valid`：项目是否应在 v0.1 通过验证；
- `diagnostics`：预期出现的错误代码集合（v1 见 `error-codes-v1.md`，v2 见 `error-codes-v2.md`）；
- `notes`：人读说明。
- 样例目录中的 `README.md` 说明目的、覆盖类别与预期触发点。
- 样例不包含 `.designer/`，因为本目录只提供项目语义数据；表现层隔离由 M3 的专门测试覆盖。

## 2. 目录

| 目录 | 内容 | 预期 |
|---|---|---|
| `minimal-project/` | Root + 叶子 + 一个必需 Slot | valid |
| `nested-composition/` | 四层嵌套 + 多模块复用 | valid |
| `edge-valid/slot-many/` | `many` + required，多实例复用 | valid |
| `edge-valid/dependency-provider/` | 显式 provider + 唯一自动解析 | valid |
| `edge-valid/format-preserved/` | 注释、空行、乱序字段 | valid |
| `invalid-projects/slot-cardinality/` | 必需 Slot 缺失 / 单值 Slot 重复接入 | invalid |
| `invalid-projects/references/` | 模块、版本、实例、Slot、Port 引用错误 | invalid |
| `invalid-projects/properties/` | 未知、类型、枚举、必填属性错误 | invalid |
| `invalid-projects/structure/` | 多父、重复 Binding、环、孤立、多 Root | invalid |
| `invalid-projects/versions/` | 版本缺失 | invalid |
| `invalid-projects/files/` | 入口缺失、路径越界、TOML 语法错误 | invalid |
| `invalid-projects/connections/` | 方向、类型、重复端点 | invalid |
| `edit-cases/` | 事务失败、Revision 冲突、重复请求（M2） | 场景说明 |
| `external-change-cases/` | 外部合法/非法修改、未保存冲突（M3） | 场景说明 |

### v2.0 数据集（035）

`v2/**` 使用 `schema_version = 2`，覆盖接口身份、service scope、版本矩阵与迁移拒绝。
每个目录带 `README.md` 与 `expected.json`。

| 目录 | 内容 | 预期 |
|---|---|---|
| `v2/valid-service-interface/` | 接口依赖 + primitive service | valid |
| `v2/warning-service-unused/` | 未被引用的 service | valid + warning |
| `v2/invalid-interface-missing/` | 接口身份无声明 | `ref-interface-not-found` |
| `v2/invalid-interface-duplicate/` | 同模块重复接口 | `id-duplicate-interface` |
| `v2/invalid-dependency-targets/` | module + interface 同时出现 | `schema-inconsistency` |
| `v2/invalid-service-binding/` | service 进入 Binding | `structure-service-in-binding` |
| `v2/invalid-root-service/` | Root 为 service | `structure-root-scope-invalid` 等 |
| `v2/invalid-service-composite/` | composite 模块作为 service | `contract-service-module-invalid` |
| `v2/invalid-version-mismatch/` | v2 项目使用 v1 Composition | `schema-version-mismatch` |
| `v2/invalid-provider-mismatch/` | provider 未实现接口 | `dependency-interface-mismatch` |
| `v2/invalid-v1-project-v2-module/` | v1 项目加载 v2 模块 | `schema-unsupported-version` |

### v2.1 数据集（041）

`v3/**` 使用 `schema_version = 3`，覆盖默认/具名 Profile 与 v3 版本矩阵。
每个目录带 `README.md` 与 `expected.json`；默认 profile 是
`workspace.composition`。

| 目录 | 内容 | 预期 |
|---|---|---|
| `v3/valid-profiles/` | 默认 + desktop/headless 两个具名 profile | valid |
| `v3/invalid-reserved-profile/` | 声明 `[profiles.default]` | `profile-reserved-name` |
| `v3/invalid-composition-version/` | v3 项目的 Composition 仍为 v2 | `schema-version-mismatch` |

## 3. 设计基线 §12.1 覆盖矩阵

| §12.1 类别 | 样例 |
|---|---|
| 1 最小合法项目 | `minimal-project/` |
| 2 嵌套组合 | `nested-composition/` |
| 3 多实例复用 | `nested-composition/`、`edge-valid/slot-many/` |
| 4 Slot 基数边界 | `edge-valid/slot-many/`、`invalid-projects/slot-cardinality/*` |
| 5 非法引用 | `invalid-projects/references/*` |
| 6 属性错误 | `invalid-projects/properties/*` |
| 7 结构错误 | `invalid-projects/structure/*` |
| 8 版本错误 | `invalid-projects/versions/version-missing/`、`invalid-projects/references/module-version-not-found/` |
| 9 文件错误 | `invalid-projects/files/*` |
| 10 编辑错误 | `edit-cases/README.md`（M2 补载荷） |
| 11 外部修改 | `external-change-cases/README.md`（M3 补序列） |
| 12 格式保留 | `edge-valid/format-preserved/` |
| v2 接口/service 合法 | `v2/valid-service-interface/`、`v2/warning-service-unused/` |
| v2 接口/service 诊断 | `v2/invalid-*/` |
| v1/v2 迁移拒绝 | `v2/invalid-version-mismatch/`、`v2/invalid-v1-project-v2-module/` |

## 4. 使用约定

- 合法样例在 M1 的 `validate` 上必须全部通过；非法样例的代码集合必须与 `expected.json` 一致。
- 样例中的字段名、错误代码以 `docs/spec/` 为准；样例不是规范的替代品。
- 新增样例必须同时提供 `README.md` 与 `expected.json`，并更新本文件的覆盖矩阵。
