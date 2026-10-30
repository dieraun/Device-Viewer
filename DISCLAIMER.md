# Device Viewer 免责声明

**生效日期：2026 年 10 月 30 日**  
**适用项目：Device Viewer**  
**适用法律：中华人民共和国大陆地区法律**

---

## 一、总则

1.1 本免责声明（以下简称“本声明”）适用于 Device Viewer 的源代码、目标代码、可执行文件、文档、快照格式、导出报告及相关材料（以下统称“本软件”）。

1.2 任何主体下载、安装、运行、复制、修改、分发或以其他方式使用本软件，即视为已阅读、理解并同意本声明全部内容。

1.3 若不同意本声明任何内容，请立即停止使用并删除本软件及其全部副本。

1.4 本声明与 [LICENSE](LICENSE)、[LEGAL_STATEMENT.md](LEGAL_STATEMENT.md)、[PRIVACY.md](PRIVACY.md) 共同适用。本声明不免除 LICENSE 中的商业使用禁止、修改报备、禁止二次发布等义务。

---

## 二、按“现状”提供

2.1 本软件按**“现状”（AS IS）**和**“现有可用”（AS AVAILABLE）**提供。

2.2 在适用法律允许的最大范围内，作者不对本软件作出任何明示或默示保证，包括但不限于：

- 适销性保证；
- 特定用途适用性保证；
- 不侵权保证；
- 准确性保证；
- 完整性保证；
- 及时性保证；
- 稳定性保证；
- 安全性保证；
- 无错误运行保证；
- 不间断运行保证；
- 与特定硬件、驱动、固件、操作系统或第三方工具兼容的保证。

2.3 作者不承诺修复任何缺陷、提供任何更新、提供任何技术支持、维持任何接口稳定或继续开发本软件。

2.4 作者有权随时修改、暂停、终止本软件的全部或部分功能，无需事先通知。

---

## 三、硬件信息准确性

3.1 本软件展示的硬件信息来自操作系统接口、驱动、固件、系统工具、第三方库与设备自身报告。

3.2 上述来源可能存在以下问题：

- 字段缺失、截断、重复、过期或格式不一致；
- 厂商字符串不规范、未本地化或包含乱码；
- 驱动未暴露完整能力；
- 固件报告错误或省略关键字段；
- 虚拟化、容器、云主机、远程桌面隐藏真实硬件；
- 权限不足导致部分字段不可见；
- 系统工具版本差异导致解析结果不同；
- 设备热插拔、休眠、唤醒或驱动重载导致状态变化；
- 多 GPU、多网卡、多显示器、外接扩展坞导致枚举顺序变化；
- USB、PCIe、显示接口的链路速率与宽度由协商结果决定，可能与设备标称值不同。

3.3 本软件会尽力标注警告、缺失项与不可用状态，但**不保证**任何字段绝对准确、完整、实时或可用于特定目的。

3.4 用户应将本软件输出视为“参考信息”，而不是厂商规格书、硬件检测报告、司法鉴定意见、计量报告或合规证明。

---

## 四、SMART、传感器与健康数据

4.1 存储 SMART 数据由设备固件、驱动与 `smartctl` 等外部工具提供，具有以下局限：

- SMART 通过不代表设备未来不会故障；
- SMART 失败可能是误报、解析错误或厂商实现差异；
- 不同厂商的属性编号、阈值、原始值含义并不统一；
- NVMe 与 SATA 的字段语义不同；
- USB 桥接、RAID 控制器、虚拟磁盘可能无法透传 SMART；
- 温度、通电时间、写入量、坏块、寿命百分比可能缺失或偏差；
- 外部工具的版本、权限与命令行为会影响结果。

4.2 温度、电压、风扇转速等传感器数据取决于传感器校准、主板实现、驱动支持与权限。读数可能偏差、抖动、冻结或不可用。

4.3 GPU 利用率、显存使用、显存预算、链路速率等数据可能由驱动估算、采样或聚合，存在延迟与误差。

4.4 本软件**不是**硬件诊断仪、示波器、万用表、热成像仪、SMART 认证工具、数据恢复工具或维修决策系统。

4.5 用户不得仅凭本软件输出作出以下决定：

- 判定存储设备健康或报废；
- 判定硬件故障或索赔；
- 决定是否继续运行关键业务；
- 决定是否更换、返修、退货或起诉供应商；
- 决定是否删除、覆盖或迁移数据；
- 决定是否超频、降压、调整风扇或修改固件。

---

## 五、兼容性与设备差异

5.1 本软件支持 Windows、Linux 与 macOS，但不同平台、版本、架构、驱动、固件与设备的能力覆盖并不一致。

5.2 以下情况可能导致部分功能不可用、字段缺失、界面显示异常或采集超时：

- 操作系统版本过旧或过新；
- 缺少图形开发库、Vulkan Loader、OpenCL ICD、OpenVX SDK；
- 缺少 `smartctl`、`nvidia-smi` 或 `system_profiler`；
- 驱动未安装、版本不匹配或厂商定制；
- 设备被虚拟机、容器、云平台、远程桌面或安全软件隔离；
- 企业安全策略限制设备枚举、驱动加载或外部命令执行；
- 硬件被禁用、脱机、休眠或热插拔；
- 多设备枚举顺序、名称重复或路径变化；
- 非标准硬件、工程样品、开发板、矿卡、改装设备；
- 未来发布的新硬件或新接口。

