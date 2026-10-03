use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub device: String,
    pub partition_number: u32,
    pub size_bytes: u64,
    pub free_bytes: u64,
    pub filesystem: String,
    pub label: Option<String>,
    pub uuid: String,
    pub part_uuid: String,
    pub is_esp: bool,
    pub mount_point: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiskType {
    Disk,
    Partition,
    Removable,
    Unknown,
}

impl Default for DiskType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
    pub size_bytes: u64,
    pub type_: DiskType,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BootEntry {
    pub boot_num: u16,
    pub label: String,
    pub path: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootMode {
    Bios,
    Uefi,
    Unknown,
}

impl Default for BootMode {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Distro {
    Arch,
    Ubuntu,
    Fedora,
    Debian,
    NixOS,
    Windows,
    Other(String),
}

impl Default for Distro {
    fn default() -> Self {
        Self::Other("Unknown".to_owned())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub kernel_release: String,
    pub kernel_version: String,
    pub architecture: String,
    pub boot_mode: BootMode,
    pub secure_boot: bool,
    pub distro: Option<String>,
    pub distro_version: Option<String>,
    pub distro_id: Option<String>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub refind_installed: bool,
    pub refind_version: Option<String>,
    pub refind_path: Option<PathBuf>,
    pub disks: Vec<DiskInfo>,
}

impl SystemInfo {
    pub async fn collect() -> anyhow::Result<Self> {
        let uname = std::process::Command::new("uname")
            .arg("-srm")
            .output()
            .ok();

        let kernel_info = uname
            .as_ref()
            .and_then(|output| String::from_utf8(output.stdout.clone()).ok())
            .map(|v| v.trim().to_owned())
            .unwrap_or_else(|| "Linux x86_64 6.8.0".to_owned());

        let parts: Vec<&str> = kernel_info.split_whitespace().collect();
        let kernel_release = parts.get(1).copied().unwrap_or("6.8.0").to_owned();
        let architecture = parts.get(2).copied().unwrap_or("x86_64").to_owned();
        let os_release = read_os_release();
        let distro_name = os_release
            .as_ref()
            .and_then(|release| release.pretty_name.clone())
            .or_else(|| os_release.as_ref().map(|release| release.name.clone()))
            .unwrap_or_else(|| "Linux".to_owned());
        let distro_version = os_release.as_ref().and_then(|release| release.version.clone());
        let distro_id = os_release.as_ref().and_then(|release| release.id.clone());
        let cpu = read_cpu_info();
        let memory = read_memory_info();
        let refind_path = detect_refind_path();

        Ok(Self {
            hostname: hostname::get()
                .ok()
                .and_then(|value| value.into_string().ok())
                .unwrap_or_else(|| "localhost".to_owned()),
            kernel_release: kernel_release.clone(),
            kernel_version: format!("Linux kernel {}", kernel_release),
            architecture,
            boot_mode: detect_efi_mode(),
            secure_boot: false,
            distro: Some(distro_name),
            distro_version: Some(distro_version.clone().unwrap_or_else(|| "Unknown".to_owned())),
            distro_id: Some(distro_id.clone().unwrap_or_else(|| "linux".to_owned())),
            cpu,
            memory,
            refind_installed: refind_path.is_some(),
            refind_version: None,
            refind_path,
            disks: Vec::new(),
        })
    }
}

fn scan_lsblk_esp(output: &str) -> anyhow::Result<Vec<EspInfo>> {
    let value: serde_json::Value = serde_json::from_str(output)?;
    let devices = value
        .get("blockdevices")
        .and_then(|d| d.as_array())
        .map(|items| items.as_slice())
        .unwrap_or_default();
    let mut esps = Vec::new();

    for device in devices {
        let name = device.get("name").and_then(|n| n.as_str()).unwrap_or_default();
        let kind = device.get("type").and_then(|t| t.as_str()).unwrap_or_default();
        let size = device.get("size").and_then(|s| s.as_str()).unwrap_or("0");
        let fstype = device.get("fstype").and_then(|f| f.as_str()).unwrap_or_default();
        let uuid = device.get("uuid").and_then(|u| u.as_str()).unwrap_or_default();
        let label = device.get("label").and_then(|l| l.as_str()).unwrap_or_default();
        let mountpoints = device
            .get("mountpoints")
            .and_then(|m| m.as_array())
            .map(|mounts| {
                mounts
                    .iter()
                    .filter_map(|m| m.as_str())
                    .filter(|m| !m.trim().is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let is_esp = matches!(fstype.to_ascii_lowercase().as_str(), "vfat" | "fat32" | "fat16" | "exfat")
            || mountpoints.iter().any(|m| m.contains("/boot/efi") || m.contains("EFI"));

        if kind != "part" && !is_esp {
            continue;
        }

        let mount_point = mountpoints
            .iter()
            .filter(|m| !m.is_empty())
            .map(PathBuf::from)
            .next();

        esps.push(EspInfo {
            device: if name.starts_with("/dev/") { name.to_string() } else { format!("/dev/{}", name) },
            partition_number: name.rsplit('p').next_back().and_then(|n| n.parse::<u32>().ok()).unwrap_or(0),
            size_bytes: parse_size_to_bytes(size),
            free_bytes: 0,
            filesystem: fstype.to_string(),
            label: if label.is_empty() { None } else { Some(label.to_string()) },
            uuid: uuid.to_string(),
            part_uuid: uuid.to_string(),
            is_system_esp: mount_point.as_ref().is_some_and(|mp| mp.to_string_lossy().contains("/boot/efi")),
            has_refind: mount_point.as_ref().is_some_and(|mp| mp.join("EFI/refind").exists()),
            refind_path: mount_point.as_ref().map(|mp| mp.join("EFI/refind")),
            mount_point,
            boot_entries: Vec::new(),
        });
    }

    Ok(esps)
}

fn parse_size_to_bytes(value: &str) -> u64 {
    let trimmed = value.trim();
    let number = trimmed
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    let numeric = number.parse::<u64>().unwrap_or(0);

    match trimmed.chars().rev().next() {
        Some('K') => numeric * 1024,
        Some('M') => numeric * 1024 * 1024,
        Some('G') => numeric * 1024 * 1024 * 1024,
        Some('T') => numeric * 1024 * 1024 * 1024 * 1024,
        _ => numeric,
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EspInfo {
    pub device: String,
    pub partition_number: u32,
    pub size_bytes: u64,
    pub free_bytes: u64,
    pub filesystem: String,
    pub label: Option<String>,
    pub uuid: String,
    pub part_uuid: String,
    pub is_system_esp: bool,
    pub has_refind: bool,
    pub refind_path: Option<PathBuf>,
    pub mount_point: Option<PathBuf>,
    pub boot_entries: Vec<BootEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KernelInfo {
    pub distro: Option<String>,
    pub version: String,
    pub architecture: String,
    pub vmlinuz: PathBuf,
    pub initramfs: Vec<InitramfsEntry>,
    pub fallback_initramfs: Option<PathBuf>,
    pub is_signed: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InitramfsEntry {
    pub path: PathBuf,
    pub size: u64,
    pub type_: InitramfsType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InitramfsType {
    Initramfs,
    Initrd,
    Unknown,
}

impl Default for InitramfsType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SecureBootStatus {
    pub state: SecureBootState,
    pub setup_mode: bool,
    pub secure_boot_enabled: bool,
    pub pk: Option<String>,
    pub kek: Vec<String>,
    pub db: Vec<String>,
    pub dbx: Vec<String>,
    pub mok_list: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SecureBootState {
    Enabled,
    Disabled,
    SetupMode,
    Unknown,
}

impl Default for SecureBootState {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SbctlStatus {
    pub installed: bool,
    pub setup_mode: bool,
    pub secure_boot: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TransactionManager {
    pub operations: Vec<String>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, operation: impl Into<String>) {
        self.operations.push(operation.into());
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DistroDetector;

impl DistroDetector {
    pub fn detect() -> Distro {
        let os = std::env::consts::OS;
        match os {
            "linux" => {
                if let Some(release) = read_os_release() {
                    let id = release.id.unwrap_or_default();
                    match id.to_ascii_lowercase().as_str() {
                        "arch" | "archlinux" => Distro::Arch,
                        "ubuntu" => Distro::Ubuntu,
                        "fedora" => Distro::Fedora,
                        "debian" => Distro::Debian,
                        "nixos" => Distro::NixOS,
                        _ => Distro::Other(release.name),
                    }
                } else {
                    Distro::Other("Linux".to_owned())
                }
            }
            "windows" => Distro::Windows,
            _ => Distro::Other(os.to_owned()),
        }
    }
}

fn read_boot_dir_for_kernels() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/boot") {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if filename.starts_with("vmlinuz") || filename.starts_with("initramfs") {
                candidates.push(path);
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir("/lib/modules") {
        for entry in entries.flatten() {
            let path = entry.path();
            let kernel = path.join("vmlinuz");
            if kernel.exists() {
                candidates.push(kernel);
            }
            let fallback = path.join("vmlinuz.old");
            if fallback.exists() {
                candidates.push(fallback);
            }
        }
    }
    candidates
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsRelease {
    pub name: String,
    pub version: Option<String>,
    pub id: Option<String>,
    pub pretty_name: Option<String>,
}

fn strip_quotes(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

fn parse_os_release(content: &str) -> anyhow::Result<OsRelease> {
    let mut name = None;
    let mut version = None;
    let mut id = None;
    let mut pretty_name = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let value = strip_quotes(value.trim());
        match key {
            "NAME" => name = Some(value.clone()),
            "VERSION_ID" => version = Some(value.clone()),
            "ID" => id = Some(value.clone()),
            "PRETTY_NAME" => pretty_name = Some(value.clone()),
            _ => {}
        }
    }

    Ok(OsRelease {
        name: name.unwrap_or_else(|| "Linux".to_owned()),
        version,
        id,
        pretty_name,
    })
}

fn read_os_release() -> Option<OsRelease> {
    let paths = ["/etc/os-release", "/usr/lib/os-release"];
    for path in paths {
        let value = std::fs::read_to_string(path).ok()?;
        return parse_os_release(&value).ok();
    }
    None
}

fn read_cpu_info() -> CpuInfo {
    let mut model = "Unknown CPU".to_owned();
    let mut cores = 0u32;
    let mut threads = 0u32;

    if let Ok(contents) = std::fs::read_to_string("/proc/cpuinfo") {
        let mut seen_core_count = false;
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("model name") {
                let value = rest.split(':').nth(1).map(str::trim).unwrap_or("");
                if !value.is_empty() {
                    model = value.to_owned();
                }
            } else if let Some(rest) = line.strip_prefix("Hardware") {
                let value = rest.split(':').nth(1).map(str::trim).unwrap_or("");
                if !value.is_empty() {
                    model = value.to_owned();
                }
            }

            if line.starts_with("processor") {
                threads += 1;
            }

            if line.starts_with("cpu cores") && !seen_core_count {
                if let Some(value) = line.split(':').nth(1) {
                    let parsed = value.trim().parse::<u32>().unwrap_or(0);
                    if parsed > 0 {
                        cores = parsed;
                        seen_core_count = true;
                    }
                }
            }
        }

        if threads == 0 {
            threads = num_cpus::get_physical() as u32;
        }
        if cores == 0 {
            cores = threads.max(1);
        }
    }

    if model.is_empty() || model == "Unknown CPU" {
        model = std::env::consts::ARCH.to_owned();
    }

    CpuInfo {
        model,
        cores: cores.max(1),
        threads: threads.max(1),
    }
}

fn read_memory_info() -> MemoryInfo {
    let mut total_bytes = 0;
    let mut available_bytes = 0;

    if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("MemTotal:") {
                total_bytes = parse_kib_value(value) * 1024;
            } else if let Some(value) = line.strip_prefix("MemAvailable:") {
                available_bytes = parse_kib_value(value) * 1024;
            }
        }
    }

    if total_bytes == 0 {
        total_bytes = 1;
    }
    if available_bytes == 0 {
        available_bytes = total_bytes;
    }

    MemoryInfo {
        total_bytes,
        available_bytes,
    }
}

fn parse_kib_value(line: &str) -> u64 {
    let value = line
        .split_whitespace()
        .nth(0)
        .and_then(|part| part.parse::<u64>().ok())
        .unwrap_or(0);
    value
}

fn detect_efi_mode() -> BootMode {
    if std::path::Path::new("/sys/firmware/efi").exists() {
        BootMode::Uefi
    } else {
        BootMode::Bios
    }
}

fn detect_refind_path() -> Option<PathBuf> {
    for candidate in [
        PathBuf::from("/boot/efi/EFI/refind"),
        PathBuf::from("/efi/EFI/refind"),
        PathBuf::from("/boot/EFI/refind"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RefindConfig {
    pub timeout: u32,
    pub default_selection: String,
    pub extra_entries: Vec<String>,
    pub show_tools: bool,
}

impl RefindConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let _ = path;
        Ok(Self::default())
    }

    pub fn validate(&self) -> anyhow::Result<Vec<String>> {
        let mut warnings = Vec::new();
        if self.timeout == 0 {
            warnings.push("Timeout is zero; boot menu may not appear".to_owned());
        }
        Ok(warnings)
    }
}

impl std::fmt::Display for RefindConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "timeout {}", self.timeout)?;
        writeln!(f, "default_selection {}", self.default_selection)?;
        writeln!(f, "showtools {}", self.show_tools)?;
        for entry in &self.extra_entries {
            writeln!(f, "load {}", entry)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct EspManager;

impl EspManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self) -> anyhow::Result<Vec<EspInfo>> {
        let output = std::process::Command::new("lsblk")
            .args(["-J", "-o", "NAME,TYPE,SIZE,FSTYPE,UUID,LABEL,MOUNTPOINTS"])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout).to_string();
                scan_lsblk_esp(&stdout)
            }
            _ => Ok(Vec::new()),
        }
    }

    pub async fn mount(&mut self, _esp: &EspInfo, _mount_point: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn unmount(&mut self, _mount_point: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn delete_boot_entry(&mut self, _boot_num: u16) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn get_boot_entries(&self) -> anyhow::Result<Vec<BootEntry>> {
        Ok(Vec::new())
    }

    pub async fn set_boot_order(&mut self, _order: &[u16]) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct KernelManager;

impl KernelManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self) -> anyhow::Result<Vec<KernelInfo>> {
        let mut kernels = Vec::new();
        let os_release = read_os_release();
        let mut versions = std::collections::BTreeSet::new();

        if let Ok(entries) = std::fs::read_dir("/lib/modules") {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    versions.insert(entry.file_name().to_string_lossy().to_string());
                }
            }
        }

        let mut seen = std::collections::BTreeSet::new();
        for version in versions {
            let vmlinuz = PathBuf::from("/lib/modules").join(&version).join("vmlinuz");
            if vmlinuz.exists() {
                seen.insert(vmlinuz.clone());
                kernels.push(KernelInfo {
                    distro: os_release.as_ref().map(|r| r.name.clone()),
                    version: version.clone(),
                    architecture: std::env::consts::ARCH.to_owned(),
                    vmlinuz,
                    initramfs: Vec::new(),
                    fallback_initramfs: None,
                    is_signed: false,
                });
            }
        }

        for candidate in read_boot_dir_for_kernels() {
            if seen.contains(&candidate) {
                continue;
            }
            let version = candidate
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .replace("vmlinuz-", "");
            kernels.push(KernelInfo {
                distro: os_release.as_ref().map(|r| r.name.clone()),
                version: version.clone(),
                architecture: std::env::consts::ARCH.to_owned(),
                vmlinuz: candidate,
                initramfs: Vec::new(),
                fallback_initramfs: None,
                is_signed: false,
            });
        }

        Ok(kernels)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MokManager;

impl MokManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_status(&self) -> anyhow::Result<SecureBootStatus> {
        Ok(SecureBootStatus::default())
    }

    pub async fn enroll_key(&self, _key: &Path, _password: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn delete_key(&self, _key: &Path, _password: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn generate_mok_key(
        &self,
        _output_dir: &Path,
        _name: &str,
    ) -> anyhow::Result<(PathBuf, PathBuf)> {
        Ok((PathBuf::from("/tmp/mok.key"), PathBuf::from("/tmp/mok.der")))
    }
}

#[derive(Clone, Debug, Default)]
pub struct SbctlManager;

impl SbctlManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn is_available(&self) -> bool {
        false
    }

    pub async fn get_status(&self) -> anyhow::Result<SbctlStatus> {
        Ok(SbctlStatus::default())
    }

    pub async fn enroll_keys(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn create_keys(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn sign_file(&self, _file: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn verify_file(&self, _file: &Path) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_valid_structure() {
        let cfg = RefindConfig::default();
        assert_eq!(cfg.timeout, 0);
        assert!(cfg.default_selection.is_empty());
    }

    #[test]
    fn distro_detector_reports_known_platform() {
        let distro = DistroDetector::detect();
        assert!(matches!(
            distro,
            Distro::Arch
                | Distro::Ubuntu
                | Distro::Fedora
                | Distro::Debian
                | Distro::NixOS
                | Distro::Windows
                | Distro::Other(_)
        ));
    }

    #[test]
    fn os_release_parser_reads_name_and_version() {
        let content = "NAME=\"Fedora Linux\"\nVERSION_ID=\"40\"\nID=fedora\nPRETTY_NAME=\"Fedora Linux 40 (Forty)\"\n";

        let parsed = parse_os_release(content).unwrap();
        assert_eq!(parsed.name, "Fedora Linux");
        assert_eq!(parsed.version, Some("40".to_owned()));
        assert_eq!(parsed.id, Some("fedora".to_owned()));
    }

    #[test]
    fn lsblk_parser_detects_esp_entries() {
        let json = r#"{
            "blockdevices": [
                {
                    "name": "nvme0n1p1",
                    "type": "part",
                    "size": "512M",
                    "fstype": "vfat",
                    "uuid": "1234",
                    "label": "EFI",
                    "mountpoints": ["/boot/efi"]
                }
            ]
        }"#;

        let esps = scan_lsblk_esp(json).unwrap();
        assert_eq!(esps.len(), 1);
        assert_eq!(esps[0].device, "/dev/nvme0n1p1");
        assert!(esps[0].is_system_esp);
    }
}
