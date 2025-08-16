use std::num::ParseIntError;

#[derive(Debug, Clone)]
pub enum ExdsdevsError {
    ErrorFileSystem(String),
    ErrorParseJson(String),
    ErrorParseInt(String),
    ErrorSimTime(String),
    ErrorCartesian(String),
    ErrorBuildSimulator(String),
}

impl ToString for ExdsdevsError {
    fn to_string(&self) -> String {
        match &self {
            ExdsdevsError::ErrorFileSystem(value) => {
                format!("ExdsdevsError::ErrorFileSystem: {}", value)
            }
            ExdsdevsError::ErrorParseJson(value) => {
                format!("ExdsdevsError::ErrorParseJson: {}", value)
            }
            ExdsdevsError::ErrorParseInt(value) => {
                format!("ExdsdevsError::ErrorParseInt: {}", value)
            }
            ExdsdevsError::ErrorSimTime(value) => format!("ExdsdevsError::ErrorSimTime: {}", value),
            ExdsdevsError::ErrorCartesian(value) => {
                format!("ExdsdevsError::ErrorCartesian: {}", value)
            }
            ExdsdevsError::ErrorBuildSimulator(value) => {
                format!("ExdsdevsError::ErrorBuildSimulator: {}", value)
            }
        }
    }
}

impl From<std::io::Error> for ExdsdevsError {
    fn from(value: std::io::Error) -> Self {
        ExdsdevsError::ErrorFileSystem(value.to_string())
    }
}

impl From<serde_json::Error> for ExdsdevsError {
    fn from(value: serde_json::Error) -> Self {
        ExdsdevsError::ErrorParseJson(value.to_string())
    }
}

impl From<ParseIntError> for ExdsdevsError {
    fn from(value: ParseIntError) -> Self {
        ExdsdevsError::ErrorParseInt(value.to_string())
    }
}
