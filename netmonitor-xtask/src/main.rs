use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Build the eBPF program
    BuildEbpf {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build and run the project
    Run {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Run all local verifications (pre-flight)
    Verify,
    /// Fast check: Linting and Unit Tests (No accuracy script)
    Check,
    /// Install the daemon and systemd service (requires sudo)
    Install,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildEbpf { release } => {
            build_ebpf(release)?;
        }
        Commands::Run { release } => {
            run(release)?;
        }
        Commands::Verify => {
            verify()?;
        }
        Commands::Check => {
            check()?;
        }
        Commands::Install => {
            install()?;
        }
    }

    Ok(())
}

fn check() -> Result<()> {
    println!("--- 0. Building eBPF Bytecode (Required for Linting/Embedding) ---");
    build_ebpf(true)?;

    println!("--- 1. Linting (fmt & clippy) ---");
    let status = Command::new("cargo").args(["fmt", "--check"]).status()?;
    if !status.success() {
        return Err(anyhow!("fmt check failed"));
    }
    let status = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("clippy failed"));
    }

    println!("--- 2. Unit Tests (Userspace & eBPF Logic) ---");
    let status = Command::new("cargo")
        .args(["test", "--workspace"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("unit tests failed"));
    }

    println!("Check PASSED.");
    Ok(())
}

fn install() -> Result<()> {
    println!("--- 1. Building Release Binaries ---");
    build_ebpf(true)?;
    let status = Command::new("cargo")
        .args(["build", "--package", "netmonitor", "--release"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("failed to build netmonitor in release mode"));
    }

    println!("--- 2. Creating System User & Group ---");
    // Create group if not exists
    let _ = Command::new("sudo")
        .args(["groupadd", "-r", "netmonitor"])
        .status();
    // Create user if not exists
    let _ = Command::new("sudo")
        .args([
            "useradd",
            "-r",
            "-g",
            "netmonitor",
            "-d",
            "/var/lib/netmonitor",
            "-s",
            "/usr/sbin/nologin",
            "netmonitor",
        ])
        .status();

    println!("--- 3. Provisioning Directories ---");
    let dirs = [
        "/var/lib/netmonitor",
        "/var/log/netmonitor",
        "/run/netmonitor",
    ];
    for dir in dirs {
        Command::new("sudo").args(["mkdir", "-p", dir]).status()?;
        Command::new("sudo")
            .args(["chown", "netmonitor:netmonitor", dir])
            .status()?;
    }

    println!("--- 4. Installing Binary ---");
    Command::new("sudo")
        .args([
            "cp",
            "target/release/netmonitor",
            "/usr/local/bin/netmonitor",
        ])
        .status()?;
    Command::new("sudo")
        .args([
            "setcap",
            "cap_net_admin,cap_bpf=ep",
            "/usr/local/bin/netmonitor",
        ])
        .status()?;

    println!("--- 5. Deploying Systemd Service ---");
    Command::new("sudo")
        .args([
            "cp",
            "netmonitor/resources/netmonitor.service",
            "/etc/systemd/system/netmonitor.service",
        ])
        .status()?;
    Command::new("sudo")
        .args(["systemctl", "daemon-reload"])
        .status()?;
    Command::new("sudo")
        .args(["systemctl", "enable", "netmonitor"])
        .status()?;

    println!("--- 6. Installation Complete ---");
    println!("You can now start the service with: sudo systemctl start netmonitor");
    println!("Check logs with: journalctl -u netmonitor -f");

    Ok(())
}

fn verify() -> Result<()> {
    println!("--- 0. Building eBPF Bytecode (Required for Linting/Embedding) ---");
    build_ebpf(true)?;

    println!("--- 1. Linting (fmt & clippy) ---");
    let status = Command::new("cargo").args(["fmt", "--check"]).status()?;
    if !status.success() {
        return Err(anyhow!("fmt check failed"));
    }
    let status = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("clippy failed"));
    }

    println!("--- 2. Unit Tests (Userspace & eBPF Logic) ---");
    let status = Command::new("cargo")
        .args(["test", "--workspace"])
        .status()?;
    if !status.success() {
        return Err(anyhow!("unit tests failed"));
    }

    println!("--- 3. Accuracy Verification (Isolated Namespace) ---");
    // Ensure binary is built for script to use
    let status = Command::new("cargo")
        .args(["build", "--package", "netmonitor", "--release"])
        .status()?;
    if !status.success() {
        return Err(anyhow!(
            "failed to build netmonitor for accuracy verification"
        ));
    }

    // Run the accuracy script with sudo
    let status = Command::new("sudo")
        .arg("./scripts/verify_accuracy.sh")
        .status()?;
    if !status.success() {
        return Err(anyhow!("accuracy verification script failed"));
    }

    println!("--- 4. Capabilities Audit ---");
    let bin_path = "target/release/netmonitor";
    let output = Command::new("getcap").arg(bin_path).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Binary caps: {}", stdout);
    // If we want to strictly fail if no caps are set, we could check here.
    // But since xtask run uses sudo, it's often not set in dev.

    println!("Verification PASSED.");
    Ok(())
}

fn build_ebpf(_release: bool) -> Result<()> {
    // Check if bpf-linker is installed
    if Command::new("bpf-linker")
        .arg("--version")
        .status()
        .is_err()
    {
        return Err(anyhow!(
            "bpf-linker not found. Please install it with 'cargo install bpf-linker'"
        ));
    }

    let args = vec![
        "build",
        "--package",
        "netmonitor-ebpf",
        "--target",
        "bpfel-unknown-none",
        "-Z",
        "build-std=core",
        "--release",
    ];

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run cargo build for eBPF")?;

    if !status.success() {
        return Err(anyhow!("Failed to build eBPF program"));
    }

    // Copy to resources for embedding
    println!("--- Copying eBPF bytecode to resources ---");
    let src = "target/bpfel-unknown-none/release/netmonitor-ebpf";
    let dst = "netmonitor/resources/netmonitor-ebpf";
    std::fs::create_dir_all("netmonitor/resources")?;
    std::fs::copy(src, dst)?;

    Ok(())
}

fn run(release: bool) -> Result<()> {
    build_ebpf(release)?;

    let mut build_args = vec!["build", "--package", "netmonitor"];
    if release {
        build_args.push("--release");
    }

    let status = Command::new("cargo")
        .args(&build_args)
        .status()
        .context("Failed to build userspace program")?;

    if !status.success() {
        return Err(anyhow!("Failed to build userspace program"));
    }

    // Determine target directory for the userspace binary
    let profile = if release { "release" } else { "debug" };
    let bin_path = format!("target/{}/netmonitor", profile);

    // Run the userspace program with sudo
    let status = Command::new("sudo")
        .arg(bin_path)
        .status()
        .context("Failed to run userspace program with sudo")?;

    if !status.success() {
        return Err(anyhow!("Userspace program failed"));
    }

    Ok(())
}
