#[derive(Debug)]
pub enum SerdeError {
    Serde(RonError),
    Io(std::io::Error),
}

impl std::fmt::Display for SerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Serde(e) => (e as &dyn std::fmt::Display).fmt(f),
            Self::Io(e) => (e as &dyn std::fmt::Display).fmt(f),
        }
    }
}

impl std::error::Error for SerdeError {}

#[derive(Debug)]
pub enum RonError {
    General(ron::error::Error),
    Spanned(ron::error::SpannedError),
}

impl std::fmt::Display for RonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::General(e) => (e as &dyn std::fmt::Display).fmt(f),
            Self::Spanned(e) => (e as &dyn std::fmt::Display).fmt(f),
        }
    }
}

impl From<ron::error::Error> for RonError {
    fn from(value: ron::error::Error) -> Self {
        Self::General(value)
    }
}

impl From<ron::error::SpannedError> for RonError {
    fn from(value: ron::error::SpannedError) -> Self {
        Self::Spanned(value)
    }
}
