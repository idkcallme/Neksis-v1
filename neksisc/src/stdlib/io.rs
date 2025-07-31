use std::fs;
use std::io::{self, Write};
use crate::error::CompilerError;

pub fn read_file(path: &str) -> Result<String, CompilerError> {
    fs::read_to_string(path)
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read file: {}", e)))
}

pub fn write_file(path: &str, content: &str) -> Result<(), CompilerError> {
    fs::write(path, content)
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to write file: {}", e)))
}

pub fn append_file(path: &str, content: &str) -> Result<(), CompilerError> {
    use std::fs::OpenOptions;
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(content.as_bytes()))
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to append to file: {}", e)))
}

pub fn file_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub fn delete_file(path: &str) -> Result<(), CompilerError> {
    fs::remove_file(path)
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to delete file: {}", e)))
}

pub fn create_directory(path: &str) -> Result<(), CompilerError> {
    fs::create_dir_all(path)
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to create directory: {}", e)))
}

pub fn list_directory(path: &str) -> Result<Vec<String>, CompilerError> {
    fs::read_dir(path)
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read directory: {}", e)))
        .and_then(|entries| {
            entries
                .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CompilerError::runtime_error(&format!("Failed to read directory entry: {}", e)))
        })
}

pub fn print_to_stdout(text: &str) {
    print!("{}", text);
    io::stdout().flush().unwrap_or_default();
}

pub fn print_to_stderr(text: &str) {
    eprint!("{}", text);
    io::stderr().flush().unwrap_or_default();
}

pub fn read_from_stdin() -> Result<String, CompilerError> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read from stdin: {}", e)))?;
    Ok(input.trim().to_string())
} 