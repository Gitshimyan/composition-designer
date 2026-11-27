# 001–025 未实现工作回溯报告

- 日期：2026-11-27（G03）
- 来源：[未实现功能显性化与授权阻塞治理](./2026-11-27-unimplemented-work-transparency.md) §10
- 登记册：[`working/decisions/unresolved-work.md`](../../working/decisions/unresolved-work.md)
- 性质：对历史任务卡“遗留 / 后续 / 暂缓 / 只读 / 未实现”线索的**回溯记录**
- 状态：G03 完成；不实现任何未决能力

## 1. 方法

1. 逐张读取 001–025 的目标、范围、清单、验收与结果；
2. 提取“遗留 / 后续 / 暂缓 / 只读 / 未实现 / 不在范围”线索；
3. 核对后续任务是否真正实现，不把“后续任务提到过”当作解决证据；
4. 对仍开放的能力分配 `UW-*`，写入登记册；
5. 对无开放能力的历史备注标为 `closed` 或 `superseded`，不推测历史意图；
6. 不改写历史卡原始范围、验收或授权字段。

## 2. 回溯总表

| 卡 | 历史遗留线索（摘要） | 处置 | coverage_status |
|---|---|---|---|
| 001 | 宿主任务只有 selftest 夹具 | closed（当前 host-task 配置已覆盖） | complete |
| 002 | 英文命名可在 M1 微调 | closed（非语义变更） | complete |
| 003 | 大小写不敏感文件系统路径行为未冻结 | UW-027（边界测试/行为） | complete_with_deferred_items |
| 004 | edit-cases / external-change-cases 具体载荷未补 | UW-026 | complete_with_deferred_items |
| 005 | TOML/Core 校验留给 006/007；SchemaUnsupportedFeature 未用 | closed（006/007 完成；预留代码） | complete |
| 006 | Windows 符号链接行为未专门测试；ImDocument 只读 | 只读→011；符号链接→UW-027 | complete_with_deferred_items |
| 007 | 级联抑制/重复 ID SourceMap 边界未专门测试；resolver 未拆 | 测试→UW-027；拆分非能力缺口 | complete_with_deferred_items |
| 008 | CLI 只读；`--json` 兼容参数；人类输出无 digest | CLI 编辑→016；其余非阻塞 | complete |
| 009 | 无 TOML/Diff/幂等；无撤销/事务历史 | 010/011 完成；撤销→UW-014；历史持久化→UW-015 | complete_with_deferred_items |
| 010 | 历史只在内存；Diff 不含依赖/模块；无撤销 | UW-015、UW-025、UW-014 | complete_with_deferred_items |
| 011 | 历史在内存、无 fsync、单入口、无撤销/外部同步 | UW-015、UW-019、UW-018、UW-014、UW-017 | complete_with_deferred_items |
| 012 | Canvas/Source 占位；无 UI 测试；窗口/安装包未完善 | 013/014/020 完成；窗口/安装包→UW-024 | complete_with_deferred_items |
| 013 | 无 UI 测试；Connection 未画；布局无 fsync；窗口未自适应 | 020 完成；Connection→024 独立视图（superseded）；UW-019、UW-024 | complete_with_deferred_items |
| 014 | Source 只读；array 只读；无多命令 Review；无 Binding/Connect UI | Source→UW-011；array→UW-016；Review→UW-025；编辑 UI→022–025 完成 | complete_with_deferred_items |
| 015 | 2s 轮询；无 watcher/三向合并；Reload 丢 Review；布局未合并 | UW-017 | complete_with_deferred_items |
| 016 | JSONL 无持久幂等；Source 只读；单入口；无 MCP | UW-015、UW-011、UW-018；MCP→017 | complete_with_deferred_items |
| 017 | 无 subscriptions/progress/cancel/auth/多项目 | UW-020 | complete_with_deferred_items |
| 018 | 无 LLM Skill 评测；配置仅 Windows；单项目；无主动外部通知；Source 只读 | UW-021、UW-020、UW-011 | complete_with_deferred_items |
| 019 | 无 fsync；Source 只读；历史在内存；单入口；MCP 限制；无 UI 测试 | UW-019、UW-011、UW-015、UW-018、UW-020；UI 测试→020 | complete_with_deferred_items |
| 020 | IPC fixture 不是真实 Tauri；无真实 WebView2 自动化 | UW-012（deferred） | complete_with_deferred_items |
| 021 | G1 接口身份、G2 runtime 容器、G3 Profile；试点手工映射/未跑 Glide 测试 | UW-002/003/004；试点范围外记录 | complete_with_deferred_items |
| 022 | 必填 Slot 子树→023；模块库/版本下载；Provider 重绑定→024/025；float wire；fixture | UW-022、UW-005、UW-012；其余已解决 | complete_with_deferred_items |
| 023 | Move child 固定；required many 固定一个子实例；MAX_DEPTH；Detach 模型语义 | UW-023；Detach 为模型语义，不单列 | complete_with_deferred_items |
| 024 | Provider→025；Connection View 非图形 Port Graph；pan/zoom 不保留；完整 Review→026 | UW-025；Port Graph 为 out_of_scope/后续 | complete_with_deferred_items |
| 025 | G1–G3、float、Source、CI 推送 | 既有 UW；CI 已完成；Source→UW-011 | complete_with_deferred_items |

