# Device Viewer 仓库上传与设置指引

**适用对象：** Device Viewer 仓库维护者  
**适用平台：** Windows / Linux / macOS  
**仓库类型：** 源码可见的专有许可项目，**不是开源项目**

> **法律前提**
>
> Device Viewer 的完整著作权归原作者独有。本项目采用定制化专有许可，不是 MIT、Apache-2.0、GPL、BSD、MPL 或任何 OSI 认可的开源许可证。
>
> 个人、非营利组织可免费本地使用、学习研究。所有盈利组织、商业团队、企业、工作室、付费开发者，未经作者书面授权，禁止使用、复制、修改、衍生、二次开源、二次闭源、二次打包、二次分发、二次发布、商用售卖、集成进商业项目、SaaS、付费工具、课程售卖。商业授权邮箱：**yanruoxinsu2@outlook.com**。
>
> 任何修改源代码的行为，必须在本仓库提交 **Modify Source Report** Issue 报备；未报备视为侵权。完整条款见 [`LICENSE`](LICENSE)、[`LEGAL_STATEMENT.md`](LEGAL_STATEMENT.md)。

---

## 1. 上传前的最终检查

### 1.1 必须存在的文件

在 `D:\device viwer`（或你的项目根目录）确认以下文件存在：

```text
Cargo.toml
Cargo.lock                  # 应用项目必须提交，用于可复现构建
build.rs
.gitignore
README.md
LICENSE
LEGAL_STATEMENT.md
PRIVACY.md
DISCLAIMER.md
REPOSITORY_GUIDE.md
.github/
  CONTRIBUTING.md
  ISSUE_TEMPLATE/
    bug_report.md
    feature_request.md
    modify_source.md
  workflows/
    release.yml
packaging/
  README_PACKAGE.md
  release_notes_template.md
src/
  main.rs
  app.rs
  i18n.rs
  state.rs
  log.rs
  export.rs
  device_query/
  ui/
tests/
  device_query.rs
  smoke.rs
```

> `Cargo.lock` 不应被 `.gitignore` 忽略。Device Viewer 是应用程序而不是库，发布构建必须锁定依赖版本。

### 1.2 检查 Git 忽略规则

```powershell
cd "D:\device viwer"
git status --short --ignored
```

确认以下内容**不会**被提交：

- `target/`
- `dist/`、`artifacts/`、`release/`
- `device-viewer.json`
- `device-viewer.log`
- `snapshots/`
- `*.dvsnap`
- `.env`、`*.key`、`*.pem`、`*.p12`、`*.pfx`
- `.idea/`、`.vscode/`
- 任何本地测试快照、日志、临时文件

如果误把敏感文件加入了暂存区：

```powershell
git restore --staged <path>
```

如果敏感文件已经进入历史提交，仅删除文件不够；必须重写历史并立即轮换泄露的密钥。**不要在公开仓库中留下任何真实令牌、证书、私钥或 Apple 凭据。**

### 1.3 检查法律文档

确认五个文件均为最新版本，且没有互相冲突：

```powershell
Get-Item LICENSE, LEGAL_STATEMENT.md, PRIVACY.md, DISCLAIMER.md, README.md |
  Select-Object Name, Length, LastWriteTime
```

确认 `LICENSE` 中至少包含：

- 著作权归原作者独有；
- 非 OSI 开源许可声明；
- 个人/非营利免费本地使用；
- 盈利组织未经书面授权禁止使用、修改、分发、商用；
- 修改必须提交 Issue 报备；
- 禁止二次开源、二次闭源、二次打包、二次发布；
- 侵权追责与管辖法律。

### 1.4 检查构建与测试

```powershell
$env:CARGO_HOME = "$env:TEMP\cargo-home-probe"
$env:CARGO_TARGET_DIR = "$env:TEMP\device-viewer-target"

cargo generate-lockfile
cargo fmt --all -- --check
cargo check --tests
cargo test
```

