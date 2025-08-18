use std::error::Error;

pub type TuError = Box<dyn Error + Send + Sync>;
