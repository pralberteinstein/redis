use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

struct TestServer {
    server: Child,
}

impl TestServer {
    fn new() -> Self {
        // Start the Redis server
        let server = Command::new("cargo")
            .args(["run", "--quiet"])
            .env("REDIS_PORT", "6380")
            .env("RUST_LOG", "error")
            .spawn()
            .expect("Failed to start Redis server");
        
        // Wait for the server to start up
        sleep(Duration::from_secs(1));
        
        TestServer { server }
    }
    
    fn client(&self) -> TcpStream {
        TcpStream::connect("127.0.0.1:6380").expect("Failed to connect to Redis server")
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Terminate the server
        self.server.kill().expect("Failed to kill Redis server");
    }
}

fn send_command(stream: &mut TcpStream, command: &str) -> String {
    stream.write_all(command.as_bytes()).unwrap();
    stream.flush().unwrap();
    
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    
    response
}

#[test]
fn test_basic_commands() {
    let server = TestServer::new();
    let mut client = server.client();
    
    // Test PING
    let response = send_command(&mut client, "PING\n");
    assert_eq!(response, "PONG\n");
    
    // Test SET/GET
    let response = send_command(&mut client, "SET testkey testvalue\n");
    assert_eq!(response, "OK\n");
    
    let response = send_command(&mut client, "GET testkey\n");
    assert_eq!(response, "testvalue\n");
    
    // Test EXISTS
    let response = send_command(&mut client, "EXISTS testkey\n");
    assert_eq!(response, "1\n");
    
    // Test DEL
    let response = send_command(&mut client, "DEL testkey\n");
    assert_eq!(response, "1\n");
    
    let response = send_command(&mut client, "GET testkey\n");
    assert_eq!(response, "(nil)\n");
}

#[test]
fn test_expiration() {
    let server = TestServer::new();
    let mut client = server.client();
    
    // Test SET with expiration
    let response = send_command(&mut client, "SET expkey value EX 1\n");
    assert_eq!(response, "OK\n");
    
    // Key should exist initially
    let response = send_command(&mut client, "EXISTS expkey\n");
    assert_eq!(response, "1\n");
    
    // Wait for key to expire
    sleep(Duration::from_secs(2));
    
    // Key should no longer exist
    let response = send_command(&mut client, "EXISTS expkey\n");
    assert_eq!(response, "0\n");
}

#[test]
fn test_keys_and_flushall() {
    let server = TestServer::new();
    let mut client = server.client();
    
    // Add several keys
    send_command(&mut client, "SET key1 value1\n");
    send_command(&mut client, "SET key2 value2\n");
    send_command(&mut client, "SET anotherkey value3\n");
    
    // Test KEYS with pattern
    let response = send_command(&mut client, "KEYS key*\n");
    assert!(response.contains("key1"));
    assert!(response.contains("key2"));
    
    // Test FLUSHALL
    let response = send_command(&mut client, "FLUSHALL\n");
    assert_eq!(response, "OK\n");
    
    // Verify all keys are gone
    let response = send_command(&mut client, "KEYS *\n");
    assert_eq!(response, "(empty list)\n");
} 