如本机配置了可用的 Cargo 镜像，可以省略 `CARGO_HOME` 覆盖；但不要提交本机的 `.cargo/config.toml` 中的私有镜像或凭据。

### 1.5 检查工作流

```powershell
Get-Content .github/workflows/release.yml -TotalCount 40
```

确认：

- 触发条件为 `v*` 标签；
- 三个平台 job 均存在；
- 每个打包步骤都复制了四个法律文档；
- 没有写死任何 token、密码、证书、私钥；
- `publish` job 使用 `${{ github.token }}`，不是个人 PAT；
- 产物命名为 `DeviceViewer-<platform>-<arch>-<version>`。

### 1.6 检查 Issue 模板

确认三个模板均存在：

```powershell
Get-ChildItem .github/ISSUE_TEMPLATE
```

每个模板都必须包含：

- 著作权归属；
- 盈利组织禁止私自商用/二次分发；
- 修改必须 Issue 报备；
- 侵权追责条款；
- 非 OSI 开源声明。

### 1.7 检查敏感信息

不要依赖 `.gitignore` 代替人工审查。至少执行一次全文关键词扫描：

```powershell
git grep -n -I -E "(ghp_|github_pat_|AKIA|BEGIN (RSA|OPENSSH|EC|PRIVATE) PRIVATE KEY|password\s*=|secret\s*=|api[_-]?key\s*=)" -- .
```

如命中，先判断是否为示例占位符；真实凭据必须立即移除并轮换。

---

## 2. 本地初始化 Git 仓库

### 2.1 安装并确认 Git

```powershell
git --version
```

建议 Git 2.28 或更高版本，以支持 `git init -b main`。如版本较旧：

```powershell
git init
git branch -M main
```

### 2.2 初始化仓库

```powershell
cd "D:\device viwer"
git init -b main
```

### 2.3 设置提交身份

只在本仓库设置，不要使用来源不明的全局身份：

```powershell
git config user.name "Device Viewer Maintainer"
git config user.email "your-public-email@example.com"
```

> 建议使用可公开的业务邮箱，不要使用包含手机号、身份证号或私人信息的邮箱。该邮箱会出现在公开提交记录中。

### 2.4 暂存全部文件

```powershell
git add .
```

### 2.5 提交前核对

```powershell
git status --short
git diff --cached --stat
```

重点确认：

- 没有 `target/`；
- 没有 `Cargo.lock` 之外的构建缓存；
- 没有 `.env`、密钥、证书；
- 没有 `device-viewer.json`、日志、快照；
- 没有 `dist/`、`artifacts/`；
- 法律文档、Issue 模板、工作流均已被暂存。

如发现误加：

```powershell
git restore --staged <path>
```

### 2.6 创建初始提交

```powershell
git commit -m "chore: initial Device Viewer repository"
```

如需拆分提交，建议至少保持以下边界：

1. `chore: add project metadata and license`
2. `feat: add hardware query layer`
3. `feat: add GUI and CLI`
4. `docs: add legal and repository documentation`
5. `ci: add release workflow and packaging`

不要在初始提交中混入构建产物或本地配置。

---

## 3. 创建 GitHub 仓库

### 3.1 网页创建

1. 登录 GitHub；
2. 右上角 **+** → **New repository**；
3. **Repository name** 填 `device-viewer`；
4. **Description** 建议：

   ```text
   Cross-platform hardware information viewer (source-visible proprietary license).
   ```

5. 选择 **Private** 或 **Public**：
   - 准备阶段建议 **Private**，避免未完成的法律/发布材料提前公开；
   - 确认无误后再转为 Public；
   - 无论 Public 还是 Private，`LICENSE` 的专有许可均适用。
6. **不要**勾选：
   - Add a README file；
   - Add .gitignore；
   - Choose a license。
7. 点击 **Create repository**。

