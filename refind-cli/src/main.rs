use clap::{Parser, Subcommand};
use refind_core::{EspManager, KernelManager, MokManager, SbctlManager, SystemInfo, DistroDetector, Distro, RefindConfig, TransactionManager};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "refind-cli")]
#[command(about = "rEFInd CLI - Command line interface for rEFInd management", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan and display system information
    Info,

    /// Manage ESP partitions
    Esp {
        #[command(subcommand)]
        command: EspCommands,
    },

    /// Manage boot entries
    Entry {
        #[command(subcommand)]
        command: EntryCommands,
    },

    /// Manage kernels
    Kernel {
        #[command(subcommand)]
        command: KernelCommands,
    },

    /// Manage themes
    Theme {
        #[command(subcommand)]
        command: ThemeCommands,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Manage Secure Boot
    SecureBoot {
        #[command(subcommand)]
        command: SecureBootCommands,
    },

    /// Install rEFInd
    Install {
        #[arg(short, long)]
        esp: Option<String>,

        #[arg(short, long, default_value = "x64")]
        arch: String,

        #[arg(short, long)]
        source: Option<PathBuf>,

        #[arg(short, long)]
        drivers: Vec<String>,

        #[arg(long)]
        no_boot_entry: bool,
    },

    /// Backup configuration
    Backup {
        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        config_only: bool,

        #[arg(long)]
        esp_only: bool,
    },

    /// Restore configuration
    Restore {
        #[arg(short, long)]
        backup: PathBuf,

        #[arg(long)]
        config_only: bool,

        #[arg(long)]
        esp_only: bool,
    },
}

#[derive(Subcommand)]
enum EspCommands {
    /// List ESP partitions
    List,

    /// Show ESP details
    Show {
        device: String,
        partition: u32,
    },

    /// Mount ESP
    Mount {
        device: String,
        partition: u32,
        #[arg(short, long)]
        mount_point: Option<PathBuf>,
    },

    /// Unmount ESP
    Unmount {
        mount_point: PathBuf,
    },
}

#[derive(Subcommand)]
enum EntryCommands {
    /// List boot entries
    List,

    /// Add boot entry
    Add {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        loader: String,

        #[arg(short, long)]
        initrd: Option<String>,

        #[arg(short, long)]
        options: Option<String>,

        #[arg(short, long)]
        icon: Option<String>,

        #[arg(short, long)]
        esp: Option<String>,
    },

    /// Remove boot entry
    Remove {
        #[arg(short, long)]
        boot_num: u16,
    },

    /// Set default boot entry
    SetDefault {
        #[arg(short, long)]
        boot_num: u16,
    },
}

#[derive(Subcommand)]
enum KernelCommands {
    /// List detected kernels
    List,

    /// Scan for kernels
    Scan,

    /// Generate rEFInd entries from kernels
    GenerateEntries {
        #[arg(short, long)]
        esp: Option<String>,
    },
}

#[derive(Subcommand)]
enum ThemeCommands {
    /// List themes
    List,

    /// Install theme
    Install {
        source: PathBuf,
    },

    /// Remove theme
    Remove {
        name: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current config
    Show,

    /// Validate config
    Validate,

    /// Edit config
    Edit,
}

#[derive(Subcommand)]
enum SecureBootCommands {
    /// Show status
    Status,

    /// Enroll key
    Enroll {
        key: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Remove key
    Remove {
        key: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Generate MOK key
    GenerateKey {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long, default_value = "RSA2048")]
        algorithm: String,
    },

    /// sbctl commands
    Sbctl {
        #[command(subcommand)]
        command: SbctlCommands,
    },
}

#[derive(Subcommand)]
enum SbctlCommands {
    Status,
    EnrollKeys,
    CreateKeys,
    Sign {
        file: PathBuf,
    },
    Verify {
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    }

