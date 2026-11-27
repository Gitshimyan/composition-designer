# 006 — M1 TOML 加载与 Source Map

- 状态：done
- 目标：新建 `composition-toml` crate，用 `toml_edit` 加载 `project.toml`、Composition 入口与
  所有 `module.toml`，保留文本、TOML 文档、字节 span 与行列映射；实现语义文件集合与
  `source_digest`。**本卡不把 Composition / Module 解码成领域对象，也不做 L3–L7 校验**（007）。
- 授权：Extend（新 crate、`toml_edit` / `sha2` 依赖）
- 变更声明：`changes/006-toml-loader.yaml`

- coverage_status：complete_with_deferred_items
- open_items：UW-027
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

005 的领域层不知道 TOML 语法。006 要把“文件 → 可定位的 TOML 文档”这一段独立出来：
后续 007 需要从 `LoadedFile::document()` 读取表、用 `span()` 生成 `SourceRef`；008 需要
稳定的 `source_digest`。如果 006 提前做引用/结构校验，会与 007 重复并产生级联噪声。

范围边界：

- 做：路径规则、文件读取（UTF-8 / BOM）、TOML 解析、模块文件发现、项目路径声明解码、
  L1 诊断、Project 的 L2 字段解码、`source_digest`。
- 不做：Composition / Module / Instance / Binding 的语义解码（007）、引用与结构校验、
  TOML 编辑、GUI。

## 清单

- [x] 根 workspace 加入 `crates/composition-toml`（依赖 domain、`toml_edit`、`sha2`）
- [x] `LoadedFile`：相对路径、文本、行索引、可选 `ImDocument`、byte span → `SourceRange`
- [x] 路径规则：绝对/反斜杠/空段/尾随斜杠/`.`/`..` 越界；`module_roots` 去重与互不嵌套
- [x] 项目加载：读取 `project.toml`，解码 `[project]` / `[workspace]`，产出 `domain::Project`
- [x] 发现 Composition 入口与 `module_roots` 下的 `module.toml`（跳过 `.` 目录与符号链接）
- [x] 解析所有 TOML 文件；语法/重复键错误带文件与范围诊断；坏文件保留文本但不提供文档
- [x] `source_digest`：项目 + Composition + 模块文件，按相对路径排序，严格按 schema §9 算法
- [x] 诊断：L1 + Project L2；不产生 L3–L7 代码
- [x] 集成测试：minimal / nested / format-preserved / syntax / duplicate-key / path-escape /
      entry-missing；25 个完整项目只产生预期 L1 诊断；digest 稳定性与注释敏感性
- [x] `cargo test` / `clippy` / `fmt` 全绿；更新 plan、crates/README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**85 passed / 0 failed**（domain 58 + toml 单元 12 + toml 集成 15）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`：通过；
- `minimal-project` 加载 4 个文件、`project` 字段正确、无诊断、`source_digest` =
  `sha256:10cef70e22f16ef050fd9d1ea3d0f4ba3502235641aaf4f88b91260daa34768e`；
- `invalid-projects/files/toml-syntax` → `toml-syntax-error`；多个 root → `toml-duplicate-key`；
  `path-outside` → `file-path-escape`；`entry-missing` → `file-project-missing`；
- 25 个完整项目中 22 个零诊断、3 个只产生上述 L1 诊断；语义非法项目不产生 L3–L7 代码；
- `[composition]` / `[[instances]]` / `[project]` / `[module]` 的 span 映射到正确行；
  `format-preserved` 的注释保留在 `LoadedFile::text()`；
- 相同文件集合重复加载 digest 相同；仅改注释会改变 digest。

## 结果 / 遗留问题

- 结果：新增 `crates/composition-toml`：
  - `LoadedFile`（`ImDocument<String>` + 文本 + 行索引）与 byte span → `SourceRange`；
  - `load_project`：project L2 解码、Composition 入口加载、模块递归发现、L1/L2 诊断；
  - `paths`：相对路径归一化、越界、`module_roots` 去重/嵌套；
  - `source_digest`：`sha256:` + 文件集合按路径排序，含已知向量测试；
  - 27 个新测试（12 单元 + 15 集成），workspace 共 85 项。
- 关键决策：
  - 用 `ImDocument` 而不是 `DocumentMut`：后者 `despan()` 会清掉 span，无法做 Source Map；
    M2 编辑时可再 `into_mut()`；
  - 006 只解码 Project 的路径声明；Composition / Module 的字段解码留给 007，避免重复校验；
  - Composition 语法错误后仍继续加载模块，保证 `source_digest` 覆盖完整语义文件集合；
  - `source_digest` 对语法错误文件仍可计算（字节可读），但项目入口/Composition 缺失时为 `None`。
- 遗留：
  - Composition / Module / Instance / Binding 的语义解码、引用与结构校验仍是 007；
  - 模块定义重复、`composition.id != project.id` 等 L3/L7 检查未在 006 实现；
  - 符号链接只做“不跟随/根越界”处理，Windows 上的实际符号链接行为未做专门集成测试；
  - `ImDocument` 是只读的；M2 的局部编辑需要另建可变解析路径。
- 下一步：007（解析与 L1–L7 校验），进入前创建任务卡与 `composition-resolver` /
  `composition-validator`（或按依赖合并）的拆分决策。
