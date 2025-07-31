pub mod base;
pub mod docker;
pub mod installer;
pub mod node;
pub mod python;
pub mod version;

use anyhow::Result;
use std::fmt;

pub use docker::DockerChecker;
pub use installer::{detect_package_managers, DependencyInstaller};
pub use node::NodeChecker;
pub use python::PythonChecker;
pub use version::{VersionHelper, VersionRequirement};

#[derive(Debug, Clone)]
pub struct DependencyCheck {
    pub dependency: Dependency,
    pub status: DependencyStatus,
    pub install_instructions: Option<InstallInstructions>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Dependency {
    NodeJs {
        min_version: Option<String>,
    },
    Python {
        min_version: Option<String>,
    },
    Docker {
        min_version: Option<String>,
        requires_compose: bool,
    },
    Git,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DependencyStatus {
    Installed { version: Option<String> },
    Missing,
    VersionMismatch { installed: String, required: String },
    ConfigurationRequired { issue: String, solution: String },
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
            Dependency::Docker { .. } => "Docker",
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
            DependencyStatus::ConfigurationRequired { issue, solution: _ } => {
                write!(f, "Configuration required: {issue}")
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

impl InstallMethod {
    fn new(name: &str, command: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            description: Some(description.to_string()),
        }
    }
}

struct InstallConfig {
    windows: &'static [(&'static str, &'static str, &'static str)],
    macos: &'static [(&'static str, &'static str, &'static str)],
    linux: &'static [(&'static str, &'static str, &'static str)],
}

impl InstallConfig {
    fn to_instructions(&self) -> InstallInstructions {
        InstallInstructions {
            windows: self
                .windows
                .iter()
                .map(|(name, cmd, desc)| InstallMethod::new(name, cmd, desc))
                .collect(),
            macos: self
                .macos
                .iter()
                .map(|(name, cmd, desc)| InstallMethod::new(name, cmd, desc))
                .collect(),
            linux: self
                .linux
                .iter()
                .map(|(name, cmd, desc)| InstallMethod::new(name, cmd, desc))
                .collect(),
        }
    }
}

const NODEJS_CONFIG: InstallConfig = InstallConfig {
    windows: &[
        (
            "winget",
            "winget install OpenJS.NodeJS",
            "Windows Package Manager (recommended)",
        ),
        (
            "chocolatey",
            "choco install nodejs",
            "Chocolatey package manager",
        ),
        (
            "download",
            "https://nodejs.org/en/download/",
            "Direct download from nodejs.org",
        ),
    ],
    macos: &[
        (
            "homebrew",
            "brew install node",
            "Homebrew package manager (recommended)",
        ),
        (
            "macports",
            "sudo port install nodejs20",
            "MacPorts package manager",
        ),
        (
            "download",
            "https://nodejs.org/en/download/",
            "Direct download from nodejs.org",
        ),
    ],
    linux: &[
        (
            "apt",
            "sudo apt update && sudo apt install nodejs npm",
            "Debian/Ubuntu (recommended)",
        ),
        ("dnf", "sudo dnf install nodejs npm", "Fedora/RHEL"),
        ("snap", "sudo snap install node --classic", "Snap package"),
        (
            "nvm",
            "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash",
            "Node Version Manager",
        ),
    ],
};

const PYTHON_CONFIG: InstallConfig = InstallConfig {
    windows: &[
        (
            "winget",
            "winget install Python.Python.3.12",
            "Windows Package Manager (recommended)",
        ),
        (
            "chocolatey",
            "choco install python",
            "Chocolatey package manager",
        ),
        (
            "download",
            "https://www.python.org/downloads/",
            "Direct download from python.org",
        ),
    ],
    macos: &[
        (
            "homebrew",
            "brew install python@3.12",
            "Homebrew package manager (recommended)",
        ),
        ("pyenv", "pyenv install 3.12.0", "Python version manager"),
        (
            "download",
            "https://www.python.org/downloads/",
            "Direct download from python.org",
        ),
    ],
    linux: &[
        (
            "apt",
            "sudo apt update && sudo apt install python3 python3-pip",
            "Debian/Ubuntu (recommended)",
        ),
        ("dnf", "sudo dnf install python3 python3-pip", "Fedora/RHEL"),
        (
            "pyenv",
            "curl https://pyenv.run | bash",
            "Python version manager",
        ),
    ],
};

const DOCKER_CONFIG: InstallConfig = InstallConfig {
    windows: &[
        (
            "docker-desktop",
            "https://www.docker.com/products/docker-desktop/",
            "Docker Desktop for Windows (recommended)",
        ),
        (
            "winget",
            "winget install Docker.DockerDesktop",
            "Install via Windows Package Manager",
        ),
    ],
    macos: &[
        (
            "docker-desktop",
            "https://www.docker.com/products/docker-desktop/",
            "Docker Desktop for Mac (recommended)",
        ),
        (
            "homebrew",
            "brew install --cask docker",
            "Install via Homebrew",
        ),
    ],
    linux: &[
        (
            "docker-ce",
            "https://docs.docker.com/engine/install/",
            "Docker Community Edition (recommended)",
        ),
        ("snap", "sudo snap install docker", "Install via Snap"),
    ],
};

const GIT_CONFIG: InstallConfig = InstallConfig {
    windows: &[
        (
            "winget",
            "winget install Git.Git",
            "Windows Package Manager (recommended)",
        ),
        (
            "chocolatey",
            "choco install git",
            "Chocolatey package manager",
        ),
        (
            "download",
            "https://git-scm.com/download/win",
            "Direct download from git-scm.com",
        ),
    ],
    macos: &[
        (
            "xcode",
            "xcode-select --install",
            "Xcode Command Line Tools (includes Git)",
        ),
        ("homebrew", "brew install git", "Homebrew package manager"),
    ],
    linux: &[
        (
            "apt",
            "sudo apt update && sudo apt install git",
            "Debian/Ubuntu",
        ),
        ("dnf", "sudo dnf install git", "Fedora/RHEL"),
        ("pacman", "sudo pacman -S git", "Arch Linux"),
    ],
};

pub fn get_install_instructions(dependency: &Dependency) -> InstallInstructions {
    match dependency {
        Dependency::NodeJs { .. } => NODEJS_CONFIG.to_instructions(),
        Dependency::Python { .. } => PYTHON_CONFIG.to_instructions(),
        Dependency::Docker { .. } => DOCKER_CONFIG.to_instructions(),
        Dependency::Git => GIT_CONFIG.to_instructions(),
    }
}
