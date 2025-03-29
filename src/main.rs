use bytes::BytesMut;
use dotenv::dotenv;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

// Error types our Redis server might encounter
#[derive(Error, Debug)]
enum RedisError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
}

// Struct to store the value along with expiration time
struct RedisValue {
    value: String,
    expires_at: Option<Instant>,
}

impl RedisValue {
    fn new(value: String, ttl_seconds: Option<u64>) -> Self {
        let expires_at = ttl_seconds.map(|ttl| Instant::now() + Duration::from_secs(ttl));
        
        RedisValue {
            value,
            expires_at,
        }
    }
    
    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Instant::now() > expires_at,
            None => false,
        }
    }
}

// Define our Redis server's state
struct RedisServer {
    data: Arc<Mutex<HashMap<String, RedisValue>>>,
}

impl RedisServer {
    fn new() -> Self {
        let data = Arc::new(Mutex::new(HashMap::new()));
        
        // Start the expiration cleanup task
        let data_clone = data.clone();
        tokio::spawn(async move {
            loop {
                // Clean expired keys every second
                sleep(Duration::from_secs(1)).await;
                RedisServer::cleanup_expired_keys(&data_clone);
            }
        });
        
        RedisServer { data }
    }
    
    // Cleanup expired keys
    fn cleanup_expired_keys(data: &Arc<Mutex<HashMap<String, RedisValue>>>) {
        let mut data = data.lock().unwrap();
        let expired_keys: Vec<String> = data.iter()
            .filter(|(_, value)| value.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            data.remove(&key);
        }
    }

    // Method to handle a client connection
    async fn handle_client(&self, mut socket: TcpStream) -> Result<(), RedisError> {
        let mut buffer = BytesMut::with_capacity(1024);
        
        loop {
            // Read data from the socket
            match socket.read_buf(&mut buffer).await {
                Ok(0) => {
                    // Connection closed
                    return Ok(());
                }
                Ok(n) => {
                    // Process the command
                    let input = String::from_utf8_lossy(&buffer[..n]);
                    let response = self.process_command(&input).await?;
                    
                    // Send response
                    socket.write_all(response.as_bytes()).await?;
                    socket.flush().await?;
                    
                    // Clear the buffer for the next command
                    buffer.clear();
                }
                Err(e) => {
                    error!("Error reading from socket: {}", e);
                    return Err(RedisError::Io(e));
                }
            }
        }
    }

    // Process a command received from a client
    async fn process_command(&self, input: &str) -> Result<String, RedisError> {
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok("ERROR: Empty command\n".to_string());
        }
        
        info!("Processing command: {}", input);
        