    match cli.command {
        Commands::Info => cmd_info().await?,
        Commands::Esp { command } => cmd_esp(command).await?,
        Commands::Entry { command } => cmd_entry(command).await?,
        Commands::Kernel { command } => cmd_kernel(command).await?,
        Commands::Theme { command } => cmd_theme(command).await?,
        Commands::Config { command } => cmd_config(command).await?,
        Commands::SecureBoot { command } => cmd_secure_boot(command).await?,
        Commands::Install { esp, arch, source, drivers, no_boot_entry } => {
            cmd_install(esp, arch, source, drivers, no_boot_entry).await?
        }
        Commands::Backup { output, config_only, esp_only } => {
            cmd_backup(output, config_only, esp_only).await?
        }
        Commands::Restore { backup, config_only, esp_only } => {
            cmd_restore(backup, config_only, esp_only).await?
        }
    }

    Ok(())
}

async fn cmd_info() -> anyhow::Result<()> {
    println!("Scanning system...");
    let info = SystemInfo::collect().await?;

    println!("\n=== System Information ===");
    println!("Hostname: {}", info.hostname);
    println!("Kernel: {} ({})", info.kernel_release, info.kernel_version);
    println!("Architecture: {}", info.architecture);
    println!("Boot Mode: {:?}", info.boot_mode);
    println!("Secure Boot: {:?}", info.secure_boot);
    println!("Distro: {} {} ({})",
        info.distro.as_deref().unwrap_or("Unknown"),
        info.distro_version.as_deref().unwrap_or(""),
        info.distro_id.as_deref().unwrap_or("")
    );
    println!("CPU: {} ({} cores, {} threads)", info.cpu.model, info.cpu.cores, info.cpu.threads);
    println!("Memory: {:.1} GB / {:.1} GB",
        info.memory.available_bytes as f64 / 1e9,
        info.memory.total_bytes as f64 / 1e9
    );
    println!("rEFInd Installed: {}", info.refind_installed);
    if let Some(v) = &info.refind_version {
        println!("rEFInd Version: {}", v);
    }
    if let Some(p) = &info.refind_path {
        println!("rEFInd Path: {}", p.display());
    }

    println!("\n=== Disks ===");
    for disk in &info.disks {
        println!("  {}: {} ({:.1} GB) [{:?}]", disk.device, disk.model, disk.size_bytes as f64 / 1e9, disk.type_);
        for part in &disk.partitions {
            let esp_marker = if part.is_esp { " [ESP]" } else { "" };
            println!("    {}: {:.1} GB {} {} {}",
                part.device,
                part.size_bytes as f64 / 1e9,
                part.filesystem,
                part.label.as_deref().unwrap_or(""),
                esp_marker
            );
        }
    }

    Ok(())
}

async fn cmd_esp(command: EspCommands) -> anyhow::Result<()> {
    let mut manager = EspManager::new();
    let esps = manager.scan().await?;

    match command {
        EspCommands::List => {
            println!("=== ESP Partitions ===");
            for esp in &esps {
                println!("  {}p{} - {:.1} MB free / {:.1} MB total {} {}",
                    esp.device, esp.partition_number,
                    esp.free_bytes as f64 / 1e6,
                    esp.size_bytes as f64 / 1e6,
                    if esp.is_system_esp { "[SYSTEM]" } else { "" },
                    if esp.has_refind { "[REFIND]" } else { "" }
                );
            }
        }
        EspCommands::Show { device, partition } => {
            if let Some(esp) = esps.iter().find(|e| e.device == device && e.partition_number == partition) {
                println!("Device: {}", esp.device);
                println!("Partition: {}", esp.partition_number);
                println!("Size: {:.1} MB", esp.size_bytes as f64 / 1e6);
                println!("Free: {:.1} MB", esp.free_bytes as f64 / 1e6);
                println!("Filesystem: {}", esp.filesystem);
                println!("Label: {}", esp.label.as_deref().unwrap_or("none"));
                println!("UUID: {}", esp.uuid);
                println!("PARTUUID: {}", esp.part_uuid);
                println!("System ESP: {}", esp.is_system_esp);
                println!("Has rEFInd: {}", esp.has_refind);
                if let Some(p) = &esp.refind_path {
                    println!("rEFInd Path: {}", p.display());
                }
                println!("Mount Point: {}", esp.mount_point.as_ref().map(|p| p.display().to_string()).unwrap_or("none".to_string()));
            } else {
                println!("ESP not found");
            }
        }
        EspCommands::Mount { device, partition, mount_point } => {
            let esp = esps.iter().find(|e| e.device == device && e.partition_number == partition)
                .ok_or_else(|| anyhow::anyhow!("ESP not found"))?;
            let mp = mount_point.unwrap_or_else(|| PathBuf::from(format!("/mnt/esp_{}_{}", device.replace('/', "_"), partition)));
            manager.mount(esp, &mp).await?;
            println!("Mounted {}p{} at {}", device, partition, mp.display());
        }
        EspCommands::Unmount { mount_point } => {
            manager.unmount(&mount_point).await?;
            println!("Unmounted {}", mount_point.display());
        }
    }

    Ok(())
}

async fn cmd_entry(command: EntryCommands) -> anyhow::Result<()> {
    let mut manager = EspManager::new();
    let esps = manager.scan().await?;

    match command {
        EntryCommands::List => {
            println!("=== Boot Entries ===");
            for esp in &esps {
                if !esp.boot_entries.is_empty() {
                    println!("\nESP: {}p{}", esp.device, esp.partition_number);
                    for entry in &esp.boot_entries {
                        println!("  Boot{:04X}: {} -> {} {}",
                            entry.boot_num, entry.label, entry.path,
                            if entry.is_active { "*" } else { "" }
                        );
                    }
                }
            }
        }
        EntryCommands::Add { name, loader, initrd, options, icon, esp: _ } => {
            println!("Adding boot entry: {}", name);
            println!("  Loader: {}", loader);
            if let Some(i) = initrd { println!("  Initrd: {}", i); }
            if let Some(o) = options { println!("  Options: {}", o); }
            if let Some(i) = icon { println!("  Icon: {}", i); }
        }
        EntryCommands::Remove { boot_num } => {
            manager.delete_boot_entry(boot_num).await?;
            println!("Deleted boot entry Boot{:04X}", boot_num);
        }
        EntryCommands::SetDefault { boot_num } => {
            let entries = manager.get_boot_entries().await?;
            let mut order: Vec<u16> = entries.iter().map(|e| e.boot_num).collect();
            if let Some(pos) = order.iter().position(|&n| n == boot_num) {
                order.remove(pos);
                order.insert(0, boot_num);
                manager.set_boot_order(&order).await?;
                println!("Set Boot{:04X} as default", boot_num);
            }
        }
    }

    Ok(())
}

async fn cmd_kernel(command: KernelCommands) -> anyhow::Result<()> {
    let mut manager = KernelManager::new();

    match command {
        KernelCommands::List => {
            let kernels = manager.scan().await?;
            println!("=== Detected Kernels ===");
            for kernel in &kernels {
                println!("  {} {} ({})",
                    kernel.distro.as_deref().unwrap_or("Linux"),
                    kernel.version,
                    kernel.architecture
                );
                println!("    vmlinuz: {}", kernel.vmlinuz.display());
                for initrd in &kernel.initramfs {
                    println!("    initrd: {} ({}, {:.1} MB)",
                        initrd.path.display(),
                        format!("{:?}", initrd.type_).to_lowercase(),
                        initrd.size as f64 / 1e6
                    );
                }
                if let Some(fb) = &kernel.fallback_initramfs {
                    println!("    fallback: {}", fb.display());
                }
                if kernel.is_signed {
                    println!("    [SIGNED]");
                }
            }
        }
        KernelCommands::Scan => {
            let kernels = manager.scan().await?;
            println!("Found {} kernel(s)", kernels.len());
        }
        KernelCommands::GenerateEntries { esp } => {
            println!("Generating rEFInd entries from kernels...");
        }
    }

    Ok(())
}

async fn cmd_theme(command: ThemeCommands) -> anyhow::Result<()> {
    match command {
        ThemeCommands::List => {
            println!("=== Themes ===");
        }
        ThemeCommands::Install { source } => {
            println!("Installing theme from {}", source.display());
        }
        ThemeCommands::Remove { name } => {
            println!("Removing theme: {}", name);
        }
    }
    Ok(())
}

async fn cmd_config(command: ConfigCommands) -> anyhow::Result<()> {
    let paths = [
        "/etc/refind.conf",
        "/boot/efi/EFI/refind/refind.conf",
        "/efi/EFI/refind/refind.conf",
    ];

    let config = paths.iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|p| RefindConfig::load(std::path::Path::new(p)))
        .transpose()?
        .unwrap_or_else(RefindConfig::default);

