# Prompt Board 产品与技术设计文档

本文档针对类 Maccy 体验的 macOS 提示词管理工具，梳理产品交互形态、功能模块以及纯 Rust 技术栈的实施方案。

## 1. 产品形态与核心理念

- **无主窗口模式**：应用作为常驻后台的菜单栏工具运行，不在 Dock 中显示图标。
- **悬浮呼出**：用户按下全局快捷键后，以轻量级悬浮窗形式唤出。
- **用完即走**：完成复制或模拟粘贴操作后，悬浮窗自动隐藏，焦点返回用户之前正在使用的软件中。

## 2. 界面布局与视觉交互

### 2.1 从右至左的面板布局

界面采用自右向左展开的面板结构，贴合 macOS 屏幕右侧边缘，减少对屏幕中央主工作区的遮挡。

- **Panel A（主控与列表）**：固定在屏幕最右侧，包含顶部搜索栏、提示词列表和管理入口。
- **Panel B（预览与详情）**：位于 Panel A 左侧，渲染当前提示词的完整内容。
- **Panel C（变量填写/编辑表单）**：位于 Panel B 左侧，仅在用户按下 Enter 或进入新建/编辑时显示。

### 2.2 视觉与排版规范

- **适度放大字体**：搜索栏约 20px，正文约 16-18px，保证高分辨率屏幕清晰易读。
- **Markdown 渲染**：Panel B 支持 Markdown 格式解析，能渲染粗体、列表和代码块。
- **自适应与主题**：界面采用圆角、半透明和轻微描边的 macOS 原生质感，后续可扩展浅色/深色模式适配。

## 3. 功能模块规划

### 3.1 提示词检索与排序

- **全局搜索**：支持输入关键字对标题、标签和正文实时模糊匹配。
- **最近使用排序**：列表默认按 `last_used_at` 降序排序。用户刚用过的提示词保持在更靠前的位置。

### 3.2 提示词管理

- **新建提示词**：在 Panel A 提供新建按钮和 `Command + N` 快捷键，允许录入标题、内容和标签。
- **编辑与删除**：选中某条提示词后，可通过 `Command + E` 编辑，`Command + Backspace` 删除。

### 3.3 变量提取与填写流

- **变量识别**：自动从提示词内容中提取 `[变量名]` 或 `[变量名|默认值]` 占位符。
- **顺畅表单**：Panel C 列出变量输入框，支持 Tab / Enter 快速切换。
- **实时预览**：填写过程中，Panel B 的 Markdown 预览实时显示替换后的结果；未填写变量保留原始占位符。
- **上屏操作**：填写完成后，按 `Command + C` 复制到剪贴板；在最后一个填写框按 Enter 可模拟粘贴到当前活跃文本编辑器。

### 3.4 扩展功能

- **快捷键自定义**：提供设置页面，允许用户更改默认唤出快捷键。
- **数据导入导出**：支持将提示词库导出为 JSON 或 Markdown 文件。

## 4. 数据库设计

采用 SQLite 作为本地轻量级存储方案。基于最近使用排序，表结构如下：

```sql
CREATE TABLE IF NOT EXISTS prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    tags TEXT,
    last_used_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

每次用户完成复制或粘贴操作时，更新该条目的 `last_used_at` 为当前时间。

## 5. 技术落地选型

- **GUI 框架**：egui + eframe，配置无边框与透明窗口。
- **Markdown 解析**：egui_commonmark。
- **全局热键**：global-hotkey。
- **剪贴板与模拟输入**：arboard 写剪贴板，enigo 模拟 `Command + V`。
- **数据操作**：rusqlite。
