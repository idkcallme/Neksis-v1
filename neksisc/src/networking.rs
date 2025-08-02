// Networking module for Neksis 2025
use std::collections::HashMap;
use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::io::{Read, Write, BufRead, BufReader};
use std::time::Duration;
use crate::modern_stdlib::{NeksisError, NeksisResult};

/// HTTP Methods
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl From<&str> for HttpMethod {
    fn from(method: &str) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET, // Default to GET
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout: Option<Duration>,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, url: &str) -> Self {
        Self {
            method,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            timeout: Some(Duration::from_secs(30)),
        }
    }
    
    pub fn get(url: &str) -> Self {
        Self::new(HttpMethod::GET, url)
    }
    
    pub fn post(url: &str) -> Self {
        Self::new(HttpMethod::POST, url)
    }
    
    pub fn put(url: &str) -> Self {
        Self::new(HttpMethod::PUT, url)
    }
    
    pub fn delete(url: &str) -> Self {
        Self::new(HttpMethod::DELETE, url)
    }
    
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
    
    pub fn json_body(mut self, json: &str) -> Self {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json.to_string());
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// HTTP Response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }
    
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500 && self.status_code < 600
    }
    
    pub fn json<T>(&self) -> NeksisResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str(&self.body)
            .map_err(|e| NeksisError::ParseError(format!("JSON parse error: {}", e)))
    }
}

/// HTTP Client
pub struct HttpClient {
    default_headers: HashMap<String, String>,
    default_timeout: Duration,
}

impl HttpClient {
    pub fn new() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), "Neksis-HTTP/1.0".to_string());
        
        Self {
            default_headers,
            default_timeout: Duration::from_secs(30),
        }
    }
    
    pub fn with_timeout(timeout: Duration) -> Self {
        let mut client = Self::new();
        client.default_timeout = timeout;
        client
    }
    
    pub fn send(&self, request: HttpRequest) -> NeksisResult<HttpResponse> {
        // Parse URL (simplified)
        let url_parts: Vec<&str> = request.url.splitn(3, '/').collect();
        if url_parts.len() < 3 {
            return Err(NeksisError::NetworkError("Invalid URL format".to_string()));
        }
        
        let protocol = url_parts[0].trim_end_matches(':');
        if protocol != "http" && protocol != "https" {
            return Err(NeksisError::NetworkError("Only HTTP and HTTPS protocols supported".to_string()));
        }
        
        let host_port = url_parts[2];
        let path = if url_parts.len() > 2 {
            format!("/{}", url_parts[2..].join("/"))
        } else {
            "/".to_string()
        };
        
        // Extract host and port
        let (host, port) = if let Some(colon_pos) = host_port.find(':') {
            let host = &host_port[..colon_pos];
            let port_str = &host_port[colon_pos + 1..];
            let port = port_str.parse::<u16>()
                .map_err(|_| NeksisError::NetworkError("Invalid port number".to_string()))?;
            (host, port)
        } else {
            let port = if protocol == "https" { 443 } else { 80 };
            (host_port, port)
        };
        
        // Create socket address
        let socket_addr = format!("{}:{}", host, port)
            .parse::<SocketAddr>()
            .map_err(|e| NeksisError::NetworkError(format!("Invalid address: {}", e)))?;
        
        // Connect to server
        let mut stream = TcpStream::connect_timeout(&socket_addr, request.timeout.unwrap_or(self.default_timeout))
            .map_err(|e| NeksisError::NetworkError(format!("Connection failed: {}", e)))?;
        
        // Build HTTP request
        let mut http_request = format!("{} {} HTTP/1.1\r\n", request.method, path);
        http_request.push_str(&format!("Host: {}\r\n", host));
        
        // Add headers
        for (key, value) in &self.default_headers {
            http_request.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        for (key, value) in &request.headers {
            http_request.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        // Add body if present
        if let Some(body) = &request.body {
            http_request.push_str(&format!("Content-Length: {}\r\n", body.len()));
            http_request.push_str("\r\n");
            http_request.push_str(body);
        } else {
            http_request.push_str("\r\n");
        }
        
        // Send request
        stream.write_all(http_request.as_bytes())
            .map_err(|e| NeksisError::NetworkError(format!("Failed to send request: {}", e)))?;
        
        // Read response
        let mut reader = BufReader::new(&mut stream);
        let mut status_line = String::new();
        reader.read_line(&mut status_line)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to read response: {}", e)))?;
        
        // Parse status line
        let status_parts: Vec<&str> = status_line.trim().split_whitespace().collect();
        if status_parts.len() < 3 {
            return Err(NeksisError::NetworkError("Invalid HTTP response".to_string()));
        }
        
        let status_code = status_parts[1].parse::<u16>()
            .map_err(|_| NeksisError::NetworkError("Invalid status code".to_string()))?;
        let status_text = status_parts[2..].join(" ");
        
        // Read headers
        let mut headers = HashMap::new();
        let mut content_length = 0;
        
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)
                .map_err(|e| NeksisError::NetworkError(format!("Failed to read headers: {}", e)))?;
            
            if line.trim().is_empty() {
                break;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                
                if key.to_lowercase() == "content-length" {
                    content_length = value.parse::<usize>().unwrap_or(0);
                }
                
                headers.insert(key, value);
            }
        }
        
        // Read body
        let mut body = String::new();
        if content_length > 0 {
            let mut buffer = vec![0; content_length];
            reader.read_exact(&mut buffer)
                .map_err(|e| NeksisError::NetworkError(format!("Failed to read body: {}", e)))?;
            body = String::from_utf8_lossy(&buffer).to_string();
        }
        
        Ok(HttpResponse {
            status_code,
            status_text,
            headers,
            body,
        })
    }
    
    pub fn get(&self, url: &str) -> NeksisResult<HttpResponse> {
        self.send(HttpRequest::get(url))
    }
    
    pub fn post(&self, url: &str, body: &str) -> NeksisResult<HttpResponse> {
        self.send(HttpRequest::post(url).body(body))
    }
}

