use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr, Ipv4Addr, Ipv6Addr};
use std::io::{Read, Write, BufRead, BufReader};
use crate::ast::Expression;
use crate::error::CompilerError;
use reqwest::Client;
use std::collections::HashMap;

pub struct NetworkingModule;

impl NetworkingModule {
    pub fn new() -> Self {
        Self
    }
}

// HTTP Client functions
pub async fn http_get(url: &str) -> Result<String, CompilerError> {
    let client = Client::new();
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("HTTP GET failed: {}", e)))?;
    
    let body = response.text()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read response body: {}", e)))?;
    
    Ok(body)
}

pub async fn http_post(url: &str, data: &str) -> Result<String, CompilerError> {
    let client = Client::new();
    let response = client.post(url)
        .body(data.to_string())
        .send()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("HTTP POST failed: {}", e)))?;
    
    let body = response.text()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read response body: {}", e)))?;
    
    Ok(body)
}

pub async fn http_post_json(url: &str, json_data: &str) -> Result<String, CompilerError> {
    let client = Client::new();
    let response = client.post(url)
        .header("Content-Type", "application/json")
        .body(json_data.to_string())
        .send()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("HTTP POST JSON failed: {}", e)))?;
    
    let body = response.text()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read response body: {}", e)))?;
    
    Ok(body)
}

pub async fn http_put(url: &str, data: &str) -> Result<String, CompilerError> {
    let client = Client::new();
    let response = client.put(url)
        .body(data.to_string())
        .send()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("HTTP PUT failed: {}", e)))?;
    
    let body = response.text()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read response body: {}", e)))?;
    
    Ok(body)
}

pub async fn http_delete(url: &str) -> Result<String, CompilerError> {
    let client = Client::new();
    let response = client.delete(url)
        .send()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("HTTP DELETE failed: {}", e)))?;
    
    let body = response.text()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read response body: {}", e)))?;
    
    Ok(body)
}

// HTTP with headers
pub async fn http_get_with_headers(url: &str, headers: &HashMap<String, String>) -> Result<String, CompilerError> {
    let client = Client::new();
    let mut request = client.get(url);
    
    for (key, value) in headers {
        request = request.header(key, value);
    }
    
    let response = request.send()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("HTTP GET with headers failed: {}", e)))?;
    
    let body = response.text()
        .await
        .map_err(|e| CompilerError::runtime_error(&format!("Failed to read response body: {}", e)))?;
    
    Ok(body)
}

// TCP functions
pub fn tcp_connect(host: &str, port: u16) -> Result<TcpStream, CompilerError> {
    let address = format!("{}:{}", host, port);
    TcpStream::connect(&address)
        .map_err(|e| CompilerError::runtime_error(&format!("TCP connection failed: {}", e)))
}

pub fn tcp_send(stream: &mut TcpStream, data: &str) -> Result<(), CompilerError> {
    stream.write_all(data.as_bytes())
        .map_err(|e| CompilerError::runtime_error(&format!("TCP send failed: {}", e)))
}

pub fn tcp_receive(stream: &mut TcpStream, buffer_size: usize) -> Result<String, CompilerError> {
    let mut buffer = vec![0; buffer_size];
    let bytes_read = stream.read(&mut buffer)
        .map_err(|e| CompilerError::runtime_error(&format!("TCP receive failed: {}", e)))?;
    
    let data = String::from_utf8_lossy(&buffer[..bytes_read]);
    Ok(data.to_string())
}

pub fn tcp_receive_line(stream: &mut TcpStream) -> Result<String, CompilerError> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)
        .map_err(|e| CompilerError::runtime_error(&format!("TCP receive line failed: {}", e)))?;
    
    Ok(line.trim().to_string())
}

// TCP Server functions
pub fn tcp_listen(host: &str, port: u16) -> Result<TcpListener, CompilerError> {
    let address = format!("{}:{}", host, port);
    TcpListener::bind(&address)
        .map_err(|e| CompilerError::runtime_error(&format!("TCP listen failed: {}", e)))
}

pub fn tcp_accept(listener: &TcpListener) -> Result<(TcpStream, SocketAddr), CompilerError> {
    listener.accept()
        .map_err(|e| CompilerError::runtime_error(&format!("TCP accept failed: {}", e)))
}

// UDP functions
pub fn udp_bind(host: &str, port: u16) -> Result<UdpSocket, CompilerError> {
    let address = format!("{}:{}", host, port);
    UdpSocket::bind(&address)
        .map_err(|e| CompilerError::runtime_error(&format!("UDP bind failed: {}", e)))
}