> 如果勾选 README、.gitignore 或 license，GitHub 会创建初始提交，导致本地首次推送出现 unrelated histories。保持空仓库最省事。

### 3.2 使用 GitHub CLI 创建（可选）

如已安装并登录 `gh`：

```powershell
gh auth status
gh repo create device-viewer --private --source=. --remote=origin --push
```

确认 `--source=.` 指向项目根目录。若仓库已存在远端，不要重复执行。

---

## 4. 关联远端并推送

### 4.1 添加远端

HTTPS：

```powershell
git remote add origin https://github.com/<owner>/device-viewer.git
```

SSH：

```powershell
git remote add origin git@github.com:<owner>/device-viewer.git
```

检查：

```powershell
git remote -v
```

### 4.2 确认分支名

```powershell
git branch -M main
```

### 4.3 首次推送

```powershell
git push -u origin main
```

### 4.4 认证注意事项

- HTTPS 推送使用 Personal Access Token 或 Git Credential Manager；
- SSH 推送使用本机 SSH key；
- **不要把 PAT、SSH 私钥、证书写入仓库、脚本、README 或 Issue**；
- 如 token 曾被提交，立即在 GitHub 设置中撤销并重新生成。

### 4.5 验证上传结果

在 GitHub 网页确认：

- 根目录存在 `LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`、`README.md`、`REPOSITORY_GUIDE.md`；
- `.github/workflows/release.yml` 存在；
- `.github/ISSUE_TEMPLATE/` 下三个模板存在；
- `.github/CONTRIBUTING.md` 存在；
- `src/`、`tests/`、`packaging/` 结构完整；
- `Cargo.lock` 已提交；
- 没有任何 `target/`、日志、快照或密钥。

---

## 5. 仓库网页设置

> GitHub 设置项名称会随界面语言和版本变化。以下以英文界面为主，中文界面按含义对应。

### 5.1 General（常规）

进入 **Settings → General**。

#### 5.1.1 基本信息

- **Description**：

  ```text
  Cross-platform hardware information viewer for Windows, Linux and macOS. Source-visible proprietary license; commercial use requires written permission.
  ```

- **Website**：可留空，或填写发布页/文档页；
- **Topics** 建议：

  ```text
  rust, egui, eframe, hardware, system-information, vulkan, opencl, cross-platform, desktop-app
  ```

- **Social preview**：可选，上传项目截图；不要使用来源不明的图片。

#### 5.1.2 Default branch

保持 `main`。不要在未迁移的情况下随意改名。

#### 5.1.3 Features

- **Issues**：**开启**。这是 Modify Source 报备、Bug 反馈、功能建议的唯一入口。
- **Discussions**：**建议关闭**。
  - 如开启，必须新建并置顶“法律声明”帖子，说明本项目不是开源项目、禁止商用与二次发布；
  - Discussions 中的内容同样受 `LICENSE` 约束；
  - 如无专人维护，关闭可减少侵权与误导风险。
- **Projects**：可选。若使用，建议设为私有或仅维护者可见。
- **Wiki**：**建议关闭**。避免出现过期文档、非官方安装包或与 `LICENSE` 冲突的内容。
- **Sponsorships**：**关闭**，除非你已准备好处理商业赞助与授权边界。
- **Preserve this repository**：不需要为普通项目开启。

#### 5.1.4 Pull Requests

- **Allow merge commits**：可关闭；
- **Allow squash merging**：建议开启；
- **Allow rebase merging**：可关闭；
- **Always suggest updating pull request branches**：可关闭；
- **Allow auto-merge**：**必须关闭**。本项目不接受未经报备的自动合并。
- **Automatically delete head branches**：建议开启，减少残留分支。

> GitHub 公共仓库无法彻底关闭“外部用户打开 PR”的入口。这里的策略是：**PR 可以出现，但未报备、未获邀的 PR 不得合并**。合并权必须由分支规则集收归维护者。

### 5.2 Actions（工作流）

进入 **Settings → Actions → General**。

