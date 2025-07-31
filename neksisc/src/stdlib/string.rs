use regex::Regex;
use crate::ast::Expression;
use crate::error::CompilerError;

pub struct StringModule;

impl StringModule {
    pub fn new() -> Self {
        Self
    }
}

// Basic string operations
pub fn string_length(s: &str) -> usize {
    s.len()
}

pub fn string_is_empty(s: &str) -> bool {
    s.is_empty()
}

pub fn string_trim(s: &str) -> String {
    s.trim().to_string()
}

pub fn string_trim_start(s: &str) -> String {
    s.trim_start().to_string()
}

pub fn string_trim_end(s: &str) -> String {
    s.trim_end().to_string()
}

pub fn string_to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

pub fn string_to_lowercase(s: &str) -> String {
    s.to_lowercase()
}

pub fn string_capitalize(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        let mut chars: Vec<char> = s.chars().collect();
        if let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap();
        }
        chars.into_iter().collect()
    }
}

pub fn string_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| string_capitalize(word))
        .collect::<Vec<String>>()
        .join(" ")
}

// String concatenation and manipulation
pub fn string_concat(a: &str, b: &str) -> String {
    a.to_string() + b
}

pub fn string_join(parts: &[String], separator: &str) -> String {
    parts.join(separator)
}

pub fn string_split(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(|s| s.to_string()).collect()
}

pub fn string_split_whitespace(s: &str) -> Vec<String> {
    s.split_whitespace().map(|s| s.to_string()).collect()
}

pub fn string_lines(s: &str) -> Vec<String> {
    s.lines().map(|s| s.to_string()).collect()
}

pub fn string_chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

pub fn string_bytes(s: &str) -> Vec<u8> {
    s.bytes().collect()
}

// Substring operations
pub fn string_substring(s: &str, start: usize, end: usize) -> Result<String, CompilerError> {
    if start > end || end > s.len() {
        Err(CompilerError::runtime_error("Substring indices out of bounds"))
    } else {
        Ok(s[start..end].to_string())
    }
}

pub fn string_replace(s: &str, old: &str, new: &str) -> String {
    s.replace(old, new)
}

pub fn string_replace_all(s: &str, old: &str, new: &str) -> String {
    s.replace(old, new)
}

pub fn string_replace_n(s: &str, old: &str, new: &str, count: usize) -> String {
    let mut result = s.to_string();
    for _ in 0..count {
        result = result.replacen(old, new, 1);
    }
    result
}

// Pattern matching and search
pub fn string_contains(s: &str, pattern: &str) -> bool {
    s.contains(pattern)
}

pub fn string_starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

pub fn string_ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

pub fn string_find(s: &str, pattern: &str) -> Option<usize> {
    s.find(pattern)
}

pub fn string_rfind(s: &str, pattern: &str) -> Option<usize> {
    s.rfind(pattern)
}

pub fn string_count(s: &str, pattern: &str) -> usize {
    s.matches(pattern).count()
}

// Regular expressions
pub fn regex_match(pattern: &str, text: &str) -> Result<bool, CompilerError> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex.is_match(text)),
        Err(_) => Err(CompilerError::runtime_error("Invalid regex pattern"))
    }
}

pub fn regex_find(pattern: &str, text: &str) -> Result<Option<String>, CompilerError> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex.find(text).map(|m| m.as_str().to_string())),
        Err(_) => Err(CompilerError::runtime_error("Invalid regex pattern"))
    }
}

pub fn regex_find_all(pattern: &str, text: &str) -> Result<Vec<String>, CompilerError> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex.find_iter(text).map(|m| m.as_str().to_string()).collect()),
        Err(_) => Err(CompilerError::runtime_error("Invalid regex pattern"))
    }
}

pub fn regex_replace(pattern: &str, text: &str, replacement: &str) -> Result<String, CompilerError> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex.replace_all(text, replacement).to_string()),
        Err(_) => Err(CompilerError::runtime_error("Invalid regex pattern"))
    }
}

pub fn regex_capture_groups(pattern: &str, text: &str) -> Result<Vec<String>, CompilerError> {
    match Regex::new(pattern) {
        Ok(regex) => {
            if let Some(captures) = regex.captures(text) {
                Ok(captures.iter()
                    .skip(1) // Skip the full match
                    .filter_map(|m| m.map(|m| m.as_str().to_string()))
                    .collect())
            } else {
                Ok(Vec::new())
            }
        },
        Err(_) => Err(CompilerError::runtime_error("Invalid regex pattern"))
    }
}

// Encoding and decoding
pub fn string_encode_utf8(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

pub fn string_decode_utf8(bytes: &[u8]) -> Result<String, CompilerError> {
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(CompilerError::runtime_error("Invalid UTF-8 sequence"))
    }
}

pub fn string_encode_base64(s: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(s.as_bytes())
}

pub fn string_decode_base64(s: &str) -> Result<String, CompilerError> {
    use base64::{Engine as _, engine::general_purpose};
    match general_purpose::STANDARD.decode(s) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(CompilerError::runtime_error("Invalid base64 encoding"))
        },
        Err(_) => Err(CompilerError::runtime_error("Invalid base64 encoding"))
    }
}

pub fn string_encode_hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn string_decode_hex(s: &str) -> Result<String, CompilerError> {
    if s.len() % 2 != 0 {
        return Err(CompilerError::runtime_error("Invalid hex string length"));
    }
    
    let mut bytes = Vec::new();
    for i in (0..s.len()).step_by(2) {
        let byte_str = &s[i..i+2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(CompilerError::runtime_error("Invalid hex string"))
        }
    }
    
    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(_) => Err(CompilerError::runtime_error("Invalid UTF-8 sequence"))
    }
}

// String formatting
pub fn string_format(template: &str, args: &[String]) -> Result<String, CompilerError> {
    let mut result = template.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    Ok(result)
}

pub fn string_pad_left(s: &str, width: usize, pad_char: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let padding = pad_char.to_string().repeat(width - s.len());
        padding + s
    }
}

pub fn string_pad_right(s: &str, width: usize, pad_char: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let padding = pad_char.to_string().repeat(width - s.len());
        s.to_string() + &padding
    }
}

pub fn string_center(s: &str, width: usize, pad_char: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let padding = width - s.len();
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        
        let left = pad_char.to_string().repeat(left_padding);
        let right = pad_char.to_string().repeat(right_padding);
        
        left + s + &right
    }
}

// String validation
pub fn string_is_alpha(s: &str) -> bool {
    s.chars().all(|c| c.is_alphabetic())
}

pub fn string_is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric())
}

pub fn string_is_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric())
}

pub fn string_is_whitespace(s: &str) -> bool {
    s.chars().all(|c| c.is_whitespace())
}

pub fn string_is_ascii(s: &str) -> bool {
    s.is_ascii()
}

// String transformation
pub fn string_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn string_repeat(s: &str, count: usize) -> String {
    s.repeat(count)
}

pub fn string_remove(s: &str, pattern: &str) -> String {
    s.replace(pattern, "")
}

pub fn string_remove_all(s: &str, pattern: &str) -> String {
    s.replace(pattern, "")
}

// Builtin function implementations for the standard library
pub struct BuiltinFunction;

impl BuiltinFunction {
    pub fn execute(&self, _args: &[Expression]) -> Result<Expression, CompilerError> {
        Err(CompilerError::runtime_error("BuiltinFunction not implemented"))
    }
}

pub struct BuiltinImpl;

impl BuiltinImpl {
    pub fn new() -> Self {
        Self
    }
} 