# AGENT.md

本文档面向后续维护本项目的 Codex/Agent/开发者，说明 `prompt_board` 的本地验证、CI/CD 设计、GitHub Actions 配置建议与发布流程。

目标仓库：

```text
https://github.com/JDLu2003/prompt_board.git
```

## 项目概览

`prompt_board` 是一个 macOS 原生风格的提示词管理工具 MVP，使用 Rust + egui/eframe 构建。

关键能力：

- 全局快捷键：`Command + Shift + P`
- 三个独立浮层 panel：Panel A 搜索列表，Panel B 预览，Panel C 变量填写
- SQLite 本地存储：`~/Library/Application Support/Prompt Board/data.db`
- 中文字体加载与中文示例数据
- 模板变量格式：`[变量名]` 和 `[变量名|默认值]`
- 填写时未输入的变量保留原始占位符
- Markdown 预览
- 提示词 CRUD
- `Command + C` 复制 Panel B 当前预览内容

主要文件：

```text
Cargo.toml
Cargo.lock
src/main.rs
src/app.rs
src/db.rs
src/system.rs
src/template.rs
scripts/bundle_macos.sh
packaging/macos/Info.plist
README.md
DESIGN.md
```

## 本地开发指令

在项目根目录运行：

```bash
cargo fmt
cargo check
cargo test
cargo run
```

打包 macOS `.app`：

```bash
./scripts/bundle_macos.sh
```

生成结果：

```text
target/release/Prompt Board.app
```

注意：

- `cargo run` 会启动 GUI 应用，CI 中不要运行。
- macOS 上全局快捷键、剪贴板、窗口焦点相关行为可能需要系统权限，CI 只能覆盖构建与单元测试，无法完整验证桌面权限行为。
- 终端中可能出现 macOS 输入法或系统服务日志，例如 `Connection invalid`，这类日志不等同于应用崩溃。

## 推荐分支策略

建议使用轻量 GitHub Flow：

```text
main       稳定分支，可发布
feature/* 具体功能开发
fix/*     bug 修复
release/* 发布准备，可选
```

提交前本地必须通过：

```bash
cargo fmt -- --check
cargo check --locked
cargo test --locked
```

如果本地目录还不是 git 仓库，可初始化并关联远端：

```bash
git init
git remote add origin https://github.com/JDLu2003/prompt_board.git
git add .
git commit -m "Initial prompt board MVP"
git branch -M main
git push -u origin main
```

如果远端已有历史，先拉取或用合适方式合并，避免覆盖远端内容。

## CI 目标

每次 push 或 pull request 应验证：

- 代码格式
- Rust 编译
- 单元测试
- macOS release 构建
- macOS `.app` 打包脚本语法与产物生成

建议 CI 先做无签名构建。签名、公证和正式发布放到 release 工作流中。

## GitHub Actions: CI

建议创建：

```text
.github/workflows/ci.yml
```

推荐内容：

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  rust:
    name: Rust checks
    runs-on: macos-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Check
        run: cargo check --locked

      - name: Test
        run: cargo test --locked

      - name: Validate macOS bundle script
        run: bash -n scripts/bundle_macos.sh

      - name: Build release app bundle
        run: ./scripts/bundle_macos.sh

      - name: Upload unsigned app artifact
        uses: actions/upload-artifact@v4
        with:
          name: Prompt-Board-unsigned-app
          path: target/release/Prompt Board.app
          if-no-files-found: error
```

说明：

- 使用 `macos-latest`，因为项目是 macOS GUI 工具，并且依赖全局热键、AppKit 相关链路。
- `cargo check --locked` 和 `cargo test --locked` 保证使用仓库内 `Cargo.lock`。
- CI 构建的是未签名 `.app`，可用于基本验证，不建议直接面向普通用户分发。

## Release/CD 目标

当推送 tag 时自动：

- 构建 release 二进制
- 生成 `.app`
- 压缩为 `.zip`
- 可选：签名 `.app`
- 可选：notarize 公证
- 创建 GitHub Release
- 上传产物

推荐 tag 格式：

```text
v0.1.0
v0.1.1
v0.2.0
```

## GitHub Actions: Release

建议创建：

```text
.github/workflows/release.yml
```

无签名基础版本：

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  build-macos:
    name: Build macOS release
    runs-on: macos-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Test
        run: cargo test --locked

      - name: Build app bundle
        run: ./scripts/bundle_macos.sh

      - name: Zip app
        run: |
          cd target/release
          ditto -c -k --sequesterRsrc --keepParent "Prompt Board.app" "Prompt-Board-${GITHUB_REF_NAME}-macos-unsigned.zip"

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: target/release/Prompt-Board-${{ github.ref_name }}-macos-unsigned.zip
          generate_release_notes: true
```