/// TCP Server
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    pub fn bind(addr: &str) -> NeksisResult<Self> {
        let listener = TcpListener::bind(addr)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to bind TCP server: {}", e)))?;
        
        Ok(Self { listener })
    }
    
    pub fn accept(&self) -> NeksisResult<(TcpStream, SocketAddr)> {
        self.listener.accept()
            .map_err(|e| NeksisError::NetworkError(format!("Failed to accept connection: {}", e)))
    }
    
    pub fn local_addr(&self) -> NeksisResult<SocketAddr> {
        self.listener.local_addr()
            .map_err(|e| NeksisError::NetworkError(format!("Failed to get local address: {}", e)))
    }
}

/// TCP Client
pub struct TcpClient {
    stream: TcpStream,
}

impl TcpClient {
    pub fn connect(addr: &str) -> NeksisResult<Self> {
        let stream = TcpStream::connect(addr)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to connect: {}", e)))?;
        
        Ok(Self { stream })
    }
    
    pub fn connect_timeout(addr: &str, timeout: Duration) -> NeksisResult<Self> {
        let socket_addr = addr.parse::<SocketAddr>()
            .map_err(|e| NeksisError::NetworkError(format!("Invalid address: {}", e)))?;
        
        let stream = TcpStream::connect_timeout(&socket_addr, timeout)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to connect: {}", e)))?;
        
        Ok(Self { stream })
    }
    
    pub fn send(&mut self, data: &[u8]) -> NeksisResult<()> {
        self.stream.write_all(data)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to send data: {}", e)))
    }
    
    pub fn receive(&mut self, buffer: &mut [u8]) -> NeksisResult<usize> {
        self.stream.read(buffer)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to receive data: {}", e)))
    }
    
    pub fn send_string(&mut self, data: &str) -> NeksisResult<()> {
        self.send(data.as_bytes())
    }
    
