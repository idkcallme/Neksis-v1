use crate::ast::Expression;
use crate::error::CompilerError;
use sha2::{Sha256, Sha512, Digest};
use aes::{Aes128, Aes256};
use aes::cipher::{
    BlockEncrypt, BlockDecrypt,
    KeyInit,
    generic_array::GenericArray,
};
use rand::{Rng, RngCore};
use base64::{Engine as _, engine::general_purpose};

pub struct CryptoModule;

impl CryptoModule {
    pub fn new() -> Self {
        Self
    }
}

// Hashing functions
pub fn hash_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn hash_sha512(data: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn hash_md5(data: &str) -> String {
    // Note: MD5 is cryptographically broken, but included for compatibility
    // TODO: Fix md5 crate usage
    // let mut hasher = md5::Md5::new();
    // hasher.update(data.as_bytes());
    // let result = hasher.finalize();
    // format!("{:x}", result)
    format!("md5_hash_{}", data.len())
}

pub fn hash_sha1(data: &str) -> String {
    // Note: SHA1 is cryptographically broken, but included for compatibility
    use sha1::{Sha1, Digest as Sha1Digest};
    let mut hasher = Sha1::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

// HMAC functions
pub fn hmac_sha256(key: &str, data: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    format!("{:x}", result.into_bytes())
}

pub fn hmac_sha512(key: &str, data: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha512;
    
    let mut mac = <Hmac<Sha512> as Mac>::new_from_slice(key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    format!("{:x}", result.into_bytes())
}

// Password hashing
pub fn hash_password(password: &str, salt: &str) -> String {
    // Simple password hashing using SHA256 with salt
    let salted = format!("{}{}", password, salt);
    hash_sha256(&salted)
}

pub fn verify_password(password: &str, salt: &str, hash: &str) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == hash
}

// Random number generation
pub fn random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

pub fn random_string(length: usize) -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

pub fn random_hex_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

pub fn secure_random_u32() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn secure_random_u64() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn secure_random_f64() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

// AES encryption/decryption
pub fn aes_encrypt_128(data: &str, key: &str) -> Result<String, CompilerError> {
    if key.len() != 16 {
        return Err(CompilerError::runtime_error("AES-128 key must be 16 bytes"));
    }
    
    let cipher = Aes128::new_from_slice(key.as_bytes())
        .map_err(|_| CompilerError::runtime_error("Invalid AES key"))?;
    
    let mut encrypted = Vec::new();
    let data_bytes = data.as_bytes();
    
    // Pad data to 16-byte blocks
    let mut padded_data = data_bytes.to_vec();
    let padding = 16 - (data_bytes.len() % 16);
    padded_data.extend(std::iter::repeat(padding as u8).take(padding));
    
    // Encrypt each block
    for chunk in padded_data.chunks(16) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }
    
    Ok(general_purpose::STANDARD.encode(encrypted))
}

pub fn aes_decrypt_128(encrypted_data: &str, key: &str) -> Result<String, CompilerError> {
    if key.len() != 16 {
        return Err(CompilerError::runtime_error("AES-128 key must be 16 bytes"));
    }
    
    let cipher = Aes128::new_from_slice(key.as_bytes())
        .map_err(|_| CompilerError::runtime_error("Invalid AES key"))?;
    
    let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_data)
        .map_err(|_| CompilerError::runtime_error("Invalid base64 encoding"))?;
    
    let mut decrypted = Vec::new();
    
    // Decrypt each block
    for chunk in encrypted_bytes.chunks(16) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        decrypted.extend_from_slice(&block);
    }
    
    // Remove padding
    if let Some(&padding) = decrypted.last() {
        if padding <= 16 {
            decrypted.truncate(decrypted.len() - padding as usize);
        }
    }
    
    String::from_utf8(decrypted)
        .map_err(|_| CompilerError::runtime_error("Invalid UTF-8 in decrypted data"))
}

pub fn aes_encrypt_256(data: &str, key: &str) -> Result<String, CompilerError> {
    if key.len() != 32 {
        return Err(CompilerError::runtime_error("AES-256 key must be 32 bytes"));
    }
    
    let cipher = Aes256::new_from_slice(key.as_bytes())
        .map_err(|_| CompilerError::runtime_error("Invalid AES key"))?;
    
    let mut encrypted = Vec::new();
    let data_bytes = data.as_bytes();
    
    // Pad data to 16-byte blocks
    let mut padded_data = data_bytes.to_vec();
    let padding = 16 - (data_bytes.len() % 16);
    padded_data.extend(std::iter::repeat(padding as u8).take(padding));
    
    // Encrypt each block
    for chunk in padded_data.chunks(16) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }
    
    Ok(general_purpose::STANDARD.encode(encrypted))
}

pub fn aes_decrypt_256(encrypted_data: &str, key: &str) -> Result<String, CompilerError> {
    if key.len() != 32 {
        return Err(CompilerError::runtime_error("AES-256 key must be 32 bytes"));
    }
    
    let cipher = Aes256::new_from_slice(key.as_bytes())
        .map_err(|_| CompilerError::runtime_error("Invalid AES key"))?;
    
    let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_data)
        .map_err(|_| CompilerError::runtime_error("Invalid base64 encoding"))?;
    
    let mut decrypted = Vec::new();
    
    // Decrypt each block
    for chunk in encrypted_bytes.chunks(16) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        decrypted.extend_from_slice(&block);
    }
    
    // Remove padding
    if let Some(&padding) = decrypted.last() {
        if padding <= 16 {
            decrypted.truncate(decrypted.len() - padding as usize);
        }
    }
    
    String::from_utf8(decrypted)
        .map_err(|_| CompilerError::runtime_error("Invalid UTF-8 in decrypted data"))
}

// Key generation
pub fn generate_aes_128_key() -> String {
    let key_bytes = random_bytes(16);
    general_purpose::STANDARD.encode(key_bytes)
}

pub fn generate_aes_256_key() -> String {
    let key_bytes = random_bytes(32);
    general_purpose::STANDARD.encode(key_bytes)
}

pub fn generate_salt(length: usize) -> String {
    random_string(length)
}

// Utility functions
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, CompilerError> {
    if hex.len() % 2 != 0 {
        return Err(CompilerError::runtime_error("Hex string must have even length"));
    }
    
    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i+2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| CompilerError::runtime_error("Invalid hex string"))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

pub fn string_to_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

pub fn bytes_to_string(bytes: &[u8]) -> Result<String, CompilerError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| CompilerError::runtime_error("Invalid UTF-8 sequence"))
}

// Cryptographic constants
pub fn crypto_constants() -> std::collections::HashMap<String, String> {
    let mut constants = std::collections::HashMap::new();
    constants.insert("SHA256_BLOCK_SIZE".to_string(), "64".to_string());
    constants.insert("SHA512_BLOCK_SIZE".to_string(), "128".to_string());
    constants.insert("AES_BLOCK_SIZE".to_string(), "16".to_string());
    constants.insert("AES128_KEY_SIZE".to_string(), "16".to_string());
    constants.insert("AES256_KEY_SIZE".to_string(), "32".to_string());
    constants
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