
#[derive(Debug, thiserror::Error)]
pub enum LefError {

}

pub type LefResult<T> = Result<T, LefError>; 