#### 5.2.1 Actions permissions

二选一：

- **Allow all actions and reusable workflows**：配置简单，适合个人维护；
- **Allow select actions and reusable workflows**：更严格。至少允许：

  ```text
  actions/checkout@*
  actions/cache@*
  actions/upload-artifact@*
  actions/download-artifact@*
  ```

  工作流还使用 runner 预装的 `gh` CLI，不需要额外 marketplace action。

如选择限制列表，先确认语法与组织策略允许，否则 `release.yml` 会在启动时失败。

#### 5.2.2 Workflow permissions

选择：

- **Read and write permissions**

原因：`publish` job 需要创建 Release 并上传资产。工作流内部已经声明：

```yaml
permissions:
  contents: write
```

但仓库级设置不能是只读，否则 job 无法获得写权限。

同时：

- **Allow GitHub Actions to create and approve pull requests**：**关闭**；
- **Fork pull request workflows**：选择 **Require approval for all outside collaborators**，避免外部 PR 自动消耗 Actions 或读取敏感环境。

#### 5.2.3 Artifact and log retention

按需设置。发布工作流上传的 artifact 保留 14 天；Release 资产永久保留，除非手动删除。

### 5.3 Branches / Rulesets（分支保护）

进入 **Settings → Rules → Rulesets**（旧界面为 **Branches**）。

#### 5.3.1 新建 main 分支规则集

1. 点击 **New branch ruleset**；
2. **Ruleset Name**：`protect-main`；
3. **Enforcement status**：`Active`；
4. **Target branches** → **Add target** → **Include default branch**；
5. 开启以下规则：

| 规则 | 建议 | 说明 |
| --- | --- | --- |
| Restrict deletions | 开启 | 禁止删除 `main` |
| Block force pushes | 开启 | 禁止强推覆盖历史 |
| Require a pull request before merging | 开启 | 所有变更走 PR，便于留痕 |
| Required approvals | 0（单人维护）/ 1（有第二维护者） | 单人维护时 1 个 approval 会阻塞自己；可把维护者加入 bypass |
| Dismiss stale pull request approvals | 开启 | 新提交后旧批准失效 |
| Require review from Code Owners | 无 CODEOWNERS 时不要开启 | 未配置会阻塞合并 |
| Require conversation resolution | 开启 | 未解决讨论不得合并 |
| Require status checks to pass | 暂不开启 | 当前没有 PR CI；如后续添加 CI 再启用 |
| Require signed commits | 可选 | 会增加外部贡献门槛 |
| Require linear history | 可选 | 保持历史整洁 |
| Require deployments to succeed | 不开启 | 本项目无部署环境 |

6. **Bypass list**：只加入仓库所有者/维护者，且仅用于紧急修复；
7. 保存。

> 如果 required approvals 设为 1，而仓库只有你一个维护者，GitHub 不允许你批准自己的 PR。此时应：
> - 将 required approvals 设为 0；或
> - 将自己加入 bypass list；或
> - 增加第二维护者。
>
> 不要为了“看起来严格”而把自己锁死。

#### 5.3.2 PR 合并策略

即使规则集要求 PR，也要在 PR 页面确认：

- 不开启 auto-merge；
- 不开启“merge when ready”；
- 外部 PR 必须由维护者手动审查；
- 未提交 Modify Source Issue 的 PR，直接关闭并引用 `.github/CONTRIBUTING.md`；
- 任何 PR 合并前，确认没有删除法律文档、没有引入未授权代码。

#### 5.3.3 保护发布标签

进入 **Settings → Rules → Rulesets → New tag ruleset**：

- **Name**：`protect-release-tags`；
- **Target tags**：`v*`；
- **Enforcement**：Active；
- 开启：
  - **Restrict deletions**：禁止删除发布标签；
  - **Block force pushes**：禁止移动已发布标签；
- **Bypass**：仅维护者，且谨慎使用。