## 签名与公证

正式分发给 macOS 用户前，建议加入签名和 notarization。

需要的 GitHub Secrets：

```text
APPLE_DEVELOPER_ID_CERTIFICATE_BASE64
APPLE_DEVELOPER_ID_CERTIFICATE_PASSWORD
APPLE_TEAM_ID
APPLE_ID
APPLE_APP_SPECIFIC_PASSWORD
```

推荐证书类型：

```text
Developer ID Application
```

签名流程概要：

```bash
codesign --force --deep --options runtime \
  --sign "Developer ID Application: <Name> (<TEAM_ID>)" \
  "target/release/Prompt Board.app"
```

公证流程概要：

```bash
xcrun notarytool submit "Prompt-Board.zip" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_APP_SPECIFIC_PASSWORD" \
  --wait

xcrun stapler staple "target/release/Prompt Board.app"
```

注意：

- 没有开发者证书时，CI 仍应保留未签名 release 产物用于内部测试。
- 签名与公证步骤应只在 tag release 或手动 workflow dispatch 中执行，不建议在每个 PR 中执行。

## 推荐发布步骤

1. 确认本地验证通过：

```bash
cargo fmt -- --check
cargo check --locked
cargo test --locked
./scripts/bundle_macos.sh
```

2. 更新版本号：

```text
Cargo.toml package.version
packaging/macos/Info.plist CFBundleShortVersionString
packaging/macos/Info.plist CFBundleVersion
README.md 如有必要
```

3. 提交版本变更：

```bash
git add Cargo.toml Cargo.lock packaging/macos/Info.plist README.md
git commit -m "Release v0.1.0"
```

4. 打 tag 并推送：

```bash
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

5. 等待 GitHub Actions 生成 release 产物。

6. 下载 artifact 或 release zip，在本机验证：

```bash
open "Prompt Board.app"
```

验证项目：

- `Command + Shift + P` 能唤出
- Panel A 固定在最右侧，高度约为屏幕 80%
- 上下键能切换 prompt
- `Enter` 能打开 Panel C
- `Esc` 能从 Panel C 返回选择阶段
- 未填写变量仍显示 `[变量名]`
- `Command + C` 能复制 Panel B 当前内容
- 焦点切到其他应用后窗口自动隐藏

## Branch Protection 建议

在 GitHub 仓库 `Settings -> Branches` 中为 `main` 配置：

- Require a pull request before merging
- Require status checks to pass before merging
- Required checks:
  - `Rust checks`
- Require branches to be up to date before merging
- Do not allow force pushes
- Do not allow deletions

## 依赖更新策略

建议每周或每两周检查依赖：

```bash
cargo update
cargo check
cargo test
```

重点关注：

- `eframe`
- `global-hotkey`
- `arboard`
- `rusqlite`

macOS GUI 依赖链容易受 `winit`、`objc2`、`AppKit` 相关版本影响。升级 GUI 相关依赖后必须实际运行 `cargo run` 做启动验证。

## Agent 工作约定

后续 agent 修改项目时应遵守：

- 优先保持 Rust/egui 单体结构，不引入 WebView 或跨语言运行时。
- 修改 UI 状态流时，同步检查键盘交互：
  - `Command + Shift + P`
  - `ArrowUp` / `ArrowDown`
  - `Enter`
  - `Esc`
  - `Command + C`
  - `Command + N`
  - `Command + E`
  - `Command + S`
  - `Command + Backspace`
- 修改模板逻辑时，必须更新 `src/template.rs` 单元测试。
- 不要把 `target/` 加入版本管理。
- 不要在 CI 中运行 GUI 交互测试，除非后续引入专门的 macOS UI 自动化。
- 发布前必须确认 `scripts/bundle_macos.sh` 仍能生成 `.app`。

## 当前最小验收命令

任何合并前至少运行：

```bash
cargo fmt -- --check
cargo check --locked
cargo test --locked
bash -n scripts/bundle_macos.sh
```

如果改动涉及窗口、热键、剪贴板、中文字体或 panel 布局，还必须手动运行：

```bash
cargo run
```

并完成 README 中的核心交互验证。
