use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("输入无效：{0}")]
    Invalid(String),
    #[error("operation canceled")]
    Cancelled,
    #[error("provider error: {0}")]
    Provider(String),
    #[error("missing credential: {0}")]
    MissingCredential(String),
    #[error("ffmpeg is unavailable")]
    FfmpegUnavailable,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Message(_) => "error",
            Self::Database(_) => "database_error",
            Self::Io(_) => "io_error",
            Self::Json(_) => "json_error",
            Self::Http(_) => "network_error",
            Self::NotFound(_) => "not_found",
            Self::Invalid(_) => "invalid_input",
            Self::Cancelled => "cancelled",
            Self::Provider(_) => "provider_error",
            Self::MissingCredential(_) => "missing_credential",
            Self::FfmpegUnavailable => "ffmpeg_unavailable",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait OptionExt<T> {
    fn or_not_found(self, what: &str) -> AppResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_not_found(self, what: &str) -> AppResult<T> {
        self.ok_or_else(|| AppError::NotFound(what.to_string()))
    }
}
