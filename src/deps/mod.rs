pub mod node;
pub mod version;

use anyhow::Result;
use std::fmt;

pub use node::NodeChecker;
pub use version::{VersionHelper, VersionRequirement};

#[derive(Debug, Clone)]
pub struct DependencyCheck {
    pub dependency: Dependency,
    pub status: DependencyStatus,
    pub install_instructions: Option<InstallInstructions>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Dependency {
    NodeJs { min_version: Option<String> },
    Python { min_version: Option<String> },
    Docker,
    Git,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyStatus {
    Installed { version: Option<String> },
    Missing,
    VersionMismatch { installed: String, required: String },
}

#[derive(Debug, Clone, Default)]
pub struct InstallInstructions {
    pub windows: Vec<InstallMethod>,
    pub macos: Vec<InstallMethod>,
    pub linux: Vec<InstallMethod>,
}

#[derive(Debug, Clone)]
pub struct InstallMethod {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

pub trait DependencyChecker: Send + Sync {
    fn check(&self) -> Result<DependencyCheck>;
}

impl Dependency {
    pub fn name(&self) -> &str {
        match self {
            Dependency::NodeJs { .. } => "Node.js",
            Dependency::Python { .. } => "Python",
            Dependency::Docker => "Docker",
            Dependency::Git => "Git",
        }
    }
}

impl fmt::Display for DependencyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyStatus::Installed { version } => {
                if let Some(v) = version {
                    write!(f, "Installed ({v})")
                } else {
                    write!(f, "Installed")
                }
            }
            DependencyStatus::Missing => write!(f, "Not installed"),
            DependencyStatus::VersionMismatch {
                installed,
                required,
            } => {
                write!(
                    f,
                    "Version mismatch (installed: {installed}, required: {required})"
                )
            }
        }
    }
}

impl InstallInstructions {
    pub fn for_platform(&self) -> &[InstallMethod] {
        #[cfg(target_os = "windows")]
        return &self.windows;

        #[cfg(target_os = "macos")]
        return &self.macos;

        #[cfg(target_os = "linux")]
        return &self.linux;

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return &[];
    }
}

// Helper macro to reduce duplication when creating InstallMethod instances
macro_rules! method {
    ($name:expr, $cmd:expr, $desc:expr) => {
        InstallMethod {
            name: $name.to_string(),
            command: $cmd.to_string(),
            description: Some($desc.to_string()),
        }
    };
}

pub fn get_install_instructions(dependency: &Dependency) -> InstallInstructions {
    match dependency {
        Dependency::NodeJs { .. } => InstallInstructions {
            windows: vec![
                method!("winget", "winget install OpenJS.NodeJS", "Windows Package Manager (recommended)"),
                method!("chocolatey", "choco install nodejs", "Chocolatey package manager"),
                method!("download", "https://nodejs.org/en/download/", "Direct download from nodejs.org"),
            ],
            macos: vec![
                method!("homebrew", "brew install node", "Homebrew package manager (recommended)"),
                method!("macports", "sudo port install nodejs20", "MacPorts package manager"),
                method!("download", "https://nodejs.org/en/download/", "Direct download from nodejs.org"),
            ],
            linux: vec![
                method!("apt", "sudo apt update && sudo apt install nodejs npm", "Debian/Ubuntu (recommended)"),
                method!("dnf", "sudo dnf install nodejs npm", "Fedora/RHEL"),
                method!("snap", "sudo snap install node --classic", "Snap package"),
                method!("nvm", "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash", "Node Version Manager"),
            ],
        },
        Dependency::Python { .. } => InstallInstructions {
            windows: vec![
                method!("winget", "winget install Python.Python.3.12", "Windows Package Manager (recommended)"),
                method!("chocolatey", "choco install python", "Chocolatey package manager"),
                method!("download", "https://www.python.org/downloads/", "Direct download from python.org"),
            ],
            macos: vec![
                method!("homebrew", "brew install python@3.12", "Homebrew package manager (recommended)"),
                method!("pyenv", "pyenv install 3.12.0", "Python version manager"),
                method!("download", "https://www.python.org/downloads/", "Direct download from python.org"),
            ],
            linux: vec![
                method!("apt", "sudo apt update && sudo apt install python3 python3-pip", "Debian/Ubuntu (recommended)"),
                method!("dnf", "sudo dnf install python3 python3-pip", "Fedora/RHEL"),
                method!("pyenv", "curl https://pyenv.run | bash", "Python version manager"),
            ],
        },
        Dependency::Docker => InstallInstructions {
            windows: vec![
                method!("docker-desktop", "https://www.docker.com/products/docker-desktop/", "Docker Desktop for Windows (recommended)"),
                method!("winget", "winget install Docker.DockerDesktop", "Install via Windows Package Manager"),
            ],
            macos: vec![
                method!("docker-desktop", "https://www.docker.com/products/docker-desktop/", "Docker Desktop for Mac (recommended)"),
                method!("homebrew", "brew install --cask docker", "Install via Homebrew"),
            ],
            linux: vec![
                method!("docker-ce", "https://docs.docker.com/engine/install/", "Docker Community Edition (recommended)"),
                method!("snap", "sudo snap install docker", "Install via Snap"),
            ],
        },
        Dependency::Git => InstallInstructions {
            windows: vec![
                method!("winget", "winget install Git.Git", "Windows Package Manager (recommended)"),
                method!("chocolatey", "choco install git", "Chocolatey package manager"),
                method!("download", "https://git-scm.com/download/win", "Direct download from git-scm.com"),
            ],
            macos: vec![
                method!("xcode", "xcode-select --install", "Xcode Command Line Tools (includes Git)"),
                method!("homebrew", "brew install git", "Homebrew package manager"),
            ],
            linux: vec![
                method!("apt", "sudo apt update && sudo apt install git", "Debian/Ubuntu"),
                method!("dnf", "sudo dnf install git", "Fedora/RHEL"),
                method!("pacman", "sudo pacman -S git", "Arch Linux"),
            ],
        },
    }
}
