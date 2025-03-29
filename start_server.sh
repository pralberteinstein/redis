#!/bin/bash

# Default values
PORT=6379
LOG_LEVEL=info

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --port=*)
      PORT="${1#*=}"
      shift
      ;;
    --log=*)
      LOG_LEVEL="${1#*=}"
      shift
      ;;
    --help)
      echo "Usage: $0 [--port=PORT] [--log=LEVEL]"
      echo ""
      echo "Options:"
      echo "  --port=PORT    Set the Redis server port (default: 6379)"
      echo "  --log=LEVEL    Set logging level (default: info)"
      echo "                 Valid levels: error, warn, info, debug, trace"
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

echo "Starting Redis server on port $PORT with log level $LOG_LEVEL"
REDIS_PORT=$PORT RUST_LOG=$LOG_LEVEL cargo run 