发布标签一旦被移动，已发布归档与源码 commit 可能不一致，会破坏校验与法律留痕。

### 5.4 Code security and analysis（代码安全）

进入 **Settings → Code security and analysis**。

按可用性开启：

- **Private vulnerability reporting**：开启；
- **Dependabot alerts**：开启；
- **Dependabot security updates**：开启；
- **Secret scanning**：开启（公开仓库通常默认可用）；
- **Secret scanning push protection**：开启；
- **Code scanning**：可选；如启用，先确保不会把安全告警误当成合并阻塞。

本项目不收集用户数据，但仓库仍可能因误提交而泄露 token、证书或签名材料。密钥扫描是必要防线。

### 5.5 Labels（标签）

进入 **Issues → Labels**，确认或创建：

| 标签 | 颜色建议 | 用途 |
| --- | --- | --- |
| `bug` | 红色 | 缺陷 |
| `enhancement` | 蓝色 | 功能建议 |
| `modify-source` | 紫色 | 源码修改报备 |
| `legal` | 黑色 | 法律/授权/侵权事项 |
| `needs-info` | 黄色 | 信息不足 |
| `security` | 深红 | 安全问题（公开 Issue 中不要披露细节） |

Issue 模板中的 `labels` 只有在标签已存在时才会生效。若模板没有自动打标，手动创建上述标签即可。

### 5.6 Moderation（互动限制）

进入 **Settings → Moderation**（如可用）：

- **Interaction limits**：遇到大规模骚扰、垃圾 Issue 或侵权争吵时可临时开启；
- **Blocked users**：仅在有明确理由时使用；
- 不要因为正常的技术质疑而封禁用户；但可对人身攻击、泄露隐私、发布侵权包的行为采取限制。

### 5.7 Pages / Wiki / Packages

- **Pages**：不需要开启。本项目没有静态站点；
- **Wiki**：关闭；
- **Packages**：除非未来发布 crates.io 之外的包，否则不需要。

---

## 6. Issue 模板与贡献规范验证

### 6.1 新建 Issue 测试

进入 **Issues → New issue**，确认出现三个选项：

1. **Bug report / 缺陷反馈**
2. **Feature request / 功能建议**
3. **Modify Source Report / 源码修改报备**

逐个打开，确认：

- 标题前缀正确；
- 法律声明完整；
- 表单字段可填写；
- 没有泄露隐私的必填项；
- Modify Source 模板明确写明“不报备即侵权”。

测试完成后，不要提交无意义的测试 Issue；如已提交，关闭并标注 `invalid`。

### 6.2 检查 CONTRIBUTING

打开 `.github/CONTRIBUTING.md`，确认包含：

- 本项目不是 OSI 开源项目；
- 不接受随意 PR；
- 所有源码修改必须提交 Modify Source Issue；
- 商业使用必须邮件申请书面授权；
- 禁止 Fork 后二次发布、商用；
- 侵权追责条款；
- 商业授权邮箱 `yanruoxinsu2@outlook.com`。

GitHub 会在新建 Issue/PR 时显示 `CONTRIBUTING.md` 链接；如未显示，检查文件名大小写与路径是否正确。

### 6.3 检查模板中的法律条款

每个模板必须包含四项核心约束：

1. 著作权归属；
2. 盈利组织禁止私自商用、二次分发；
3. 修改必须 Issue 报备；
4. 侵权追责。

同时必须明确：

- 本项目不属于 OSI 开源协议；
- 禁止 Fork 后二次发布、商用；
- 商业授权必须书面确认。

---

## 7. 发布流程

### 7.1 发布前准备

1. 确认 `main` 为最新，工作区干净：

   ```powershell
   git status
   git pull --ff-only
   ```

2. 更新版本号：

   ```toml
   [package]
   version = "1.0.0"
   ```

