---
name: release
description: |
  发布 terminal-serial 新版本。当用户说"发布"、"release"、"打tag"、"新版本"、"发版"时使用此技能。
  覆盖场景：修改版本号、提交推送、创建 tag 触发 GitHub Actions 自动构建。
---

# Release 技能

自动化 terminal-serial 的版本发布流程。最终通过推送 `v*` tag 触发 GitHub Actions 进行多平台构建（macOS aarch64/x86_64、Linux x86_64、Windows x86_64），并生成 draft release。

## 前置检查

发布前必须依次完成以下检查，任何一项不通过都需要用户确认后再继续。

### 1. 分支检查

```bash
git branch --show-current
```

必须是 `master` 分支。如果不是，提醒用户先切换：

> 当前不在 master 分支（当前：xxx），请先切换到 master 分支后再发布。

### 2. 工作区状态检查

```bash
git status --porcelain
```

工作区必须干净（无未提交的修改）。如果有未提交的更改，提醒用户先处理：

> 工作区有未提交的更改，请先 commit 或 stash 后再发布。

### 3. 远程同步检查

```bash
git fetch origin
git log origin/master..HEAD --oneline
```

确保本地 master 不落后于远程。如果有未推送的 commit，需要先推送。也检查远程是否有本地没有的 commit：

```bash
git log HEAD..origin/master --oneline
```

如果远程有更新，提醒用户先 pull。

## 版本号确认

使用 AskUserQuestion 询问用户要发布的版本号。询问前先收集上下文信息：

```bash
# 当前 Cargo.toml 中的版本
head -5 Cargo.toml | grep version

# 已有的 tag
git tag --sort=-v:refname | head -10
```

将当前版本和最近的 tag 展示给用户作为参考。版本号格式为 `x.y.z`（语义化版本）。

## 执行发布

以下步骤按顺序执行，每步成功后再继续下一步。

### Step 1: 更新 Cargo.toml 版本号

将 `Cargo.toml` 中 `[package]` 下的 `version` 字段更新为用户指定的版本号。使用 Edit 工具精确修改，不要影响文件其他内容。

### Step 2: 提交版本变更

```bash
git add Cargo.toml
git commit -m "release: bump version to <version>"
```

### Step 3: 推送到远程

```bash
git push origin master
```

### Step 4: 创建并推送 tag

```bash
git tag v<version>
git push origin v<version>
```

tag 推送后会自动触发 GitHub Actions release workflow，在 macOS、Linux、Windows 三个平台上构建二进制文件，并创建 draft release。

### Step 5: 等待构建完成并更新 Release Notes

等待 GitHub Actions 构建完成。可以用以下命令轮询状态：

```bash
gh run list --workflow=release.yml --limit=1
```

构建完成后（run 状态变为 `completed`），自动生成 release notes 并更新到 draft release。

**生成 release notes：** 基于 git log 自动提取变更内容：

```bash
# 获取上一个版本的 tag
git describe --tags --abbrev=0 HEAD^

# 提取两个版本之间的 commit 记录
git log <上一个tag>..v<version> --oneline --no-decorate
```

将 commit 记录整理分类为 release notes，包含以下段落（按实际情况取舍）：

- **新功能**：feat 相关的 commit
- **Bug 修复**：fix 相关的 commit
- **改进**：其他改进类 commit
- **其他变更**：无法归类的 commit

**更新 draft release：**

```bash
gh release edit v<version> --draft=false --notes "<release notes 内容>"
```

这会将 draft release 更新为正式发布状态。如果想保持 draft 让用户手动确认，则使用：

```bash
gh release edit v<version> --notes "<release notes 内容>"
```

使用 AskUserQuestion 询问用户是否直接发布（`--draft=false`）还是保持 draft 状态稍后手动发布。

## 完成后

告诉用户：

- 版本号已更新并提交
- tag `v<version>` 已推送
- Release notes 已更新
- 如果选择了直接发布，则 release 已正式发布；否则提醒用户到 GitHub Releases 页面手动发布