5.3 上述差异属于硬件与系统环境的正常现象，不必然构成本软件缺陷。作者不承诺对每一种设备组合提供完整支持。

5.4 用户应在目标设备上自行验证输出，不应将其他设备、其他系统或其他版本的结果直接套用。

---

## 六、只读采集与无硬件修改

6.1 本软件的设计目标是**只读采集与展示**，不提供 BIOS、固件、驱动、分区、超频、电压、风扇曲线或硬件配置的写入功能。

6.2 本软件不会主动修改磁盘分区、引导记录、文件系统、注册表关键项、驱动配置或设备固件。

6.3 本软件的内存带宽测试会在本机分配内存缓冲区并执行读写拷贝，用于采样带宽；该操作不写入用户文件，但会占用一定内存与 CPU 资源。

6.4 本软件可能调用外部命令读取信息；外部命令自身的行为由其权利人控制。用户应自行确认外部命令的来源、版本与安全性。

6.5 尽管本软件以只读为目标，作者仍不对以下情况承担责任：

- 因操作系统、驱动、固件、外部工具或第三方软件导致的硬件变化；
- 因用户同时运行其他工具导致的冲突、死机、蓝屏、掉盘或数据损坏；
- 因权限提升、安全软件拦截、企业策略或人为操作导致的事故；
- 因设备本身已存在故障而在采集过程中暴露或加重。

---

## 七、高危与关键场景禁止

7.1 本软件**不得**用于以下高风险或关键场景：

- 医疗诊断、治疗、监护、生命维持；
- 航空、航天、飞行控制、导航；
- 核设施、核材料、辐射控制；
- 能源、电力、电网、油气、化工过程控制；
- 铁路、公路、船舶、自动驾驶、交通调度；
- 军工、武器、爆炸物、安防控制；
- 应急通信、消防、救援、公共安全；
- 工业自动化、机器人、生产线停机决策；
- 金融交易、清算、风控、支付；
- 数据中心关键业务切换、容灾、停机；
- 任何一旦信息错误、延迟或不可用就可能导致人身伤害、死亡、重大财产损失、环境破坏或社会影响的场景。

7.2 本软件**不得**作为上述场景的诊断依据、决策依据、安全认证依据、合规依据或唯一监控手段。

7.3 本软件**不得**用于高危生产诊断、故障根因判定、设备报废判定、索赔定损、保险核赔、司法鉴定或安全审计。

7.4 用户若将本软件用于上述场景，应自行承担全部风险与责任；作者明确拒绝承担由此产生的任何责任。

---

## 八、用户责任

8.1 用户应自行备份重要数据，并确保在采集、导出、对比或分享前已采取必要的数据保护措施。

8.2 用户应自行确认其有权在目标设备上运行本软件，包括公司设备、客户设备、生产设备、租赁设备、共享设备与受控设备。

8.3 用户应自行遵守所在国家或地区的法律、法规、行业规范、保密义务、出口管制、数据安全与个人信息保护要求。

8.4 用户应自行审查快照、日志与导出报告，避免泄露主机名、序列号、MAC 地址、IP 地址、内部资产编号、网络拓扑或安全配置。

8.5 用户应自行验证外部命令的来源、版本、完整性与安全性，避免执行被篡改的 `smartctl`、`nvidia-smi`、`system_profiler` 或恶意替换程序。

8.6 用户应自行评估权限提升的必要性与风险，避免在生产设备、共享设备或不受控环境中使用管理员/root 权限。

8.7 用户不得使用本软件从事违法活动、侵犯他人权利、规避技术保护措施、破坏计算机信息系统或危害网络安全。

---

## 九、责任限制

9.1 在适用法律允许的最大范围内，作者不对因使用或无法使用本软件产生的任何直接、间接、附带、特殊、惩罚性、示范性或后果性损失承担责任。

9.2 前款损失包括但不限于：

- 数据丢失、损坏、泄露、无法恢复；
- 设备损坏、掉盘、死机、蓝屏、过热、断电；
- 业务中断、生产停滞、项目延期、客户流失；
- 利润损失、收入损失、商誉损失、机会损失；
- 误诊、误判、误报、漏报；
- 采购、维修、更换、召回、索赔、诉讼成本；
- 第三方索赔、行政处罚、刑事风险；
- 因快照、日志、报告泄露导致的隐私、商业秘密或合规责任；
- 因外部命令、第三方库、驱动、固件、操作系统导致的问题；
- 因用户违反法律、合同、保密义务或操作规范导致的问题。

9.3 即使作者已被告知上述损失的可能性，作者仍不承担责任。

9.4 在适用法律不允许完全排除或限制责任的范围内，作者的责任以用户为本软件实际支付的费用为上限；本软件为免费提供时，该上限为零。

9.5 本条不排除或限制依法不得排除或限制的责任。

