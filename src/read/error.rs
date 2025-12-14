
#[derive(Debug, thiserror::Error)]
pub enum LefReadError {
    #[error("{0}")]
    Si2(String),


    #[error("{0}")]
    Msg(String),
}

pub type LefReadResult<T> = Result<T, LefReadError>; 