3. 如版本变化影响文档，更新 `README.md`、`packaging/README_PACKAGE.md`；
4. 如需要，复制 `packaging/release_notes_template.md` 的内容，填写本次发布说明；
5. 运行完整验证：

   ```powershell
   $env:CARGO_HOME = "$env:TEMP\cargo-home-probe"
   $env:CARGO_TARGET_DIR = "$env:TEMP\device-viewer-target"

   cargo fmt --all -- --check
   cargo check --tests
   cargo test
   cargo build --release --no-default-features --features "vulkan,opencl,sensors"
   ```

6. 手动运行一次 GUI 与 CLI：

   ```powershell
   .\target\release\device-viewer.exe --version
   .\target\release\device-viewer.exe --cli --only cpu,memory --quiet --out .\dist\smoke.json
   ```

7. 提交版本变更：

   ```powershell
   git add Cargo.toml Cargo.lock README.md packaging/
   git commit -m "chore: prepare v1.0.0 release"
   git push origin main
   ```

> 确保 `Cargo.lock` 与 `Cargo.toml` 一起提交，否则 CI 可能解析出不同依赖版本。

### 7.2 创建并推送标签

```powershell
git tag -a v1.0.0 -m "Device Viewer v1.0.0"
git push origin v1.0.0
```

标签必须：

- 以 `v` 开头；
- 版本部分匹配 `^[0-9A-Za-z][0-9A-Za-z._-]*$`；
- 指向已经推送到 `main` 的 commit；
- 不要在标签创建后继续修改同一 commit 的构建内容。

### 7.3 工作流执行过程

推送标签后，`.github/workflows/release.yml` 会执行：

1. **metadata**：校验标签，输出版本与 tag；
2. **build-windows**：MSVC x86_64 构建，打包 zip，生成 SHA-256；
3. **build-macos**：x86_64 与 aarch64 矩阵构建，打包 zip，附加签名/公证说明；
4. **build-linux**：musl x86_64 构建，打包 tar.gz，生成 SHA-256；
5. **publish**：下载全部 artifact，创建或更新 GitHub Release。

在 **Actions** 页面观察：

- 每个 job 是否绿色；
- Windows 是否缺少 MSVC 链接器；
- macOS 是否因 Gatekeeper/签名说明失败；
- Linux 是否缺少 X11/Wayland/GL 开发库；
- `publish` 是否因权限失败。

### 7.4 验证 Release 资产

发布完成后，在 **Releases** 页面确认：

- `DeviceViewer-windows-x86_64-1.0.0.zip`
- `DeviceViewer-windows-x86_64-1.0.0.zip.sha256`
- `DeviceViewer-macos-x86_64-1.0.0.zip`
- `DeviceViewer-macos-x86_64-1.0.0.zip.sha256`
- `DeviceViewer-macos-aarch64-1.0.0.zip`
- `DeviceViewer-macos-aarch64-1.0.0.zip.sha256`
- `DeviceViewer-linux-x86_64-1.0.0.tar.gz`
- `DeviceViewer-linux-x86_64-1.0.0.tar.gz.sha256`

每个归档必须包含：

- `DeviceViewer` 或 `DeviceViewer.exe`
- `LICENSE`
- `LEGAL_STATEMENT.md`
- `PRIVACY.md`
- `DISCLAIMER.md`
- `README_PACKAGE.md`

macOS 归档还应包含 `SIGNING_AND_NOTARIZATION.txt`。

### 7.5 下载并校验

```powershell
gh release download v1.0.0 --dir .\release-check
Get-FileHash .\release-check\DeviceViewer-windows-x86_64-1.0.0.zip -Algorithm SHA256
Get-Content .\release-check\DeviceViewer-windows-x86_64-1.0.0.zip.sha256
```

Linux：

```bash
sha256sum -c DeviceViewer-linux-x86_64-1.0.0.tar.gz.sha256
tar -tzf DeviceViewer-linux-x86_64-1.0.0.tar.gz
```

macOS：

```bash
shasum -a 256 -c DeviceViewer-macos-aarch64-1.0.0.zip.sha256
```

