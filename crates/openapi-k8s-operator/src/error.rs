use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Kube(kube::Error),
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Kube(e) => write!(f, "Kubernetes error: {}", e),
            AppError::Reqwest(e) => write!(f, "HTTP error: {}", e),
            AppError::Serde(e) => write!(f, "Serialization error: {}", e),
            AppError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Kube(e) => Some(e),
            AppError::Reqwest(e) => Some(e),
            AppError::Serde(e) => Some(e),
            AppError::Io(e) => Some(e),
        }
    }
}

impl From<kube::Error> for AppError {
    fn from(err: kube::Error) -> Self {
        AppError::Kube(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Reqwest(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serde(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}
