# Designer Layout v1（M3 013）

- 状态：M3 013 冻结（2026-11-26）
- 文件：`.designer/layout.toml`
- 范围：Canvas 节点位置；表现层数据，**不属于** Composition Model

## 1. 格式

```toml
[nodes.app]
x = 0.0
y = 0.0

[nodes.hello]
x = 120.0
y = 190.0
```

- `nodes` 是表，键是实例 ID（`LocalId` 语法）。
- `x` / `y` 是 React Flow 画布坐标（CSS px），可为浮点。
- 文件不存在时使用自动布局。
- 读取时忽略未知/非法实例 ID 与缺少 `x`/`y` 的条目。
- 保存会替换整个文件；v1 不保留未知字段。

## 2. 语义边界

- `.designer/**` 不由 Core 读取，不参与解析、校验、Revision 或 `source_digest`。
- 移动节点不修改 `composition/**`、模块定义、Binding 或 Connection。
- 布局保存失败只影响界面位置持久化，不阻塞项目读取。
- 写入没有 fsync/崩溃恢复；损坏时回退到自动布局。

## 3. 命令

| Tauri command | 输入 | 输出 |
|---|---|---|
| `load_layout` | `root` | `{ [instanceId]: { x, y } }`；缺失为空对象 |
| `save_layout` | `root`, `nodes` | `()`；创建 `.designer/` 并写 `layout.toml` |

`root` 必须是已打开项目目录；布局文件固定为 `<root>/.designer/layout.toml`。
