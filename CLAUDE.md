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
- Initial implementations: Slack, Mattermost
- Future platforms: Discord, Microsoft Teams, etc.

## Project Structure

```
libcommunicator/
├── CLAUDE.md              # This file
├── Cargo.toml             # Rust project manifest
├── src/
│   ├── lib.rs            # Main library entry point
│   ├── ffi.rs            # C-compatible FFI exports (planned)
│   ├── platforms/        # Platform-specific implementations
│   │   ├── mod.rs
│   │   ├── slack/        # Slack integration (planned)
│   │   └── mattermost/   # Mattermost integration
│   │       ├── mod.rs
│   │       └── api-spec.yaml  # Mattermost OpenAPI 3.0 specification
│   ├── types/            # Common data structures (planned)
│   └── error.rs          # Error handling (planned)
├── include/              # C header files for FFI
└── examples/             # Usage examples for different languages (planned)
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

#### Mattermost Platform
The Mattermost platform adapter is located in `src/platforms/mattermost/` and provides integration with Mattermost servers.

**API Specification** (`api-spec.yaml`):
- Complete OpenAPI 3.0 specification for Mattermost API v4
- Documents all REST endpoints for users, teams, channels, posts, files, and more
- Includes WebSocket event system documentation
- Authentication methods: Session tokens, Personal Access Tokens, OAuth 2.0
- Base URL pattern: `{your-mattermost-url}/api/v4`

**Key API Features**:
- **Authentication**: Multiple methods including email/password login, SSO (SAML/OAuth), LDAP
- **REST API**: Full CRUD operations for all Mattermost resources
- **WebSocket**: Real-time event delivery at `/api/v4/websocket` endpoint
  - Events: posted, user_added, channel_updated, typing, status_change, etc.
  - Bidirectional communication with sequence numbers
- **Rate Limiting**: Includes X-Ratelimit-* headers for monitoring usage
- **Error Handling**: Standardized JSON error responses with error IDs

**Implementation TODO**:
- HTTP client for REST API operations
- WebSocket client for real-time events
- Authentication manager (session tokens, PATs)
- Type definitions matching API schemas
- Error handling for Mattermost-specific errors

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

- Initial project setup complete
- Platform module structure established
- Mattermost platform:
  - API specification imported (OpenAPI 3.0)
  - Module structure created
  - Implementation pending
- Slack platform adapter: planned
- FFI layer design in progress

## Development Notes

### General
- Use `cbindgen` to generate C headers from Rust code
- Memory allocated by Rust must be freed by Rust
- All FFI functions should be marked `#[no_mangle]` and `extern "C"`
- Document all public FFI functions thoroughly

### Logging and Output Policy

**CRITICAL: This is a dynamic library - NEVER write to stdout/stderr in production code**

As a dynamic library, libcommunicator must operate **silently**. Any logging or output interferes with the consuming application's behavior and violates library design principles.

**Strict Rules**:
- ❌ **NEVER** use `println!` or `eprintln!` in any code
- ❌ **NEVER** use `print!` or `eprint!` macros
- ❌ **NEVER** use `dbg!` macro outside of `#[cfg(test)]`
- ❌ **NEVER** write directly to stdout/stderr
- ❌ **NEVER** use logging frameworks (log, tracing) that output to console by default

**Error Communication**:
- ✅ Return errors via `Result<T, Error>` types
- ✅ Use error codes and detailed error messages in `Error` structs
- ✅ Provide error callbacks through FFI for async operations
- ✅ Store diagnostic information in error types, not console output

**Debugging During Development**:
- Use `#[cfg(test)]` blocks for debug output in tests only
- Use conditional compilation with custom feature flags for development logging
- Use Rust's built-in debugging tools (rust-gdb, rust-lldb) instead of print debugging
- Consider using trace/debug macros behind feature flags that are disabled by default

**Examples**:
```rust
// ❌ BAD - Never do this in library code
eprintln!("Error: Failed to connect: {}", e);
println!("WebSocket connected successfully");

// ✅ GOOD - Return errors through Result types
Err(Error::new(ErrorCode::NetworkError, format!("Failed to connect: {e}")))

// ✅ GOOD - Silent error handling
if let Err(_) = some_operation() {
    // Handle error silently, update internal state
    *connection_state.lock().await = ConnectionState::Disconnected;
}

// ✅ GOOD - Debug output only in tests
#[cfg(test)]
{
    eprintln!("Test debug: connection state = {:?}", state);
}
```

