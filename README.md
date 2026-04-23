# Prompt Board

一个类 Maccy 体验的 macOS 提示词管理 MVP。它以轻量悬浮面板运行，支持搜索和管理本地提示词模板、填写 `[变量]` / `[变量|默认值]`，并复制或上屏实时预览中的最终文本。

## 安装

双击仓库根目录的 `install.command`，它会构建 release 版本、安装到：

```text
~/Applications/Prompt Board.app
```

安装完成后会自动启动应用。之后可以直接从 Finder、Spotlight 或 Launchpad 打开 `Prompt Board.app`，运行时不需要再启动命令行客户端。

如果要安装到系统应用目录，可以在终端执行：

```bash
PROMPT_BOARD_INSTALL_DIR=/Applications ./scripts/install_macos.sh
```

更新程序时再次运行 `install.command` 即可。安装脚本只替换 `.app` 应用包，不会删除数据库。

## 数据库

提示词数据库保存在：

```text
~/Library/Application Support/Prompt Board/data.db
```

应用更新、重新安装、替换 `Prompt Board.app` 都不会改动这个目录。只有删除上述 `data.db` 文件或整个 `Prompt Board` 数据目录时，提示词数据才会被清除。

## 开发运行

```bash
cargo run
```

默认快捷键：`Command + Shift + P`。

正式安装后的应用是菜单栏/后台工具形态：不会占用 Dock 位置，启动后默认隐藏浮窗，按快捷键呼出。

按下快捷键后，面板会保持显示；按 `Esc` 隐藏。再次按快捷键只会显示并聚焦面板，不会把它切走。

当焦点切到其他应用或窗口时，面板会自动隐藏。

## 交互

- Panel A/B/C 是三个独立定位的浮层 panel，彼此之间保留间距
- Panel A 固定在窗口最右侧，包含搜索栏和提示词列表，高度为当前屏幕高度的 80%
- Panel B 位于 Panel A 左侧，用于显示当前选中提示词或填写后的实时预览
- Panel B 支持 Markdown 预览
- Panel C 位于 Panel B 左侧，在按 `Enter` 进入变量填写或新建/编辑 prompt 时出现
- 打开面板后默认焦点在搜索栏，可用上下键切换提示词
- 在列表中按 `Enter` 打开 Panel C，填写框默认使用上一次填写内容或 `[变量|默认值]`
- 在 Panel C 中按 `Esc` 或点击返回按钮关闭 Panel C，回到选择 prompt 阶段
- 在 Panel C 中按 `Enter` 切换到下一个填写框，最后一个填写框按 `Enter` 复制 Panel B 当前预览内容并保持页面显示
- 在 Panel C 中按 `Shift + Enter` 可在当前填写框中换行
- Panel C 中未填写的变量会在 Panel B 里继续显示为原始 `[变量名]` 或 `[变量名|默认值]` 占位符
- 在 Panel C 中按 `Command + C` 复制 Panel B 当前预览内容
- `Command + N` 新建提示词，`Command + E` 编辑当前提示词，`Command + Backspace` 删除当前提示词
- 在编辑表单中按 `Command + S` 保存

## MVP 范围

- 使用 SQLite 存储提示词：`~/Library/Application Support/Prompt Board/data.db`
- 首次启动写入中文示例提示词
- 按标题、标签、内容搜索
- 默认按最近使用时间排序
- 使用方向键切换条目
- 从 `[变量名]` 和 `[变量名|默认值]` 中提取模板变量
- 变量表单支持默认值、自定义输入、实时预览
- 提示词 CRUD
- Markdown 预览
- 写入剪贴板
- 类 macOS 原生的浅色半透明三 panel 悬浮窗口
- 自动加载 macOS 系统中文字体，支持中文标题、标签、正文与变量名显示

## 注意

macOS 上，全局快捷键可能需要在系统设置中授予辅助功能/输入监控权限。