---

## 十、第三方组件与外部命令

10.1 本软件使用第三方库与系统接口，包括但不限于 eframe、egui、glow、tokio、serde、sysinfo、pci-info、ash、opencl3、metal、openvx、smartctl-rs、Windows DXGI/GDI/SetupAPI、Linux sysfs/hwmon/DRM、macOS system_profiler。

10.2 第三方组件由其各自权利人提供，适用其各自许可条款。作者不对第三方组件的质量、安全性、合法性、准确性、兼容性或持续可用性作出保证。

10.3 本软件可能调用 `smartctl`、`nvidia-smi`、`system_profiler` 等外部命令。作者不控制这些命令的行为、输出、权限、版本或副作用。

10.4 第三方组件或外部命令的缺陷、变更、停更、收费、封禁、恶意篡改或安全漏洞，可能导致本软件功能异常；作者不承担由此产生的责任。

10.5 用户应自行查阅并遵守第三方组件的许可条款，不得因使用本软件而违反第三方许可。

---

## 十一、出口管制与合规

11.1 用户应自行确认本软件的使用、出口、再出口、转让、远程访问或技术服务是否受到适用出口管制、制裁、禁运、行业监管或合同限制。

11.2 用户不得将本软件用于受制裁国家或地区、受限制主体、军事用途、武器开发、核扩散、恐怖主义或其他违法用途。

11.3 用户应自行取得必要的政府许可、客户授权、保密审批、安全评估或合规意见。

11.4 作者不承担因用户违反出口管制、制裁、行业监管或合同义务产生的任何责任。

---

## 十二、声明更新

12.1 作者有权根据功能变化、法律要求、安全需要或风险变化更新本声明。

12.2 更新后的声明通过项目仓库、发布页面或随附文档公布，自公布之日起生效。

12.3 用户继续使用本软件，视为接受更新后的声明；不接受更新的，应停止使用并删除全部副本。

12.4 本声明任何条款被认定无效、非法或不可执行的，不影响其余条款的效力；应以最接近原条款目的的有效条款替代。

---

## 十三、法律适用与争议解决

13.1 本声明的订立、效力、解释、履行、变更、终止及争议解决，均适用**中华人民共和国大陆地区法律**（不含香港特别行政区、澳门特别行政区及台湾地区法律）。

13.2 因本声明或本软件引起的任何争议，双方应首先友好协商；协商不成的，任何一方均可向**作者住所地、侵权行为地或侵权结果发生地有管辖权的人民法院**提起诉讼。

13.3 本声明与 LICENSE 不一致的，以 LICENSE 的许可条款为准；涉及免责、责任限制与风险分配的，以本声明中更有利于保护作者的表述为准。

13.4 本声明未尽事宜，适用中华人民共和国大陆地区相关法律法规。

---

## 十四、联系

如对本声明有疑问，请联系：

**yanruoxinsu2@outlook.com**

商业授权、修改报备与侵权举报，请同时参阅 [LICENSE](LICENSE) 与 [LEGAL_STATEMENT.md](LEGAL_STATEMENT.md)。

---

## 十五、核心法律条款

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

Device Viewer is provided **“as is”** and **“as available”**, without warranties of merchantability, fitness for a particular purpose, non-infringement, accuracy, completeness, timeliness, stability, security, error-free operation, uninterrupted operation, or compatibility with any specific hardware, driver, firmware, operating system or third-party tool.

Hardware information, SMART data, sensor readings, GPU utilization, VRAM usage and link speeds come from operating-system interfaces, drivers, firmware, external tools and device-reported values. They may be missing, delayed, inaccurate, vendor-specific or affected by virtualization, permissions and driver versions. SMART “passed” does not guarantee future health; SMART “failed” may be a false positive.

The software is intended for read-only inspection. It does not write BIOS, firmware, drivers, partitions, overclocking settings, fan curves or hardware configuration. The memory bandwidth test allocates local buffers and does not write user files.

**Do not use Device Viewer in high-risk or safety-critical environments**, including medical, aviation, aerospace, nuclear, energy, chemical, transportation, autonomous driving, military, emergency response, industrial control, financial trading, or any scenario where incorrect, delayed or unavailable information could cause injury, death, major property loss, environmental damage or social impact. Do not use it as the sole basis for production diagnostics, device retirement, warranty claims, insurance claims, litigation, safety certification or compliance audits.

To the maximum extent permitted by law, the author disclaims liability for any direct, indirect, incidental, special, punitive, exemplary or consequential loss, including data loss, device damage, business interruption, lost profits, misdiagnosis, third-party claims and legal costs. Where liability cannot be excluded, it is limited to the amount the user actually paid for the software; for a free copy, that amount is zero.

Users are responsible for backing up data, verifying authorization to inspect the target device, complying with applicable laws and confidentiality obligations, reviewing snapshots and logs before sharing, and validating external tools such as `smartctl`, `nvidia-smi` and `system_profiler`.

Governing law: the laws of mainland China. Contact: **yanruoxinsu2@outlook.com**.

---

**Copyright © 2026 Device Viewer 原作者. All rights reserved.**
