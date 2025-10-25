# libcommunicator

## Overview

libcommunicator is a Rust library that provides the core communication layer for the Communicator project. It handles connections and interactions with multiple chat platforms through a unified API.

## Architecture

### Core Design
- Written in Rust for performance, safety, and reliability
- Exposes a C-compatible FFI (Foreign Function Interface) for cross-language support
- Builds as a dynamic library (.so on Linux, .dylib on macOS, .dll on Windows)
- Provides bindings for C, C++, and Go

### Platform Abstraction
- Modular platform adapters for different chat services
- Initial implementation: Slack
- Future platforms: Mattermost, Discord, Microsoft Teams, etc.

## Project Structure

```
libcommunicator/
├── claude.md              # This file
├── Cargo.toml             # Rust project manifest
├── src/
│   ├── lib.rs            # Main library entry point
│   ├── ffi.rs            # C-compatible FFI exports
│   ├── platforms/        # Platform-specific implementations
│   │   ├── mod.rs
│   │   └── slack/        # Slack integration
│   ├── types/            # Common data structures
│   └── error.rs          # Error handling
├── include/              # C header files for FFI
└── examples/             # Usage examples for different languages
```

## Key Components

### FFI Layer
- Exposes C-compatible functions for library consumers
- Handles memory management across language boundaries
- Uses opaque pointers for Rust types
- Provides error handling through return codes

### Platform Adapters
Each platform adapter implements:
- Authentication and connection management
- Message sending and receiving
- Channel/conversation management
- User management
- Real-time event handling (websockets, webhooks, etc.)

### Core Types
- Connection handles
- Message structures
- User and channel information
- Event callbacks

## Building

```bash
# Build the library
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

## FFI Bindings

### C/C++
- Header files in `include/` directory
- Link against the dynamic library

### Go
- Uses cgo for FFI
- Go bindings wrapper around C interface

## API Design Principles

- Thread-safe operations
- Async-first design with callback support
- Graceful error handling
- Minimal dependencies
- Clear ownership semantics across FFI boundary

## Dependencies

Key Rust crates:
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` / `serde_json` - Serialization
- `tokio-tungstenite` - WebSocket support
- Platform-specific SDKs as needed

## Error Handling

- Rust Result types internally
- C-style error codes for FFI boundary
- Detailed error messages accessible through API

## Current Status

- Initial project setup
- Slack platform adapter in development
- FFI layer design in progress

## Development Notes

- Use `cbindgen` to generate C headers from Rust code
- Memory allocated by Rust must be freed by Rust
- All FFI functions should be marked `#[no_mangle]` and `extern "C"`
- Document all public FFI functions thoroughly
