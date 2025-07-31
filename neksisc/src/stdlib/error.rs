// Basic error module
#[derive(Debug, Clone, PartialEq)]
pub struct Exception {
    pub message: String,
}

pub struct StdLib;

impl StdLib {
    pub fn new() -> Self {
        Self
    }
}

pub struct BuiltinFunction;
pub struct BuiltinImpl; 