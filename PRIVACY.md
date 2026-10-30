# Device Viewer 隐私声明

**生效日期：2026 年 10 月 30 日**  
**适用项目：Device Viewer**  
**适用法律：中华人民共和国大陆地区法律**

---

## 一、核心承诺

1.1 Device Viewer 是**纯本地运行**的桌面工具。

1.2 Device Viewer **不上传**任何硬件信息、设备标识、快照、日志、配置、使用记录或个人数据。

1.3 Device Viewer **不收集**统计分析、崩溃报告、使用行为、设备指纹、位置信息、账号信息或广告标识。

1.4 Device Viewer **不包含**账号系统、云同步、在线激活、远程配置、广告 SDK、埋点 SDK 或第三方追踪代码。

1.5 Device Viewer 默认**不发起网络请求**。导出、复制、保存、对比、查看日志均在本地完成。

1.6 本声明与 [LICENSE](LICENSE)、[LEGAL_STATEMENT.md](LEGAL_STATEMENT.md)、[DISCLAIMER.md](DISCLAIMER.md) 共同适用。

---

## 二、本地处理的数据

Device Viewer 在本地读取和处理以下类别的信息，用于界面展示、快照保存与报告导出：

| 类别 | 示例 | 是否离开本机 |
| --- | --- | --- |
| 系统信息 | 操作系统、内核版本、主机名、架构、启动时间 | 否 |
| CPU | 厂商、型号、核心数、频率、占用率 | 否 |
| 内存 | 容量、使用量、交换分区 | 否 |
| 主板 / BIOS | 厂商、型号、版本、序列号、资产标签 | 否 |
| PCIe / USB | 设备地址、VID/PID、厂商、产品、序列号、驱动 | 否 |
| 存储 | 型号、固件、容量、SMART 健康、温度、通电时间、写入量 | 否 |
| 显示器 | 分辨率、刷新率、位置、适配器 | 否 |
| 图形 / 计算 | Vulkan、OpenGL、OpenCL、Metal、DirectX、OpenVX 信息 | 否 |
| 传感器 | 温度、风扇转速、电压 | 否 |
| 网络接口 | 接口名、MAC 地址、IP 地址、累计收发字节 | 否 |
| 音频 | 端点名称、厂商 | 否 |
| 带宽 | 本机内存读、写、拷贝采样 | 否 |

上述数据仅在本机内存中处理，或由用户主动保存到本机文件。

---

## 三、配置文件

3.1 Device Viewer 在本机保存一个配置文件，文件名为 `device-viewer.json`。

3.2 配置文件可能包含：

- 界面语言；
- 明暗主题；
- 单位制（二进制 / 十进制）；
- 当前页面；
- 窗口位置与尺寸；
- 筛选条件；
- 收藏的 Vulkan 扩展；
- 快照索引（文件名、标签、创建时间、大小）；
- 便携模式开关；
- 其他界面偏好。

3.3 配置文件**不包含**硬件采集结果。硬件结果仅存在于内存、用户主动保存的快照或用户主动导出的报告中。

3.4 配置文件不会被上传、同步、共享或用于分析。

---

## 四、快照文件

4.1 用户可以在界面中保存快照，扩展名为 `.dvsnap`；也支持导入 `.dvsnap` 或 `.json` 快照。

4.2 快照是用户主动创建的本机文件，内容为某一时刻的硬件信息结构化数据。

4.3 快照可能包含：

- 主机名；
- 主板、BIOS、存储、USB 设备的序列号或资产标签；
- 网卡 MAC 地址与 IP 地址；
- 显示器标识；
- GPU、CPU、内存的型号与能力信息；
- 传感器读数；
- 采集过程中的警告与错误信息。

4.4 快照由用户自行保管。Device Viewer 不会自动上传、同步或分享快照。

4.5 快照可能包含可直接识别设备或个人的信息。将快照发送给他人、上传到工单系统、附在公开 Issue 或用于商业交付前，请自行审查并脱敏。

4.6 快照对比功能在本地读取用户选择的文件，不联网、不上传。

---

## 五、日志文件

5.1 Device Viewer 在本机保存内部日志，文件名为 `device-viewer.log`。

5.2 日志用于记录启动、采集进度、模块警告、导出结果、配置保存与异常信息，便于用户自查和问题反馈。

5.3 日志可能包含：

- 时间戳；
- 模块名称；
- 设备名称或路径；
- 错误信息与系统返回文本；
- 导出文件路径；
- 配置保存结果。

5.4 日志不会自动上传。用户主动提交日志用于问题反馈时，应自行检查并删除敏感信息。

5.5 日志为追加写入，用户可随时删除；删除后程序会在下次运行时重新创建。