### 7.6 手动触发工作流

只有以下情况使用 **Actions → Release → Run workflow**：

- 标签推送后工作流因临时网络/缓存失败；
- 需要在同一 commit 上重跑打包；
- `publish` 失败但构建 artifact 已成功。

在 `tag` 输入框填写**已存在的标签**，例如 `v1.0.0`。

> **注意：** `workflow_dispatch` 的 `github.ref` 通常是分支，不是标签。构建 job 会检出分支 HEAD。手动触发前必须确认：
>
> ```powershell
> git rev-parse main
> git rev-parse v1.0.0^{commit}
> ```
>
> 两者必须相同，否则产物可能不对应标签。更安全的做法是：在失败的那次标签运行中点击 **Re-run jobs**，而不是重新 dispatch。

### 7.7 发布失败的处理

#### 构建失败

1. 查看失败 job 日志；
2. 在 `main` 修复；
3. 提交并推送；
4. 删除旧标签并重新打标签（仅限尚未发布 Release 时）：

   ```powershell
   git tag -d v1.0.0
   git push origin :refs/tags/v1.0.0
   git tag -a v1.0.0 -m "Device Viewer v1.0.0"
   git push origin v1.0.0
   ```

   如标签规则集禁止删除，需要维护者临时 bypass，操作后立即恢复保护。

#### `publish` 失败

- 检查 **Settings → Actions → General → Workflow permissions** 是否为 **Read and write**；
- 检查 `permissions.contents: write` 是否被组织策略覆盖；
- 检查 `GH_TOKEN` 是否使用了 `${{ github.token }}`，而不是不存在的 secret；
- 修复后重跑 `publish`，不要重复创建同名 Release。

#### 已发布 Release 发现错误

不要静默移动已发布标签。推荐：

1. 保留原标签与资产；
2. 在 `main` 修复；
3. 发布新的补丁版本 `v0.1.1`；
4. 在旧 Release 说明中标注“已知问题，请使用 v0.1.1”。

### 7.8 签名与公证

当前工作流**不包含** Apple 证书、私钥、App-Specific Password 或 API Key。macOS 包默认未签名、未公证，首次启动可能被 Gatekeeper 拦截。

正式签名/公证由维护者使用自己的 Apple Developer ID 在本地或受保护环境完成。**不要把证书、私钥、密码提交到仓库或写入 workflow。**

如未来接入 CI 签名：

- 使用 GitHub Environments 的 protection rules；
- 使用受保护的 secrets；
- 限制仅标签发布 job 可访问；
- 不在日志中打印凭据；
- 不把签名材料放入 artifact。

---

## 8. 发布后的持续维护

### 8.1 Issue 处理

- **Bug**：复现、标记版本、脱敏、修复后关联 commit；
- **Feature**：评估范围、平台、依赖、隐私影响；
- **Modify Source**：登记编号、核对修改位置与分发情况、确认是否商用；
- 对未报备的修改，要求补充 Issue；拒绝补充的，按侵权处理。

### 8.2 PR 处理

- 未报备、未获邀的 PR：直接关闭，引用 `CONTRIBUTING.md`；
- 已报备但未获邀的 PR：说明政策后关闭；
- 获邀 PR：只审查 Issue 范围内的变更；
- 不合并删除法律文档、改变许可模式、引入未授权依赖的 PR；
- 不开启 auto-merge。

### 8.3 侵权监测

定期搜索：

- 仓库名、产品名、独特类名、界面文案；
- GitHub/Gitee 镜像；
- 网盘、论坛、课程平台、应用商店；
- 闲鱼/淘宝等付费分发；
- 删除 `LICENSE` 或替换署名的 Fork。

发现侵权时：

1. 保存证据（URL、截图、时间、commit、下载包哈希）；
2. 向平台提交侵权投诉；
3. 发送停止侵权通知；
4. 必要时委托律师；
5. 公开处理进展时避免泄露个人信息。