## 3. 新增 UW 说明

完整字段见 [`working/decisions/unresolved-work.md`](../../working/decisions/unresolved-work.md)。

- **UW-014 撤销/重做**：9/10/11 均提到没有撤销栈；属产品能力，deferred。
- **UW-015 Revision/request 跨进程持久化**：9/10/11/16/19 的进程内历史；deferred。
- **UW-016 Inspector array 属性编辑**：014 起 array 只读；Modify 级 UI 能力，pending_decision。
- **UW-017 外部修改 watcher / 三向合并 / 草稿保留 / 布局合并**：015 遗留；M7 方向，deferred。
- **UW-018 多 composition 入口 / 多文件**：011/016/019；M9 方向，deferred。
- **UW-019 fsync 级崩溃恢复**：011/013/019；deferred。
- **UW-020 MCP subscriptions/progress/cancel/auth/multi-project**：017/018/019；deferred。
- **UW-021 Agent Skill LLM 评测 / 跨平台配置生成**：018；deferred。
- **UW-022 模块库/版本发现与下载**：022；M9 方向，deferred。
- **UW-023 Binding 向导 UX 限制**：child 固定、required many 只建一个、MAX_DEPTH；pending_decision。
- **UW-024 窗口自适应 / bundle·installer / 大 bundle 拆分**：012/013/014；deferred。
- **UW-025 Diff/Review 覆盖 Dependency 与模块影响**：010/014/024；026 承接，accepted。
- **UW-026 edit-cases / external-change-cases 具体载荷**：004；Compose+Modify，pending_decision。
- **UW-027 边界测试补充**：006 符号链接、007 级联/重复 SourceMap、003 大小写路径；Modify，pending_decision。

## 4. 已关闭 / 已解决线索

- 001 宿主任务：当前 DSH host-task 配置已覆盖构建/测试；
- 002 术语英文命名：未发生语义变更，不需要能力补齐；
- 005 TOML/Core 校验：006/007 已完成；
- 008 CLI 编辑：016 增加 query/apply；只读 CLI 的 M2 缺口已解决；
- 009/010/011 的 TOML 落盘、Diff、幂等、source_digest：010/011 已完成；
- 012/013/014 的 Canvas/Source/Binding/Connection UI：013/014/022–025 已完成；
- 015/016/018/019/020 的 UI 自动化：020 已完成并远端通过；
- 021 的 G1–G3 转入 UW-002/003/004；
- 022 的 required Slot 子树：023 已完成；
- 024 的 Provider 编辑：025 已完成；
- 025 的 CI 远端执行：已在 run 45984567517 完成。

## 5. 结论

- 001–025 的历史遗留线索均已有去向：关闭、既有 UW 或新增 UW；
- 新增 14 个 UW（UW-014–UW-027），未实现能力不被静默消失；
- G1–G3、ConfigValue wire、Source、机器门禁等仍为等待决策/延期项；
- 本报告不改写任何历史任务卡的范围、验收或授权字段。
