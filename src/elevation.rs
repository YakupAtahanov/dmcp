//! Polkit/pkexec elevation for system-scope operations.

use std::path::Path;
use std::process;

/// Returns true if the current process is running as root (e.g. via pkexec).
pub fn is_elevated() -> bool {
    nix::unistd::Uid::current().is_root()
}

/// Returns true if the path is under the system install directory.
pub fn is_system_scope(path: &Path, system_install_dir: &Path) -> bool {
    path.starts_with(system_install_dir)
}

/// Re-execute the current binary with pkexec for elevation.
/// Passes through all current args. Exits with the child's exit code.
pub fn re_exec_with_pkexec() -> ! {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: cannot get executable path: {}", e);
            process::exit(1);
        }
    };

    let args: Vec<String> = std::env::args().skip(1).collect();

    let status = process::Command::new("pkexec")
        .arg(&exe)
        .args(&args)
        .status();

    match status {
        Ok(s) => process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("Error: pkexec failed: {}", e);
            eprintln!("Make sure polkit is installed. You can also try: sudo dmcp ...");
            process::exit(1);
        }
    }
}
