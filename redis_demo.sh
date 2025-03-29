#!/bin/bash

# Default values
PORT=6379

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --port=*)
      PORT="${1#*=}"
      shift
      ;;
    --help)
      echo "Usage: $0 [--port=PORT]"
      echo ""
      echo "Options:"
      echo "  --port=PORT    Set the Redis client port (default: 6379)"
      echo "  --help         Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# Function to run a Redis command and display the result
run_cmd() {
  cmd="$1"
  desc="$2"
  
  echo -e "\n\033[1;34m# $desc\033[0m"
  echo -e "\033[1;32m> $cmd\033[0m"
  
  # Run the command using the client
  REDIS_PORT=$PORT cargo run --bin client <<< "$cmd" | grep -v "Connected" | grep -v "Available commands" | head -n 3
  
  # Add a small delay between commands
  sleep 0.5
}

echo "=== Redis Demo ==="
echo "Running commands on port $PORT"

# Basic commands
run_cmd "PING" "Test connection to server"
run_cmd "SET user:1 John" "Set a simple key-value pair"
run_cmd "GET user:1" "Get the value we just set"
run_cmd "SET visitor:1234 active EX 3" "Set a key with expiration (3 seconds)"
run_cmd "TTL visitor:1234" "Check how much time is left on the expiring key"
run_cmd "KEYS user*" "Find all keys with a pattern"
run_cmd "EXISTS user:1" "Check if a key exists (should be 1)"
run_cmd "DEL user:1" "Delete a key"
run_cmd "EXISTS user:1" "Check if key exists after deletion (should be 0)"
run_cmd "SET temp:1 value1" "Create temporary key 1"
run_cmd "SET temp:2 value2" "Create temporary key 2"
run_cmd "KEYS temp*" "List all temporary keys"
run_cmd "FLUSHALL" "Clear the entire database"
run_cmd "KEYS *" "Verify no keys are left after FLUSHALL"

echo -e "\n\033[1;33mDemo complete! Your Redis server is working correctly.\033[0m" 