    pub fn receive_string(&mut self, max_size: usize) -> NeksisResult<String> {
        let mut buffer = vec![0; max_size];
        let bytes_read = self.receive(&mut buffer)?;
        buffer.truncate(bytes_read);
        String::from_utf8(buffer)
            .map_err(|e| NeksisError::NetworkError(format!("Invalid UTF-8: {}", e)))
    }
}

/// UDP Socket
pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    pub fn bind(addr: &str) -> NeksisResult<Self> {
        let socket = UdpSocket::bind(addr)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to bind UDP socket: {}", e)))?;
        
        Ok(Self { socket })
    }
    
    pub fn send_to(&self, data: &[u8], addr: &str) -> NeksisResult<usize> {
        self.socket.send_to(data, addr)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to send UDP data: {}", e)))
    }
    
    pub fn receive_from(&self, buffer: &mut [u8]) -> NeksisResult<(usize, SocketAddr)> {
        self.socket.recv_from(buffer)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to receive UDP data: {}", e)))
    }
    
    pub fn send_string_to(&self, data: &str, addr: &str) -> NeksisResult<usize> {
        self.send_to(data.as_bytes(), addr)
    }
    
    pub fn receive_string(&self, max_size: usize) -> NeksisResult<(String, SocketAddr)> {
        let mut buffer = vec![0; max_size];
        let (bytes_read, addr) = self.receive_from(&mut buffer)?;
        buffer.truncate(bytes_read);
        let data = String::from_utf8(buffer)
            .map_err(|e| NeksisError::NetworkError(format!("Invalid UTF-8: {}", e)))?;
        Ok((data, addr))
    }
    
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> NeksisResult<()> {
        self.socket.set_read_timeout(timeout)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to set read timeout: {}", e)))
    }
    
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> NeksisResult<()> {
        self.socket.set_write_timeout(timeout)
            .map_err(|e| NeksisError::NetworkError(format!("Failed to set write timeout: {}", e)))
    }
}

/// Convenience functions
pub fn http_get(url: &str) -> NeksisResult<HttpResponse> {
    HttpClient::new().get(url)
}

pub fn http_post(url: &str, body: &str) -> NeksisResult<HttpResponse> {
    HttpClient::new().post(url, body)
}

pub fn tcp_connect(addr: &str) -> NeksisResult<TcpClient> {
    TcpClient::connect(addr)
}

pub fn tcp_listen(addr: &str) -> NeksisResult<TcpServer> {
    TcpServer::bind(addr)
}

pub fn udp_bind(addr: &str) -> NeksisResult<UdpClient> {
    UdpClient::bind(addr)
}

/// URL utilities
pub fn parse_url(url: &str) -> NeksisResult<(String, String, u16, String)> {
    // Returns (protocol, host, port, path)
    if !url.contains("://") {
        return Err(NeksisError::ParseError("URL must contain protocol".to_string()));
    }
    
    let parts: Vec<&str> = url.splitn(2, "://").collect();
    let protocol = parts[0];
    let rest = parts[1];
    
    let (host_port, path) = if let Some(slash_pos) = rest.find('/') {
        (&rest[..slash_pos], &rest[slash_pos..])
    } else {
        (rest, "/")
    };
    
    let (host, port) = if let Some(colon_pos) = host_port.find(':') {
        let host = &host_port[..colon_pos];
        let port_str = &host_port[colon_pos + 1..];
        let port = port_str.parse::<u16>()
            .map_err(|_| NeksisError::ParseError("Invalid port number".to_string()))?;
        (host, port)
    } else {
        let port = match protocol {
            "http" => 80,
            "https" => 443,
            "ftp" => 21,
            _ => 80,
        };
        (host_port, port)
    };
    
    Ok((protocol.to_string(), host.to_string(), port, path.to_string()))
}

pub fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok()
}

pub fn is_valid_ipv4(ip: &str) -> bool {
    ip.parse::<Ipv4Addr>().is_ok()
}

pub fn is_valid_ipv6(ip: &str) -> bool {
    ip.parse::<Ipv6Addr>().is_ok()
}