### 8.4 法律文档更新

如修改 `LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`：

- 保留历史版本与生效日期；
- 不要追溯削弱已发布版本的权利；
- 同步更新 README、CONTRIBUTING、Issue 模板中的法律摘要；
- 重要变更建议单独发版并在 Release 说明中注明。

---

## 9. 常见问题

### 9.1 推送被拒绝：`non-fast-forward`

原因：远端已有提交（例如创建仓库时勾选了 README）。

处理：

```powershell
git pull --rebase origin main
git push origin main
```

如确认远端内容不需要：

```powershell
git push --force-with-lease origin main
```

`--force-with-lease` 仅用于你确认没有他人协作的初始阶段；不要对已发布历史强推。

### 9.2 `release.yml` 不触发

检查：

- 标签是否以 `v` 开头；
- 是否执行了 `git push origin v1.0.0`，而不是只创建本地标签；
- **Settings → Actions → General** 是否禁用了 Actions；
- 工作流文件是否在默认分支上存在；
- YAML 是否缩进错误。

### 9.3 `publish` 报 `Resource not accessible by integration`

处理：

- **Settings → Actions → General → Workflow permissions** 改为 **Read and write**；
- 确认工作流顶部有 `permissions: contents: write`；
- 确认组织策略未强制只读；
- 不要改用个人 PAT 硬编码到 workflow。

### 9.4 Windows 构建找不到链接器

安装 **Visual Studio 2022 Build Tools**，勾选 **Desktop development with C++**，包含 MSVC v143 与 Windows SDK。仅安装 Rust 不够。

### 9.5 Linux 构建缺少 X11/Wayland/GL

工作流已安装常见开发库。如仍失败，查看错误中缺失的 `-sys` crate，补对应 `-dev` 包。不要通过关闭 `wayland` feature 来“假装修复”，因为 eframe 的依赖树仍可能拉取 Wayland。

### 9.6 macOS 包无法打开

这是未签名/未公证导致的 Gatekeeper 拦截，不是构建失败。确认 SHA-256 后：

```bash
xattr -dr com.apple.quarantine DeviceViewer
```

正式分发应完成签名与公证。

### 9.7 Issue 模板没有出现

检查：

- 文件是否在 `.github/ISSUE_TEMPLATE/`；
- 扩展名是否为 `.md`；
- YAML front matter 是否以 `---` 开始和结束；
- 默认分支是否包含这些文件；
- 仓库 Issues 是否开启。

### 9.8 标签写错版本

未发布 Release 时：

```powershell
git tag -d v1.0.0
git push origin :refs/tags/v1.0.0
git tag -a v1.0.0 -m "Device Viewer v1.0.0"
git push origin v1.0.0
```

已发布 Release 时：不要移动标签，发布新的补丁版本。

---

## 10. 法律与联系方式

- **商业授权 / 法律事务：** yanruoxinsu2@outlook.com
- **安全漏洞：** yanruoxinsu2@outlook.com（主题：`[SECURITY] Device Viewer`）
- **源码修改报备：** 在本仓库提交 **Modify Source Report** Issue
- **完整条款：** [`LICENSE`](LICENSE)、[`LEGAL_STATEMENT.md`](LEGAL_STATEMENT.md)、[`PRIVACY.md`](PRIVACY.md)、[`DISCLAIMER.md`](DISCLAIMER.md)

> 本项目不是 OSI 开源项目。禁止 Fork 后二次发布、二次开源、二次闭源、二次打包、二次分发、商用售卖、SaaS 集成、付费工具集成或课程售卖。任何商业使用必须提前取得作者书面授权；任何源码修改必须提交 Modify Source Issue 报备。未报备的修改、未授权的商用与二次发布均属于侵权，作者将依法追究民事责任，情节严重的将向公安机关报案。

---

**Copyright © Device Viewer 原作者。保留所有权利。**