    match command {
        ConfigCommands::Show => {
            println!("{}", config.to_string());
        }
        ConfigCommands::Validate => {
            let warnings = config.validate()?;
            if warnings.is_empty() {
                println!("Config is valid");
            } else {
                println!("Warnings:");
                for w in warnings {
                    println!("  - {}", w);
                }
            }
        }
        ConfigCommands::Edit => {
            println!("Opening editor... (not implemented)");
        }
    }

    Ok(())
}

async fn cmd_secure_boot(command: SecureBootCommands) -> anyhow::Result<()> {
    match command {
        SecureBootCommands::Status => {
            let manager = MokManager::new();
            let status = manager.get_status().await?;

            println!("=== Secure Boot Status ===");
            println!("State: {:?}", status.state);
            println!("Setup Mode: {}", status.setup_mode);
            println!("Secure Boot Enabled: {}", status.secure_boot_enabled);
            println!("PK: {}", if status.pk.is_some() { "Enrolled" } else { "Not enrolled" });
            println!("KEK Count: {}", status.kek.len());
            println!("db Count: {}", status.db.len());
            println!("dbx Count: {}", status.dbx.len());
            println!("MOK Count: {}", status.mok_list.len());
        }
        SecureBootCommands::Enroll { key, password } => {
            let manager = MokManager::new();
            let pass = password.unwrap_or_else(|| {
                rpassword::prompt_password("MOK Password: ").unwrap()
            });
            manager.enroll_key(&key, &pass).await?;
            println!("Key enrolled successfully");
        }
        SecureBootCommands::Remove { key, password } => {
            let manager = MokManager::new();
            let pass = password.unwrap_or_else(|| {
                rpassword::prompt_password("MOK Password: ").unwrap()
            });
            manager.delete_key(&key, &pass).await?;
            println!("Key removed successfully");
        }
        SecureBootCommands::GenerateKey { name, output, algorithm } => {
            let manager = MokManager::new();
            let (key_path, der_path) = manager.generate_mok_key(&output, &name).await?;
            println!("Generated key pair:");
            println!("  Private: {}", key_path.display());
            println!("  Public (DER): {}", der_path.display());
        }
        SecureBootCommands::Sbctl { command } => {
            let manager = SbctlManager::new();

            match command {
                SbctlCommands::Status => {
                    if manager.is_available().await {
                        let status = manager.get_status().await?;
                        println!("sbctl Status:");
                        println!("  Installed: {}", status.installed);
                        println!("  Setup Mode: {}", status.setup_mode);
                        println!("  Secure Boot: {}", status.secure_boot);
                    } else {
                        println!("sbctl not available");
                    }
                }
                SbctlCommands::EnrollKeys => {
                    manager.enroll_keys().await?;
                    println!("Keys enrolled with sbctl");
                }
                SbctlCommands::CreateKeys => {
                    manager.create_keys().await?;
                    println!("Keys created with sbctl");
                }
                SbctlCommands::Sign { file } => {
                    manager.sign_file(&file).await?;
                    println!("Signed: {}", file.display());
                }
                SbctlCommands::Verify { file } => {
                    let ok = manager.verify_file(&file).await?;
                    println!("Verification: {}", if ok { "PASSED" } else { "FAILED" });
                }
            }
        }
    }

    Ok(())
}

async fn cmd_install(
    esp: Option<String>,
    arch: String,
    source: Option<PathBuf>,
    drivers: Vec<String>,
    no_boot_entry: bool,
) -> anyhow::Result<()> {
    println!("Installing rEFInd ({})...", arch);
    println!("Source: {}", source.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "/usr/share/refind (default)".to_string()));
    println!("Drivers: {:?}", drivers);
    println!("Create boot entry: {}", !no_boot_entry);
    Ok(())
}

async fn cmd_backup(
    output: Option<PathBuf>,
    config_only: bool,
    esp_only: bool,
) -> anyhow::Result<()> {
    println!("Creating backup...");
    Ok(())
}

async fn cmd_restore(
    backup: PathBuf,
    config_only: bool,
    esp_only: bool,
) -> anyhow::Result<()> {
    println!("Restoring from {}", backup.display());
    Ok(())
}