**Why This Matters**:
- Library output interferes with application's stdout/stderr
- Breaks applications that parse stdout (CLI tools, scripts)
- Cannot be disabled or redirected by consuming applications
- Violates the principle of least surprise
- Makes the library unsuitable for production use

### Working with Mattermost API Specification
- The `api-spec.yaml` file in `src/platforms/mattermost/` is the authoritative source for API endpoints
- Use the spec to:
  - Understand available endpoints and their parameters
  - Generate type definitions for request/response bodies
  - Implement proper error handling based on documented error codes
  - Reference WebSocket event types and message formats
- The spec can be viewed in OpenAPI-compatible tools (Swagger UI, Postman, etc.)
- Official Mattermost documentation: https://api.mattermost.com/
- Consider using code generation tools like `openapi-generator` for type scaffolding

### FFI Design Guidelines

**Keep the FFI Generic and Platform-Agnostic**:
- The FFI layer must work for ALL chat platforms (Mattermost, Slack, Discord, Teams, etc.)
- Never expose platform-specific types or functions at the FFI boundary
- Use generic abstractions like `Client`, `Message`, `Channel`, `User` instead of platform-specific ones
- Platform-specific details should be handled internally in Rust, not exposed to FFI consumers

**FFI Update Workflow**:
When adding new functionality:
1. **Design in Rust First**: Implement the feature in Rust with proper abstractions
2. **Evaluate Generality**: Ensure the feature can work across multiple platforms
3. **Add FFI Functions**: Only add to `ffi.rs` if the functionality is generic enough
4. **Update C Headers**: Regenerate headers using `cbindgen` or update manually
5. **Update Go Bindings**: Synchronize Go wrapper functions in `bindings/go/`
6. **Document Changes**: Update relevant documentation and examples

**When to Add FFI Functions**:
- ✅ Generic operations: connect, disconnect, send message, get channels, get users
- ✅ Common callbacks: on_message, on_error, on_connection_status
- ✅ Universal configuration: set timeout, set log level, set credentials
- ❌ Platform-specific features: Mattermost playbooks, Slack workflows, Discord voice
- ❌ Implementation details: HTTP headers, API rate limits, internal caching

**FFI Function Naming Convention**:
```c
// Good: Generic and clear
communicator_client_send_message(client, channel_id, text)
communicator_client_get_channels(client)
communicator_set_callback_message(client, callback)

// Bad: Platform-specific
mattermost_client_create_post(client, post)
slack_send_block_kit_message(client, blocks)
```

### Go Bindings Update Workflow

When the FFI changes, update Go bindings in this order:

1. **Update Low-Level Bindings** (`bindings/go/communicator/ffi.go`):
   - Add new C function declarations
   - Define corresponding Go types that match C types
   - Use proper cgo types: `C.char`, `*C.char`, `C.int`, `unsafe.Pointer`, etc.

2. **Update High-Level API** (`bindings/go/communicator/client.go` or similar):
   - Create idiomatic Go wrapper functions
   - Convert between Go types and C types
   - Handle memory management (freeing C strings, etc.)
   - Add proper error handling (convert C error codes to Go errors)
   - Add Go documentation comments

3. **Update Examples** (`bindings/go/examples/`):
   - Add new example code demonstrating the new functionality
   - Ensure examples are simple and clear
   - Test that examples compile and run

4. **Memory Management Rules**:
   - Go must free any strings allocated by Rust using the provided free functions
   - Use `defer C.free(unsafe.Pointer(cStr))` pattern for C strings
   - Never keep C pointers beyond the function scope without proper lifetime management
   - Use finalizers for long-lived opaque handles: `runtime.SetFinalizer(obj, cleanup)`

5. **Error Handling Pattern**:
```go
// In Go bindings
func (c *Client) SendMessage(channelID, text string) error {
    cChannelID := C.CString(channelID)
    defer C.free(unsafe.Pointer(cChannelID))

    cText := C.CString(text)
    defer C.free(unsafe.Pointer(cText))

    result := C.communicator_client_send_message(c.handle, cChannelID, cText)
    if result != 0 {
        return fmt.Errorf("failed to send message: error code %d", result)
    }
    return nil
}
```

6. **Testing**:
   - Add unit tests where possible
   - Create integration test examples
   - Test on all target platforms (Linux, macOS, Windows)

**Key Principles for Go Bindings**:
- Make the Go API idiomatic (not just a direct C wrapper)
- Use Go error handling (return `error` not error codes)
- Use Go strings (not `*C.char`)
- Use Go slices and maps (not C arrays)
- Hide all `unsafe` operations inside the binding layer
- Provide type safety wherever possible
