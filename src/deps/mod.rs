use anyhow::Result;
use std::fmt;

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

#[derive(Debug, Clone)]
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

pub fn get_install_instructions(dependency: &Dependency) -> InstallInstructions {
    match dependency {
        Dependency::NodeJs { .. } => InstallInstructions {
            windows: vec![
                InstallMethod {
                    name: "winget".to_string(),
                    command: "winget install OpenJS.NodeJS".to_string(),
                    description: Some("Windows Package Manager (recommended)".to_string()),
                },
                InstallMethod {
                    name: "chocolatey".to_string(),
                    command: "choco install nodejs".to_string(),
                    description: Some("Chocolatey package manager".to_string()),
                },
                InstallMethod {
                    name: "download".to_string(),
                    command: "https://nodejs.org/en/download/".to_string(),
                    description: Some("Direct download from nodejs.org".to_string()),
                },
            ],
            macos: vec![
                InstallMethod {
                    name: "homebrew".to_string(),
                    command: "brew install node".to_string(),
                    description: Some("Homebrew package manager (recommended)".to_string()),
                },
                InstallMethod {
                    name: "macports".to_string(),
                    command: "sudo port install nodejs20".to_string(),
                    description: Some("MacPorts package manager".to_string()),
                },
                InstallMethod {
                    name: "download".to_string(),
                    command: "https://nodejs.org/en/download/".to_string(),
                    description: Some("Direct download from nodejs.org".to_string()),
                },
            ],
            linux: vec![
                InstallMethod {
                    name: "apt".to_string(),
                    command: "sudo apt update && sudo apt install nodejs npm".to_string(),
                    description: Some("Debian/Ubuntu (recommended)".to_string()),
                },
                InstallMethod {
                    name: "dnf".to_string(),
                    command: "sudo dnf install nodejs npm".to_string(),
                    description: Some("Fedora/RHEL".to_string()),
                },
                InstallMethod {
                    name: "snap".to_string(),
                    command: "sudo snap install node --classic".to_string(),
                    description: Some("Snap package".to_string()),
                },
                InstallMethod {
                    name: "nvm".to_string(),
                    command: "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash".to_string(),
                    description: Some("Node Version Manager".to_string()),
                },
            ],
        },
        Dependency::Python { .. } => InstallInstructions {
            windows: vec![
                InstallMethod {
                    name: "winget".to_string(),
                    command: "winget install Python.Python.3.12".to_string(),
                    description: Some("Windows Package Manager (recommended)".to_string()),
                },
                InstallMethod {
                    name: "chocolatey".to_string(),
                    command: "choco install python".to_string(),
                    description: Some("Chocolatey package manager".to_string()),
                },
                InstallMethod {
                    name: "download".to_string(),
                    command: "https://www.python.org/downloads/".to_string(),
                    description: Some("Direct download from python.org".to_string()),
                },
            ],
            macos: vec![
                InstallMethod {
                    name: "homebrew".to_string(),
                    command: "brew install python@3.12".to_string(),
                    description: Some("Homebrew package manager (recommended)".to_string()),
                },
                InstallMethod {
                    name: "pyenv".to_string(),
                    command: "pyenv install 3.12.0".to_string(),
                    description: Some("Python version manager".to_string()),
                },
                InstallMethod {
                    name: "download".to_string(),
                    command: "https://www.python.org/downloads/".to_string(),
                    description: Some("Direct download from python.org".to_string()),
                },
            ],
            linux: vec![
                InstallMethod {
                    name: "apt".to_string(),
                    command: "sudo apt update && sudo apt install python3 python3-pip".to_string(),
                    description: Some("Debian/Ubuntu (recommended)".to_string()),
                },
                InstallMethod {
                    name: "dnf".to_string(),
                    command: "sudo dnf install python3 python3-pip".to_string(),
                    description: Some("Fedora/RHEL".to_string()),
                },
                InstallMethod {
                    name: "pyenv".to_string(),
                    command: "curl https://pyenv.run | bash".to_string(),
                    description: Some("Python version manager".to_string()),
                },
            ],
        },
        Dependency::Docker => InstallInstructions {
            windows: vec![
                InstallMethod {
                    name: "docker-desktop".to_string(),
                    command: "https://www.docker.com/products/docker-desktop/".to_string(),
                    description: Some("Docker Desktop for Windows (recommended)".to_string()),
                },
                InstallMethod {
                    name: "winget".to_string(),
                    command: "winget install Docker.DockerDesktop".to_string(),
                    description: Some("Install via Windows Package Manager".to_string()),
                },
            ],
            macos: vec![
                InstallMethod {
                    name: "docker-desktop".to_string(),
                    command: "https://www.docker.com/products/docker-desktop/".to_string(),
                    description: Some("Docker Desktop for Mac (recommended)".to_string()),
                },
                InstallMethod {
                    name: "homebrew".to_string(),
                    command: "brew install --cask docker".to_string(),
                    description: Some("Install via Homebrew".to_string()),
                },
            ],
            linux: vec![
                InstallMethod {
                    name: "docker-ce".to_string(),
                    command: "https://docs.docker.com/engine/install/".to_string(),
                    description: Some("Docker Community Edition (recommended)".to_string()),
                },
                InstallMethod {
                    name: "snap".to_string(),
                    command: "sudo snap install docker".to_string(),
                    description: Some("Install via Snap".to_string()),
                },
            ],
        },
        Dependency::Git => InstallInstructions {
            windows: vec![
                InstallMethod {
                    name: "winget".to_string(),
                    command: "winget install Git.Git".to_string(),
                    description: Some("Windows Package Manager (recommended)".to_string()),
                },
                InstallMethod {
                    name: "chocolatey".to_string(),
                    command: "choco install git".to_string(),
                    description: Some("Chocolatey package manager".to_string()),
                },
                InstallMethod {
                    name: "download".to_string(),
                    command: "https://git-scm.com/download/win".to_string(),
                    description: Some("Direct download from git-scm.com".to_string()),
                },
            ],
            macos: vec![
                InstallMethod {
                    name: "xcode".to_string(),
                    command: "xcode-select --install".to_string(),
                    description: Some("Xcode Command Line Tools (includes Git)".to_string()),
                },
                InstallMethod {
                    name: "homebrew".to_string(),
                    command: "brew install git".to_string(),
                    description: Some("Homebrew package manager".to_string()),
                },
            ],
            linux: vec![
                InstallMethod {
                    name: "apt".to_string(),
                    command: "sudo apt update && sudo apt install git".to_string(),
                    description: Some("Debian/Ubuntu".to_string()),
                },
                InstallMethod {
                    name: "dnf".to_string(),
                    command: "sudo dnf install git".to_string(),
                    description: Some("Fedora/RHEL".to_string()),
                },
                InstallMethod {
                    name: "pacman".to_string(),
                    command: "sudo pacman -S git".to_string(),
                    description: Some("Arch Linux".to_string()),
                },
            ],
        },
    }
}
