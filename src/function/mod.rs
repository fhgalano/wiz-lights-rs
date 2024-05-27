// Defining all funcs to run on registry members
use std::fmt;
use std::error::Error;
use std::fmt::Formatter;

use crate::bulb::response::ErrorResponse;


#[derive(Debug, Clone)]
pub struct FunctionError {
    function_name: String,
    inner_error: String,
}

impl FunctionError {
    pub fn new(function_name: String, inner_error: String) -> FunctionError {
        FunctionError {
            function_name,
            inner_error,
        }
    }
}

impl fmt::Display for FunctionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "An error occurred during WizFunction {} - {}",
            self.function_name,
            self.inner_error,
        )
    }
}

impl Error for FunctionError {}

pub trait On {
    fn on(&mut self) -> Result<bool, ErrorResponse>;
}

pub trait Off {
    fn off(&mut self) -> Result<bool, ErrorResponse>;
}