pub fn udp_send_to(socket: &UdpSocket, data: &str, host: &str, port: u16) -> Result<usize, CompilerError> {
    let address = format!("{}:{}", host, port);
    socket.send_to(data.as_bytes(), &address)
        .map_err(|e| CompilerError::runtime_error(&format!("UDP send failed: {}", e)))
}

pub fn udp_receive_from(socket: &UdpSocket, buffer_size: usize) -> Result<(String, SocketAddr), CompilerError> {
    let mut buffer = vec![0; buffer_size];
    let (bytes_read, addr) = socket.recv_from(&mut buffer)
        .map_err(|e| CompilerError::runtime_error(&format!("UDP receive failed: {}", e)))?;
    
    let data = String::from_utf8_lossy(&buffer[..bytes_read]);
    Ok((data.to_string(), addr))
}

// Network utility functions
pub fn parse_ipv4(ip_str: &str) -> Result<Ipv4Addr, CompilerError> {
    ip_str.parse::<Ipv4Addr>()
        .map_err(|e| CompilerError::runtime_error(&format!("Invalid IPv4 address: {}", e)))
}

pub fn parse_ipv6(ip_str: &str) -> Result<Ipv6Addr, CompilerError> {
    ip_str.parse::<Ipv6Addr>()
        .map_err(|e| CompilerError::runtime_error(&format!("Invalid IPv6 address: {}", e)))
}

pub fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr, CompilerError> {
    addr_str.parse::<SocketAddr>()
        .map_err(|e| CompilerError::runtime_error(&format!("Invalid socket address: {}", e)))
}

pub fn is_valid_port(port: u16) -> bool {
    port > 0 && port < 65535
}

pub fn is_private_ip(ip: &str) -> bool {
    if let Ok(ipv4) = ip.parse::<Ipv4Addr>() {
        ipv4.is_private()
    } else {
        false
    }
}

pub fn is_loopback_ip(ip: &str) -> bool {
    if let Ok(ipv4) = ip.parse::<Ipv4Addr>() {
        ipv4.is_loopback()
    } else {
        false
    }
}

// DNS functions (simplified)
pub fn resolve_hostname(hostname: &str) -> Result<Vec<String>, CompilerError> {
    // This is a simplified DNS resolution
    // In a real implementation, you would use a proper DNS resolver
    Ok(vec![hostname.to_string()])
}

// Network testing functions
pub fn ping_host(host: &str, port: u16) -> Result<bool, CompilerError> {
    match tcp_connect(host, port) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

pub fn check_port_open(host: &str, port: u16) -> Result<bool, CompilerError> {
    ping_host(host, port)
}

// URL parsing functions
pub fn parse_url(url: &str) -> Result<HashMap<String, String>, CompilerError> {
    let mut parts = HashMap::new();
    
    if let Some(scheme_end) = url.find("://") {
        parts.insert("scheme".to_string(), url[..scheme_end].to_string());
        let rest = &url[scheme_end + 3..];
        
        if let Some(slash_pos) = rest.find('/') {
            let authority = &rest[..slash_pos];
            let path = &rest[slash_pos..];
            
            if let Some(colon_pos) = authority.find(':') {
                parts.insert("host".to_string(), authority[..colon_pos].to_string());
                if let Some(port_end) = authority[colon_pos + 1..].find('/') {
                    parts.insert("port".to_string(), authority[colon_pos + 1..colon_pos + 1 + port_end].to_string());
                } else {
                    parts.insert("port".to_string(), authority[colon_pos + 1..].to_string());
                }
            } else {
                parts.insert("host".to_string(), authority.to_string());
            }
            
            parts.insert("path".to_string(), path.to_string());
        } else {
            parts.insert("host".to_string(), rest.to_string());
        }
    } else {
        return Err(CompilerError::runtime_error("Invalid URL format"));
    }
    
    Ok(parts)
}

// WebSocket functions (simplified - would need a proper WebSocket library)
pub fn websocket_connect(_url: &str) -> Result<(), CompilerError> {
    // This is a placeholder for WebSocket functionality
    // In a real implementation, you would use a WebSocket library like tungstenite
    Err(CompilerError::runtime_error("WebSocket support not yet implemented"))
}

pub fn websocket_send(_data: &str) -> Result<(), CompilerError> {
    Err(CompilerError::runtime_error("WebSocket support not yet implemented"))
}

pub fn websocket_receive() -> Result<String, CompilerError> {
    Err(CompilerError::runtime_error("WebSocket support not yet implemented"))
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