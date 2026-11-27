# 021 — M6-002 Glide Task + Terminal 真实项目试点

- 状态：done
- 目标：用 Glide 的 Task + Terminal 垂直切片复核 v0.1 Schema 的表达能力；不复制源码、不改冻结 Schema。
- 授权：Compose + Modify（试点声明、文档、计划）
- 来源：`E:\Work.Projects\Glide`（只读）
- 变更声明：`changes/021-glide-pilot.yaml`

## 范围

- Glide App composition root；
- Task panel / Terminal panel；
- Project task runner / ConPTY session / WebView2 terminal bridge；
- WPF 结构组合、依赖 provider、端口连接、`glide.toml` 配置。

不包含：Glide 的 Git、窗口系统、托盘、Raw Input、运行时秘密/完整 profile。

## 清单

- [x] 选择 Glide Task + Terminal，并说明选择原因
- [x] 建立只读试点工程 `working/pilots/glide-task-terminal`
- [x] 7 个模块 / 7 个实例
- [x] 6 个结构 Binding（含 root runtime slot）
- [x] 3 个显式 Dependency provider
- [x] 6 个 Port Connection（task 双向、terminal 四向）
- [x] 实例 Config（profile/shell/cwd/rows/cols）
- [x] `composition-cli validate` / `resolve` 通过
- [x] Schema 缺口清单与结论
- [x] 更新 plan / README / docs/ai

## 验收

```text
valid=true
error_count=0
warning_count=0
instances=7
bindings=6
connections=6
dependencies=3
modules=7
source_digest=sha256:310944bf29521179698509a5c20c38d05005c3ae89be584339b7bac00f12ce48
```

- 现有 Core 能表达 Glide 子系统的拓扑；
- 结构、依赖、端口、连接三类关系分别可见，不再把三类关系画成普通边；
- 不能表达：接口/契约独立身份、项目级 Profile/条件选择、运行时/技术/进程元数据、Profile-scoped task 列表；
- 这些问题记录在试点 README，暂不修改 v0.1 Schema。

## 结果

- 试点工程：[`working/pilots/glide-task-terminal`](../pilots/glide-task-terminal)
- 结论：
  - 当前模型足以支撑 Instances/Binding/Connection/Dependency 编辑的拓扑基础；
  - G1/G2/G3 等语义缺口应在后续 UI 需求出现时再评估，不在 021 提前扩展；
  - 022 可先继续 Instance 编辑。
- 局限：
  - 试点是手工映射，不代表 Glide 代码可自动导入；
  - Glide 的 WPF/ConPTY 运行时行为未在本仓库重建；
  - 未重新执行 Glide 的 102 个测试。

## 下一步

022 — Instance 编辑。
