use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("erreur d'entree/sortie : {0}")]
    Io(#[from] std::io::Error),
    #[error("erreur HEIC : {0}")]
    Heic(String),
    #[error("trop de pixels : {got} > {limit}")]
    TooManyPixels { got: u64, limit: u64 },
}

impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Heic(message.into())
    }
}
