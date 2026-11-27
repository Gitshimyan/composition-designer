# 008 — M1 CLI 与确定性

- 状态：done
- 目标：新建 `composition-cli`，提供 `inspect` / `resolve` / `validate` 三个命令，输出稳定 JSON
  与人类可读诊断、定义退出码，并用 golden 测试锁定输出；完成 M1 收口。
- 授权：Extend（新 crate、serde 序列化依赖）
- 变更声明：`changes/008-composition-cli.yaml`

- coverage_status：complete
- open_items：无
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

007 已有完整 Core，但只能作为 Rust API 使用。008 要给人类与脚本一个稳定入口：同一项目目录
必须产生可复现的 JSON、明确退出码和可定位诊断；不能读 GUI 状态，也不能写 TOML。

CLI 设计：

- `composition-cli inspect <dir>`：项目 + 模块契约 + 诊断 + `source_digest`；
- `composition-cli resolve <dir>`：完整规范化 `CompositionModel` + 诊断 + `source_digest`；
- `composition-cli validate <dir> [--json]`：校验摘要 + 诊断；默认人类可读，`--json` 机器可读；
- 退出码：`0` 无 error，`1` 有 error，`2` 用法/内部错误；warning 不导致失败。

## 清单

- [x] 根 workspace 加入 `crates/composition-cli`
- [x] 领域类型实现 serde 序列化（ID/版本/契约/模型/诊断/Revision）
- [x] 手写参数解析：三个命令、`--json`、`--help`、`--version`、路径与未知参数
- [x] inspect / resolve / validate 的 JSON 输出
- [x] validate 的人类可读输出与退出码
- [x] 输出稳定：模型/契约/诊断顺序固定，不包含绝对路径
- [x] 集成测试：三命令、exit code、entry-missing、usage、JSON 解析
- [x] golden 测试：validate minimal、validate connections、inspect minimal、resolve minimal
- [x] `cargo test` / `clippy` / `fmt` 全绿；更新 plan、crates/README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**101 passed / 0 failed**
  （domain 58 + toml 单元 12 + toml 集成 15 + core 8 + cli 8）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check`：通过；
- `validate examples/minimal-project --json` exit 0，`valid=true`、诊断为空、digest 与 006 一致；
- `validate invalid-projects/connections/errors --json` exit 1，三个连接错误代码与
  `expected.json` 一致；
- `inspect` 输出项目 id、2 个模块、source_digest；`resolve` 输出 2 个实例、1 个 Binding；
- `validate invalid entry-missing` exit 1 且给出 `file-project-missing`；
- 无参数 exit 2 并打印 usage；
- 4 个 golden 文件逐字节稳定；相同输入重复运行输出相同；
- CLI 不写项目文件、不依赖 GUI/Agent。

## 结果 / 遗留问题

- 结果：新增 `crates/composition-cli`：
  - `inspect` / `resolve` / `validate` 三个命令，JSON 输出与人类可读 validate；
  - 退出码 0/1/2；
  - `tests/golden/` 下 4 个 golden 文件：validate-minimal、validate-connections-errors、
    inspect-minimal、resolve-minimal；
  - 8 个 CLI 集成测试，包含 exit code、JSON 解析、usage 与 golden 比对。
- 序列化：`composition-domain` 增加 `serde` 依赖并实现 Serialize：
  - ID/版本/枚举/诊断/Revision 使用稳定字符串；
  - `ConfigValue` 手工实现为原生 JSON 值；
  - `Contract` 手工实现为按 id 排序的数组；
  - 不输出绝对路径。
- M1 收口：005 domain + 006 loader + 007 Core + 008 CLI；workspace 共 101 项测试。
- 遗留：
  - CLI 是只读的；编辑命令/事务/Revision 冲突属于 M2；
  - 没有 GUI/Agent Adapter；`composition-editor` / `protocol` / `app` 仍未建立；
  - `--json` 对 inspect/resolve 是允许但无作用的兼容参数；help 只作为顶层命令处理；
  - 人类可读输出不含 digest；需要时用 `--json`。
- 下一步：M2 009（命令 Schema 与事务），进入前创建任务卡与编辑器 crate 决策。