        // Command parsing - simple text-based protocol
        match parts[0].to_uppercase().as_str() {
            "GET" => {
                if parts.len() != 2 {
                    return Ok("ERROR: GET requires exactly one argument\n".to_string());
                }
                
                let key = parts[1];
                let data = self.data.lock().unwrap();
                
                match data.get(key) {
                    Some(value) if !value.is_expired() => {
                        Ok(format!("{}\n", value.value))
                    },
                    _ => Ok("(nil)\n".to_string()),
                }
            }
            "SET" => {
                // Basic SET command with optional expiration
                if parts.len() < 3 || parts.len() > 5 {
                    return Ok("ERROR: SET requires two arguments (key value) with optional EX/TTL\n".to_string());
                }
                
                let key = parts[1].to_string();
                let value = parts[2].to_string();
                
                // Check for TTL option (SET key value EX seconds)
                let mut ttl = None;
                if parts.len() >= 4 && parts[3].to_uppercase() == "EX" && parts.len() == 5 {
                    if let Ok(seconds) = parts[4].parse::<u64>() {
                        ttl = Some(seconds);
                    }
                }
                
                let mut data = self.data.lock().unwrap();
                let redis_value = RedisValue::new(value, ttl);
                data.insert(key, redis_value);
                
                Ok("OK\n".to_string())
            }
            "EXPIRE" => {
                if parts.len() != 3 {
                    return Ok("ERROR: EXPIRE requires exactly two arguments\n".to_string());
                }
                
                let key = parts[1];
                
                if let Ok(seconds) = parts[2].parse::<u64>() {
                    let mut data = self.data.lock().unwrap();
                    
                    if let Some(value) = data.get_mut(key) {
                        // Update the expiration time
                        value.expires_at = Some(Instant::now() + Duration::from_secs(seconds));
                        Ok("1\n".to_string())
                    } else {
                        Ok("0\n".to_string())  // Key doesn't exist
                    }
                } else {
                    Ok("ERROR: EXPIRE seconds must be a positive integer\n".to_string())
                }
            }
            "TTL" => {
                if parts.len() != 2 {
                    return Ok("ERROR: TTL requires exactly one argument\n".to_string());
                }
                
                let key = parts[1];
                let data = self.data.lock().unwrap();
                
                match data.get(key) {
                    Some(value) => {
                        match value.expires_at {
                            Some(expires_at) => {
                                let now = Instant::now();
                                if expires_at > now {
                                    let remaining = expires_at.duration_since(now).as_secs();
                                    Ok(format!("{}\n", remaining))
                                } else {
                                    Ok("-2\n".to_string())  // Key expired
                                }
                            },
                            None => Ok("-1\n".to_string()),  // Key exists but has no expiry
                        }
                    },
                    None => Ok("-2\n".to_string()),  // Key doesn't exist
                }
            }
            "DEL" => {
                if parts.len() != 2 {
                    return Ok("ERROR: DEL requires exactly one argument\n".to_string());
                }
                
                let key = parts[1];
                let mut data = self.data.lock().unwrap();
                
                match data.remove(key) {
                    Some(_) => Ok("1\n".to_string()),
                    None => Ok("0\n".to_string()),
                }
            }
            "EXISTS" => {
                if parts.len() != 2 {
                    return Ok("ERROR: EXISTS requires exactly one argument\n".to_string());
                }
                
                let key = parts[1];
                let data = self.data.lock().unwrap();
                
                match data.get(key) {
                    Some(value) if !value.is_expired() => Ok("1\n".to_string()),
                    _ => Ok("0\n".to_string()),
                }
            }
            "KEYS" => {
                if parts.len() != 2 {
                    return Ok("ERROR: KEYS requires exactly one argument\n".to_string());
                }
                
                let pattern = parts[1];
                let data = self.data.lock().unwrap();
                
                // Simple pattern matching (only supporting * wildcard)
                let keys: Vec<String> = if pattern == "*" {
                    // Return all non-expired keys
                    data.iter()
                        .filter(|(_, v)| !v.is_expired())
                        .map(|(k, _)| k.clone())
                        .collect()
                } else {
                    // Return keys that match pattern (simple contains for now)
                    data.iter()
                        .filter(|(k, v)| !v.is_expired() && k.contains(&pattern.replace("*", "")))
                        .map(|(k, _)| k.clone())
                        .collect()
                };
                
                if keys.is_empty() {
                    Ok("(empty list)\n".to_string())
                } else {
                    let result = keys.join("\n");
                    Ok(format!("{}\n", result))
                }
            }
            "FLUSHALL" => {
                let mut data = self.data.lock().unwrap();
                data.clear();
                Ok("OK\n".to_string())
            }
            "PING" => {
                Ok("PONG\n".to_string())
            }
            "HELP" => {
                Ok("Available commands: GET, SET, DEL, EXISTS, EXPIRE, TTL, KEYS, FLUSHALL, PING, HELP\n".to_string())
            }
            _ => {
                Err(RedisError::UnknownCommand(parts[0].to_string()))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if present
    dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    // Default to 6379 (standard Redis port) if not specified
    let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    // Create and bind the TCP listener
    let listener = TcpListener::bind(&addr).await?;
    info!("Redis server listening on {}", addr);
    
    // Create our Redis server instance
    let redis_server = RedisServer::new();
    
    // Accept and handle connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New client connection: {}", addr);
                
                // Clone the server data for this connection
                let server_data = redis_server.data.clone();
                
                // Spawn a new task to handle this client
                tokio::spawn(async move {
                    let server = RedisServer { data: server_data };
                    if let Err(e) = server.handle_client(socket).await {
                        error!("Error handling client {}: {}", addr, e);
                    }
                    info!("Client {} disconnected", addr);
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
