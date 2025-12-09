
#[derive(Debug, thiserror::Error)]
pub enum LefReadError {
    #[error("{0}")]
    Si2(String)
}

pub type LefReadResult<T> = Result<T, LefReadError>; 