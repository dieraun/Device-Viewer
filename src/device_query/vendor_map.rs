//! Offline PCI/USB vendor and device name tables.
//!
//! No network access is performed. The table covers the vendors commonly found
//! in desktop, laptop and server hardware; unknown identifiers fall back to a
//! formatted hexadecimal string at the call site.

/// PCI/USB vendor identifiers shared by both buses.
pub static VENDORS: &[(u16, &str)] = &[
    (0x0010, "Broadcom"),
    (0x003D, "Master"),
    (0x0073, "SDL Communications"),
    (0x0095, "Silicon Graphics"),
    (0x00A9, "Cisco Systems"),
    (0x00E0, "Digital Equipment"),
    (0x021B, "Compaq"),
    (0x025E, "Solidigm"),
    (0x02C1, "Western Digital"),
    (0x0432, "Sun Microsystems"),
    (0x046D, "Logitech"),
    (0x04A9, "Canon"),
    (0x04B3, "IBM"),
    (0x04D9, "Holtek"),
    (0x04E8, "Samsung"),
    (0x04F2, "Chicony Electronics"),
    (0x04F3, "ELAN Microelectronics"),
    (0x054C, "Sony"),
    (0x05AC, "Apple"),
    (0x0781, "SanDisk"),
    (0x0951, "Kingston Technology"),
    (0x096E, "Feitian Technologies"),
    (0x0B05, "ASUSTek"),
    (0x0BDA, "Realtek Semiconductor"),
    (0x0BC2, "Seagate"),
    (0x0C45, "Sonix Technology"),
    (0x0E8D, "MediaTek"),
    (0x0F0D, "Hori"),
    (0x1002, "Advanced Micro Devices"),
    (0x1004, "LG Electronics"),
    (0x1014, "IBM"),
    (0x1022, "Advanced Micro Devices"),
    (0x1028, "Dell"),
    (0x103C, "Hewlett-Packard"),
    (0x1043, "ASUSTeK Computer"),
    (0x104C, "Texas Instruments"),
    (0x1058, "Western Digital"),
    (0x106B, "Apple"),
    (0x10DE, "NVIDIA"),
    (0x10EC, "Realtek Semiconductor"),
    (0x10CF, "Fujitsu"),
    (0x1130, "Tenx Technology"),
    (0x1179, "Toshiba"),
    (0x11AB, "Marvell Technology"),
    (0x1234, "Bochs/QEMU"),
    (0x13B5, "Arm"),
    (0x13FE, "Phison Electronics"),
    (0x1414, "Microsoft"),
    (0x1458, "Gigabyte Technology"),
    (0x1462, "Micro-Star International"),
    (0x14E4, "Broadcom"),
    (0x152D, "JMicron Technology"),
    (0x1532, "Razer"),
    (0x15AD, "VMware"),
    (0x168C, "Qualcomm Atheros"),
    (0x174C, "ASMedia Technology"),
    (0x17AA, "Lenovo"),
    (0x17EF, "Lenovo"),
    (0x1814, "Ralink Technology"),
    (0x1849, "ASRock"),
    (0x18D1, "Google"),
    (0x1912, "Renesas Electronics"),
    (0x1A03, "ASPEED Technology"),
    (0x1A40, "Terminus Technology"),
    (0x1AF4, "Red Hat"),
    (0x1B1C, "Corsair"),
    (0x1B21, "ASMedia Technology"),
    (0x1B36, "Red Hat"),
    (0x1B3F, "Generalplus Technology"),
    (0x1BCF, "Sunplus Technology"),
    (0x1D0F, "Amazon"),
    (0x1D49, "Lenovo"),
    (0x1D6B, "Linux Foundation"),
    (0x1DE1, "Tekram Technology"),
    (0x20A0, "Clay Logic"),
    (0x2109, "VIA Technologies"),
    (0x2717, "Xiaomi"),
    (0x5143, "Qualcomm"),
    (0x8086, "Intel"),
    (0x8087, "Intel"),
    (0x80EE, "VirtualBox"),
];

