# Device Viewer {{VERSION}}

> 发布日期：{{RELEASE_DATE}}  
> 提交：`{{COMMIT_SHA}}`  
> 构建编号：`{{RUN_ID}}`

<!--
发布维护者填写说明：
1. 将 {{VERSION}}、{{RELEASE_DATE}}、{{COMMIT_SHA}}、{{RUN_ID}} 替换为实际值。
2. 填写“本次亮点”“变更内容”“已知问题”“校验和”。
3. 不要在本文件或发布说明中写入任何 token、API Key、证书、私钥、Apple ID 密码。
4. 发布前确认所有归档都包含 LICENSE、LEGAL_STATEMENT.md、PRIVACY.md、DISCLAIMER.md。
-->

## 本次亮点

<!-- 用 3-5 条短句说明本版本最重要的变化。 -->
- {{HIGHLIGHT_1}}
- {{HIGHLIGHT_2}}
- {{HIGHLIGHT_3}}

## 下载

| 平台 | 架构 | 文件 | SHA-256 |
| --- | --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-{{VERSION}}.zip` | `{{SHA256_WINDOWS}}` |
| macOS | x86_64 | `DeviceViewer-macos-x86_64-{{VERSION}}.zip` | `{{SHA256_MACOS_X86_64}}` |
| macOS | aarch64 | `DeviceViewer-macos-aarch64-{{VERSION}}.zip` | `{{SHA256_MACOS_AARCH64}}` |
| Linux | x86_64 (musl) | `DeviceViewer-linux-x86_64-{{VERSION}}.tar.gz` | `{{SHA256_LINUX}}` |

每个压缩包内均包含 `README_PACKAGE.md`、`LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md` 与 `DISCLAIMER.md`。

## 安装与运行

### Windows

1. 解压 `DeviceViewer-windows-x86_64-{{VERSION}}.zip`。
2. 安装 **Microsoft Visual C++ 2015-2022 Redistributable (x64)**。
3. 运行 `DeviceViewer.exe`；需要便携模式时使用 `DeviceViewer.exe --portable`。
4. SMART、部分传感器可能需要管理员权限；程序不会主动提权。

### macOS

1. 解压对应架构的 zip。
2. 首次启动如被 Gatekeeper 拦截，请在“系统设置 → 隐私与安全性”中允许，或在确认校验和后执行 `xattr -dr com.apple.quarantine DeviceViewer`。
3. 本版本默认未签名、未公证；正式分发前请由维护者使用自己的 Apple Developer ID 完成签名与公证。
4. 需要 SMART 时请自行安装 `smartctl` 并以 `sudo` 运行。

### Linux

```bash
tar -xzf DeviceViewer-linux-x86_64-{{VERSION}}.tar.gz
cd DeviceViewer-linux-x86_64-{{VERSION}}
./DeviceViewer
```

无图形环境时：

```bash
./DeviceViewer --cli --out report.json
```

需要 SMART 时请安装 `smartctl` 并以 root 运行；X11/Wayland、Mesa 或厂商 GL 驱动由系统提供。

## 变更内容

### 新增

- {{ADDED_1}}
- {{ADDED_2}}

### 改进

- {{IMPROVED_1}}
- {{IMPROVED_2}}

### 修复

- {{FIXED_1}}
- {{FIXED_2}}

### 移除 / 弃用

- {{REMOVED_1}}

## 兼容性说明

- Windows：Windows 10/11 x64；需要 MSVC 运行库与可用显卡驱动。
- macOS：macOS 11+；Intel 与 Apple Silicon 分别提供归档。
- Linux：x86_64；musl 静态链接，仍需系统提供 X11/Wayland 与 OpenGL/EGL。
- Vulkan、OpenCL、OpenVX、Metal、SMART、传感器等能力依赖设备、驱动、权限与运行时组件；缺失时对应面板显示“不可用”，不影响其他模块。
- 硬件信息与 SMART 数据仅供参考，不应作为高危生产环境的唯一诊断依据。

## 已知问题

- {{KNOWN_ISSUE_1}}
- {{KNOWN_ISSUE_2}}

## 校验

```text
{{CHECKSUM_BLOCK}}
```

Windows 可使用 `Get-FileHash -Algorithm SHA256`，macOS/Linux 可使用 `shasum -a 256` 或 `sha256sum -c`。

## 法律与隐私

- 项目完整著作权归原作者独有。
- 个人、非营利组织可免费本地使用、学习研究。
- 盈利组织、商业团队、企业、工作室、付费开发者未经书面授权，禁止使用、复制、修改、衍生、二次开源、二次闭源、二次打包、二次分发、二次发布、商用售卖、集成进商业项目、SaaS、付费工具、课程售卖。
- 商业授权：**yanruoxinsu2@outlook.com**。
- 修改源代码必须在项目仓库提交 Issue 报备；禁止修改后二次公开、二次发布、二次开源、二次引流售卖。
- 完整条款：[LICENSE](../LICENSE)、[LEGAL_STATEMENT.md](../LEGAL_STATEMENT.md)、[PRIVACY.md](../PRIVACY.md)、[DISCLAIMER.md](../DISCLAIMER.md)。

程序纯本地运行，不上传硬件信息、快照、日志或使用统计。分享快照前请检查其中可能包含的主机名、序列号、MAC 地址与 IP 地址。

---

Copyright © {{YEAR}} Device Viewer 原作者。保留所有权利。
