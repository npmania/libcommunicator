# libcommunicator

A Rust library that gives you a unified API for talking to different chat platforms. Currently focused on Mattermost, with plans for Slack, Discord, and others.

This isn't just another API wrapper - it's designed as a proper dynamic library with C FFI bindings, making it usable from any language that can call C functions (which is basically everything).

## Why?

Most chat platform libraries are tied to specific languages or frameworks. This library takes a different approach: build a solid Rust core and expose it through FFI. You get Rust's safety and performance, but you're not locked into Rust for your application.

## Platform Support

Currently implemented:
- [x] **Mattermost** - Production-ready for core messaging

Planned:
- [ ] **Slack**
- [ ] **Discord**
- [ ] **Microsoft Teams**

## Feature Checklist

Features are listed generically below. Currently only Mattermost is implemented.

**Messaging:**
- [x] Send/receive/edit/delete messages (Mattermost)
- [x] Message pagination (Mattermost)
- [x] Threaded conversations (Mattermost)
- [x] Direct messages and group channels (Mattermost)
- [x] Reactions and emoji (Mattermost)
- [x] Pin messages (Mattermost)
- [x] Typing indicators (Mattermost)
- [x] Message search (Mattermost)

**Channels/Conversations:**
- [x] List channels (Mattermost)
- [x] Get channel info (Mattermost)
- [x] Create DM/group channels (Mattermost)
- [x] Manage members (Mattermost)
- [x] Search channels (Mattermost)
- [x] Channel read state tracking (Mattermost)
- [ ] Create/update/delete channels

**Users:**
- [x] Get user info (Mattermost)
- [x] Batch user lookups (Mattermost)
- [x] User presence/status (Mattermost)
- [x] Custom status (Mattermost)
- [x] Search users (Mattermost)

**Workspaces/Teams:**
- [x] List workspaces (Mattermost: teams)
- [x] Get workspace info (Mattermost: teams)
- [x] Switch active workspace (Mattermost: teams)

**Files:**
- [x] Upload files (Mattermost)
- [x] Download files (Mattermost)
- [x] File thumbnails (Mattermost)
- [x] File metadata (Mattermost)

**Authentication:**
- [x] Username/password (Mattermost)
- [x] Token-based auth (Mattermost)
- [x] Multi-factor auth (Mattermost)
- [ ] OAuth 2.0
- [ ] Session management

**Real-time Events:**
- [x] WebSocket streaming (Mattermost)
- [x] Auto-reconnection (Mattermost)
- [x] Event polling (Mattermost)
- [x] Full event coverage (Mattermost)

**Notifications & Preferences:**
- [x] Get/set preferences (Mattermost)
- [x] Mute/unmute channels (Mattermost)
- [x] Notification settings (Mattermost)

**Platform Infrastructure:**
- [x] Rate limiting with retry (Mattermost)
- [x] Response caching (Mattermost)
- [x] Structured errors (Mattermost)
- [ ] Request retry for failures
- [ ] Connection pooling
- [ ] Configuration API

**Integrations:**
- [ ] Webhooks
- [ ] Custom commands
- [ ] Interactive messages
- [ ] Bot accounts

**Developer Tools:**
- [ ] Pagination helpers
- [ ] Batch operations
- [ ] Comprehensive docs
- [ ] Test coverage
- [ ] Enhanced FFI errors

## Building

```bash
cargo build --release
```

The compiled library ends up in `target/release/`:
- Linux: `libcommunicator.so`
- macOS: `libcommunicator.dylib`
- Windows: `communicator.dll`

## Testing

```bash
cargo test
```

## C API

The C API is documented in `include/communicator.h`. It covers:

- Library initialization (`communicator_init`, `communicator_cleanup`)
- Error handling (error codes, error messages)
- Platform creation and connection (Mattermost)
- All messaging operations (send, get, search)
- Channel and user management
- File operations
- Real-time events (subscribe, poll, unsubscribe)
- Thread operations
- Reactions
- User preferences and notifications

## Language Bindings

### Go

Full Go bindings with idiomatic API. See `bindings/go/README.md` for details.

Quick example:
```go
import comm "libcommunicator"

comm.Init()
defer comm.Cleanup()

platform, _ := comm.NewMattermostPlatform("https://mattermost.example.com")
defer platform.Destroy()

config := comm.NewPlatformConfig(serverURL).
    WithToken("your-token").
    WithTeamID("team-id")

platform.Connect(config)
defer platform.Disconnect()

channels, _ := platform.GetChannels()
platform.SendMessage(channels[0].ID, "Hello!")
```

Event handling with channels:
```go
stream, _ := platform.NewEventStream(ctx, 100)
defer stream.Close()

router := comm.NewEventRouter()
router.OnMessagePosted(func(event *comm.Event) {
    fmt.Printf("New message: %+v\n", event)
})
router.Run(ctx, stream)
```

### Other Languages

The C FFI means you can use this from pretty much any language:
- **C/C++**: Direct usage via `communicator.h`
- **Python**: Via `ctypes` or `cffi`
- **Ruby**: Via `fiddle` or `ffi` gem
- **Node.js**: Via `node-ffi` or `N-API`
- **Java**: Via JNI

## Design Philosophy

This is a dynamic library, not an application. That means:

- **Silent by default**: No stdout/stderr output (libraries shouldn't pollute the host app's output)
- **Error handling via return values**: Not via logging or panics
- **Memory management**: Rust allocates, Rust frees (use the provided free functions)
- **Thread-safe**: All operations can be called from multiple threads
- **Async runtime**: Uses Tokio internally but presents a sync FFI (for maximum compatibility)

## Examples

See the `bindings/go/examples/` directory:
- `mattermost_demo`: Complete feature demonstration
- `simple_bot`: Interactive bot that responds to commands

The examples show real-world usage: authentication, sending messages, handling events, file uploads, etc.

## Architecture

```
libcommunicator/
├── src/
│   ├── lib.rs                    # FFI exports and initialization
│   ├── error.rs                  # Error types and conversion
│   ├── runtime.rs                # Tokio runtime management
│   ├── platforms/
│   │   └── mattermost/
│   │       ├── client.rs         # HTTP client with rate limiting
│   │       ├── websocket.rs      # WebSocket with auto-reconnect
│   │       ├── auth.rs           # Authentication (password, token, MFA)
│   │       ├── messages.rs       # Message operations
│   │       ├── channels.rs       # Channel management
│   │       ├── users.rs          # User operations
│   │       ├── files.rs          # File upload/download
│   │       ├── threads.rs        # Thread operations
│   │       ├── reactions.rs      # Reaction management
│   │       ├── teams.rs          # Team operations
│   │       ├── search.rs         # Search functionality
│   │       ├── preferences.rs    # User preferences
│   │       ├── cache.rs          # Multi-layer cache
│   │       └── types.rs          # Mattermost type definitions
├── include/
│   └── communicator.h            # C API header
├── bindings/
│   └── go/                       # Go bindings
└── examples/                     # Usage examples
```

## Contributing

The codebase follows standard Rust conventions. Some specific notes:

- Run `cargo clippy` before committing
- Use `cargo fmt` for formatting
- The FFI layer should stay platform-agnostic (don't expose Mattermost-specific types in `lib.rs`)
- Memory allocated in Rust must be freed in Rust (provide free functions for all allocations)
- Document all public FFI functions with examples
- This is a dynamic library - never write to stdout/stderr (use error return values instead)