---

## 六、存储位置

### 6.1 非便携模式

| 平台 | 配置与快照根目录 | 配置文件 | 快照目录 | 日志文件 |
| --- | --- | --- | --- | --- |
| Windows | `%APPDATA%\DeviceViewer` | `device-viewer.json` | `snapshots\` | `device-viewer.log` |
| macOS | `~/Library/Application Support/DeviceViewer` | `device-viewer.json` | `snapshots/` | `device-viewer.log` |
| Linux | `$XDG_CONFIG_HOME/device-viewer` 或 `~/.config/device-viewer` | `device-viewer.json` | `snapshots/` | `device-viewer.log` |

### 6.2 便携模式

启用便携模式后，配置、快照与日志保存在**可执行文件所在目录**：

```text
<可执行文件目录>/
├── device-viewer.json
├── device-viewer.log
└── snapshots/
    ├── snapshot-....dvsnap
    └── ....json
```

便携模式适合 U 盘、移动硬盘、离线工作站与受控环境。删除该目录即可清除便携模式下的全部本地数据。

### 6.3 模式切换

在设置中切换便携模式后，程序会将后续配置与快照路径切换到对应根目录。旧路径下的文件不会被自动删除；如不再需要，请自行清理。

---

## 七、外部命令与系统接口

7.1 为读取部分硬件信息，Device Viewer 可能在本机调用以下外部命令或系统接口：

- `smartctl`：读取存储 SMART 健康信息；
- `nvidia-smi`：读取 NVIDIA GPU 温度；
- `system_profiler`：读取 macOS 硬件、USB、显示、音频信息；
- Windows DXGI、GDI、SetupAPI；
- Linux sysfs、procfs、hwmon、DRM；
- Vulkan Loader、OpenCL ICD、Metal、OpenVX 运行时。

7.2 上述命令与接口均在本机执行，用于读取信息，不用于上传数据。

7.3 外部命令是否可用、返回哪些字段，取决于系统安装、驱动版本、设备权限与厂商实现。缺失时程序会给出警告或跳过，不会因此联网获取替代数据。

7.4 外部命令由其各自权利人提供，用户应遵守其许可与隐私规则。Device Viewer 不控制第三方命令的行为，但不会主动向其传递用户文件或个人信息。

---

## 八、权限说明

8.1 部分信息可能需要管理员（Windows）或 root（Linux/macOS）权限，例如：

- 存储 SMART 数据；
- 部分主板、传感器、风扇、电压信息；
- 部分 PCIe、USB 或显示适配器信息；
- 部分 NVIDIA GPU 温度。

8.2 权限仅用于本机读取。Device Viewer 不会因提权而上传数据，也不会修改固件、BIOS、驱动、分区、超频设置或硬件配置。

8.3 用户应自行评估在受控设备、公司设备或生产设备上运行时的权限与合规要求。

---

## 九、复制、导出与剪贴板

9.1 复制按钮会将用户选择的文本写入系统剪贴板；剪贴板由操作系统管理，可能被其他本地程序读取。

9.2 导出功能将报告写入用户指定路径；路径由用户选择，程序不会自动上传。

9.3 Markdown 表格复制、JSON 复制、日志复制与快照标签编辑均在本地完成。

9.4 用户应避免将包含序列号、MAC 地址、IP 地址、主机名或内部资产信息的报告复制到公开渠道。

---

## 十、无追踪与无广告

10.1 Device Viewer 不设置 Cookie、不生成广告标识、不读取通讯录、不读取短信、不读取通话记录、不读取照片、不读取麦克风、不读取摄像头、不读取精确位置。

10.2 Device Viewer 不集成第三方分析平台、广告平台、推送平台、社交登录、支付 SDK 或用户画像服务。

10.3 Device Viewer 不进行 A/B 测试、远程开关、灰度发布、在线配置下发或使用行为回传。

10.4 Device Viewer 不销售、出租、交换或向任何第三方提供用户数据。

---

## 十一、数据保留与删除

11.1 Device Viewer 不设云端账户，不存在“云端数据保留”。

11.2 本地数据保留期限由用户自行控制：

- 删除 `device-viewer.json`：清除配置与快照索引；
- 删除 `device-viewer.log`：清除日志；
- 删除 `snapshots/`：清除全部快照；
- 删除便携目录：清除便携模式全部数据；
- 卸载程序：删除程序文件；如未手动删除，配置、日志与快照可能仍保留在用户目录。

11.3 删除文件后，操作系统或其他备份工具可能仍保留副本；该等副本由用户或第三方工具管理，不属于 Device Viewer 的控制范围。

---

## 十二、第三方组件

12.1 Device Viewer 使用 Rust 生态中的第三方库，包括但不限于 eframe、egui、glow、tokio、serde、sysinfo、pci-info、ash、opencl3、metal、openvx、smartctl-rs。

12.2 这些库在本机运行，用于界面、异步调度、序列化与硬件读取。Device Viewer 未在其上叠加广告、追踪或数据回传逻辑。

12.3 第三方库可能包含其自身的许可条款与隐私说明；用户应自行查阅。Device Viewer 的隐私承诺不改变第三方组件对其自身行为的责任边界。

12.4 本项目不控制操作系统、驱动、固件、系统工具或第三方运行时的行为。操作系统更新、驱动更新或第三方工具变更，可能影响数据读取范围。

---

## 十三、未成年人保护

13.1 Device Viewer 不面向未成年人单独收集任何个人信息。

13.2 由于本软件不设账号、不联网、不收集数据，不存在针对未成年人的在线画像、精准营销或跨应用追踪。

13.3 未成年人使用本软件，应在监护人指导下进行，避免在公开渠道分享包含设备序列号、MAC 地址、IP 地址或主机名的快照。

---

## 十四、跨境传输

14.1 Device Viewer 不进行数据跨境传输。

14.2 用户主动将快照、日志或报告发送至境外平台、境外客户或境外服务商时，应自行遵守《中华人民共和国个人信息保护法》《中华人民共和国数据安全法》及相关出口管制、保密与合规要求。

14.3 作者不参与用户主动发起的跨境传输，也不对用户的传输行为承担责任。

---

## 十五、声明更新

15.1 作者有权根据功能变化、法律要求或安全需要更新本声明。

15.2 更新后的声明通过项目仓库、发布页面或随附文档公布，自公布之日起生效。

15.3 重大变更可能包括：新增本地存储位置、新增外部命令、新增导出格式、调整便携模式规则。

15.4 用户继续使用本项目，视为接受更新后的声明；不接受更新的，应停止使用并删除全部副本。

---

## 十六、联系我们

如对本声明有疑问、意见或投诉，请联系：

**yanruoxinsu2@outlook.com**

请在邮件中说明：问题类型、涉及版本、复现步骤、相关文件路径（如适用）。请勿在首次联系时发送包含序列号、MAC 地址、IP 地址或主机名的完整快照；如确需提供，请先脱敏。

---

## 十七、核心法律条款

本声明受 [LICENSE](LICENSE) 与 [LEGAL_STATEMENT.md](LEGAL_STATEMENT.md) 约束，核心条款如下：

1. 本项目完整著作权归原作者独有。
2. 个人、非营利组织：可以免费本地使用、学习研究。
3. 所有盈利组织、商业团队、企业、工作室、付费开发者：禁止使用、复制、修改、衍生、二次开源、二次闭源、二次打包、二次分发、二次发布、商用售卖、集成进商业项目、SaaS、付费工具、课程售卖。所有商业用途必须提前发送邮件 yanruoxinsu2@outlook.com 获取作者书面授权，未经授权全部属于侵权。
4. 任何个人/组织修改源代码，必须在项目仓库提交 Issue 写明修改内容，不报备视为侵权。
5. 严禁任何主体修改代码后二次公开、二次发布、二次开源、二次引流售卖。
6. 一旦查实商业侵权、二次发布、未授权商用、私自魔改分发行为，作者将直接依据《中华人民共和国著作权法》启动完整法律追责流程，包含民事起诉、侵权索赔、下架全部侵权项目、公开追责。
7. 协议最终解释权归原作者所有；管辖法律：中华人民共和国大陆地区法律。

---

## English Summary

Device Viewer is a local-only desktop application. It does **not** upload hardware information, device identifiers, snapshots, logs, configuration, usage records or personal data. It contains **no** analytics, crash reporting, accounts, cloud sync, online activation, advertising SDKs, tracking SDKs or telemetry.

Configuration (`device-viewer.json`), snapshots (`snapshots/*.dvsnap`, `*.json`) and logs (`device-viewer.log`) are stored locally:

- Windows: `%APPDATA%\DeviceViewer`
- macOS: `~/Library/Application Support/DeviceViewer`
- Linux: `$XDG_CONFIG_HOME/device-viewer` or `~/.config/device-viewer`

In portable mode, these files are stored beside the executable. Snapshots may contain hostnames, serial numbers, MAC addresses and IP addresses; review and redact before sharing. External tools such as `smartctl`, `nvidia-smi` and `system_profiler` are executed locally for read-only collection. Clipboard and export operations are local. Local data can be removed by deleting the files or the portable directory. For questions, contact **yanruoxinsu2@outlook.com**.

---

**Copyright © 2026 Device Viewer 原作者. All rights reserved.**
