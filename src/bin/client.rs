use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file if present
    dotenv().ok();
    
    // Default to 127.0.0.1:6379 if not specified
    let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    println!("Connecting to Redis server at {}", addr);
    let stream = TcpStream::connect(addr)?;
    
    println!("Connected! Type Redis commands or 'exit' to quit.");
    println!("Available commands: GET, SET, DEL, EXISTS, EXPIRE, TTL, KEYS, FLUSHALL, PING, HELP");
    println!("Examples:");
    println!("  SET key value");
    println!("  SET key value EX 10  (expire in 10 seconds)");
    println!("  GET key");
    println!("  EXPIRE key 30");
    println!("  TTL key");
    println!("  KEYS *");
    println!("  DEL key");
    println!("  EXISTS key");
    println!("  FLUSHALL");
    
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut input = String::new();
    
    // Create a buffered reader for the stream
    let mut stream_reader = BufReader::new(stream.try_clone()?);
    let mut stream_writer = stream;
    let mut response = String::new();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        // Clear the input buffer
        input.clear();
        reader.read_line(&mut input)?;
        
        let trimmed_input = input.trim();
        
        // Check if user wants to exit
        if trimmed_input.to_lowercase() == "exit" {
            println!("Goodbye!");
            break;
        }
        
        // Send command to server
        stream_writer.write_all(input.as_bytes())?;
        stream_writer.flush()?;
        
        // Read response
        response.clear();
        stream_reader.read_line(&mut response)?;
        
        if response.is_empty() {
            println!("Server closed connection");
            break;
        }
        
        // Print response
        print!("{}", response);
    }
    
    Ok(())
} 