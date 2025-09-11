use std::error::Error;

pub type TuError = Box<dyn Error + Send + Sync>;
pub type Res<T> = Result<T, TuError>;