/// Known PCI device identifiers: `(vendor_id, device_id, name)`.
pub static PCI_DEVICES: &[(u16, u16, &str)] = &[
    (0x1002, 0x67DF, "Radeon RX 480"),
    (0x1002, 0x73BF, "Radeon RX 6900 XT"),
    (0x1002, 0x744C, "Radeon RX 7900 XTX"),
    (0x10DE, 0x1B80, "GeForce GTX 1080"),
    (0x10DE, 0x1B81, "GeForce GTX 1070"),
    (0x10DE, 0x2204, "GeForce RTX 3090"),
    (0x10DE, 0x2484, "GeForce RTX 3070"),
    (0x10DE, 0x2684, "GeForce RTX 4090"),
    (0x8086, 0x56A0, "Arc A770"),
    (0x8086, 0x9A49, "Iris Xe Graphics"),
    (0x1AF4, 0x1050, "Virtio GPU"),
    (0x15AD, 0x0405, "VMware SVGA II"),
];

/// Human readable PCI class names, indexed by the class code.
pub static PCI_CLASSES: &[(u8, &str)] = &[
    (0x00, "Unclassified"),
    (0x01, "Mass Storage Controller"),
    (0x02, "Network Controller"),
    (0x03, "Display Controller"),
    (0x04, "Multimedia Controller"),
    (0x05, "Memory Controller"),
    (0x06, "Bridge"),
    (0x07, "Communication Controller"),
    (0x08, "Base System Peripheral"),
    (0x09, "Input Device Controller"),
    (0x0A, "Docking Station"),
    (0x0B, "Processor"),
    (0x0C, "Serial Bus Controller"),
    (0x0D, "Wireless Controller"),
    (0x0E, "Intelligent Controller"),
    (0x0F, "Satellite Communication Controller"),
    (0x10, "Encryption Controller"),
    (0x11, "Signal Processing Controller"),
    (0x12, "Processing Accelerator"),
    (0x13, "Non-Essential Instrumentation"),
    (0x40, "Coprocessor"),
];

pub fn vendor_name(vendor_id: u16) -> Option<&'static str> {
    VENDORS
        .iter()
        .find_map(|(id, name)| (*id == vendor_id).then_some(*name))
}

/// USB and PCI share the vendor ID registry, so this is an alias with intent.
pub fn usb_vendor_name(vendor_id: u16) -> Option<&'static str> {
    vendor_name(vendor_id)
}

pub fn pci_device_name(vendor_id: u16, device_id: u16) -> Option<&'static str> {
    PCI_DEVICES
        .iter()
        .find_map(|(vid, did, name)| (*vid == vendor_id && *did == device_id).then_some(*name))
}

pub fn pci_class_name(class_code: u8) -> Option<&'static str> {
    PCI_CLASSES
        .iter()
        .find_map(|(code, name)| (*code == class_code).then_some(*name))
}

/// Returns `Vendor Device` style text with an offline fallback.
pub fn describe_pci(vendor_id: u16, device_id: u16) -> String {
    match (
        vendor_name(vendor_id),
        pci_device_name(vendor_id, device_id),
    ) {
        (Some(vendor), Some(device)) => format!("{vendor} {device}"),
        (Some(vendor), None) => format!("{vendor} [{device_id:04X}]"),
        (None, Some(device)) => format!("[{vendor_id:04X}] {device}"),
        (None, None) => format!("[{vendor_id:04X}:{device_id:04X}]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vendors_resolve() {
        assert_eq!(vendor_name(0x10DE), Some("NVIDIA"));
        assert_eq!(usb_vendor_name(0x046D), Some("Logitech"));
    }

    #[test]
    fn unknown_vendor_is_none() {
        assert_eq!(vendor_name(0xFFFE), None);
    }

    #[test]
    fn description_has_a_fallback() {
        assert_eq!(describe_pci(0xFFFE, 0x0001), "[FFFE:0001]");
    }
}
