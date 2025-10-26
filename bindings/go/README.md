# libcommunicator Go Bindings

Go bindings for libcommunicator - a unified API for multiple chat platforms.

These bindings wrap the Rust library with an idiomatic Go API. You get Rust's speed and safety under the hood, but you write normal Go code.

**Currently supported platforms:**
- Mattermost (production-ready)

**Planned platforms:**
- Slack, Discord, Microsoft Teams

## Why Use This?

- **Actually idiomatic Go**: Not just a thin C wrapper - uses Go channels, contexts, errors, and conventions
- **Real-time events**: WebSocket events flow through Go channels naturally
- **Memory safe**: Resources clean up automatically (but you can also clean up explicitly)
- **Thread safe**: Call from multiple goroutines without worrying
- **Context support**: Cancellation and timeouts work like you'd expect
- **Battle-tested core**: The heavy lifting happens in Rust, which means no segfaults or data races

## Installation

### Prerequisites

1. Build the Rust library first:
```bash
cd ../../  # Go to project root
cargo build --release
```

2. Ensure the library is in your system library path or use `LD_LIBRARY_PATH`:
```bash
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH
```

### Using the Bindings

```bash
cd bindings/go/libcommunicator
go get
```

## Feature Checklist

Currently only Mattermost platform is implemented. Features are described generically.

**Messaging:**
- [x] Send, edit, delete messages (Mattermost)
- [x] Threaded conversations (Mattermost)
- [x] Get messages from channel (Mattermost)
- [x] Search messages (Mattermost)
- [x] Message pagination (Mattermost)

**Channels/Conversations:**
- [x] List all channels (Mattermost)
- [x] Get channel info (Mattermost)
- [x] Create DM/group channels (Mattermost)
- [x] Manage members (Mattermost)
- [x] Search channels (Mattermost)
- [ ] Create/update/delete channels
- [ ] Unread tracking
- [ ] Mark as viewed

**Users:**
- [x] Get user info (Mattermost)
- [x] Batch lookups (Mattermost)
- [x] User presence/status (Mattermost)
- [x] Custom status (Mattermost)

**Workspaces/Teams:**
- [x] List workspaces (Mattermost: teams)
- [x] Get workspace info (Mattermost: teams)
- [x] Set active workspace (Mattermost: teams)

**Threads:**
- [x] Get thread messages (Mattermost)
- [x] Follow/unfollow (Mattermost)
- [x] Read/unread state (Mattermost)

**Reactions:**
- [x] Add/remove reactions (Mattermost)
- [x] Custom emoji (Mattermost)

**Pinned Content:**
- [x] Pin/unpin messages (Mattermost)
- [x] List pinned (Mattermost)

**Files:**
- [x] Upload (Mattermost)
- [x] Download (Mattermost)
- [x] Thumbnails (Mattermost)
- [x] Metadata (Mattermost)
- [ ] Streaming downloads

**Search:**
- [x] Search users (Mattermost)
- [x] Search channels (Mattermost)
- [x] Search messages (Mattermost)

**Preferences & Notifications:**
- [x] Get/set preferences (Mattermost)
- [x] Mute/unmute (Mattermost)
- [x] Notification settings (Mattermost)

**Real-time Events:**
- [x] Event streaming via Go channels (Mattermost)
- [x] Subscribe/unsubscribe (Mattermost)
- [x] Event polling (Mattermost)
- [x] Type-safe routing (Mattermost)
- [x] Typing indicators (Mattermost)

**Authentication:**
- [x] Token auth (Mattermost)
- [x] Password auth (Mattermost)
- [x] Multi-factor auth (Mattermost)
- [ ] OAuth 2.0
- [ ] Session management

**Platform Integrations:**
- [ ] Webhooks
- [ ] Custom commands
- [ ] Bot accounts
- [ ] Interactive messages

**Go-Specific:**
- [ ] Better error types (platform error details)
- [ ] Rate limit exposure
- [ ] Pagination helpers
- [ ] Batch helpers

## Quick Start

```go
package main

import (
    "fmt"
    "log"

    comm "libcommunicator"
)

func main() {
    // Initialize the library
    if err := comm.Init(); err != nil {
        log.Fatal(err)
    }
    defer comm.Cleanup()

    // Create a Mattermost platform instance
    platform, err := comm.NewMattermostPlatform("https://mattermost.example.com")
    if err != nil {
        log.Fatal(err)
    }
    defer platform.Destroy()

    // Connect with token authentication
    config := comm.NewPlatformConfig("https://mattermost.example.com").
        WithToken("your-token-here").
        WithTeamID("team-id")

    if err := platform.Connect(config); err != nil {
        log.Fatal(err)
    }
    defer platform.Disconnect()

    // Get current user
    user, err := platform.GetCurrentUser()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Logged in as: @%s\n", user.Username)

    // Get channels
    channels, err := platform.GetChannels()
    if err != nil {
        log.Fatal(err)
    }

    for _, channel := range channels {
        fmt.Printf("Channel: %s (%s)\n", channel.Name, channel.Type)
    }

    // Send a message
    msg, err := platform.SendMessage(channels[0].ID, "Hello from Go!")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Sent message: %s\n", msg.ID)
}
```

## Event Streaming

Handle real-time events using Go channels:

```go
import (
    "context"
    "fmt"
    "log"

    comm "libcommunicator"
)

func main() {
    // ... initialize and connect ...

    ctx := context.Background()

    // Create event stream
    stream, err := platform.NewEventStream(ctx, 100)
    if err != nil {
        log.Fatal(err)
    }
    defer stream.Close()

    // Create event router
    router := comm.NewEventRouter()

    // Register event handlers
    router.OnMessagePosted(func(event *comm.Event) {
        fmt.Printf("New message: %+v\n", event)
    })

    router.OnUserTyping(func(event *comm.Event) {
        fmt.Printf("User typing: %s\n", event.UserID)
    })

    // Run the router (blocks until context is cancelled)
    if err := router.Run(ctx, stream); err != nil {
        log.Fatal(err)
    }
}
```

## API Reference

This is a high-level overview. Check the code documentation for complete details.

### Library Initialization

```go
// Initialize the library (must be called first)
func Init() error

// Cleanup the library (call when done)
func Cleanup()

// Get version information
func GetVersion() Version
```

### Platform Management

```go
// Create a new Mattermost platform instance
func NewMattermostPlatform(serverURL string) (*Platform, error)

// Connect to the platform
func (p *Platform) Connect(config *PlatformConfig) error

// Disconnect from the platform
func (p *Platform) Disconnect() error

// Check if connected
func (p *Platform) IsConnected() bool

// Get connection information
func (p *Platform) GetConnectionInfo() (*ConnectionInfo, error)

// Destroy the platform (explicit cleanup)
func (p *Platform) Destroy()
```

### Authentication Configuration

```go
// Create a new config
config := NewPlatformConfig(serverURL)

// Token auth (recommended for bots)
config.WithToken("your-personal-access-token").WithTeamID("team-id")

// Password auth
config.WithPassword("user@example.com", "password").WithTeamID("team-id")

// MFA support (if enabled on server)
config.WithPassword("user@example.com", "password").WithMFAToken("123456")
```

### Messages

```go
// Send a message
func (p *Platform) SendMessage(channelID, text string) (*Message, error)

// Send a reply to a message (threaded)
func (p *Platform) SendReply(channelID, text, rootID string) (*Message, error)

// Update a message
func (p *Platform) UpdateMessage(messageID, newText string) (*Message, error)

// Delete a message
func (p *Platform) DeleteMessage(messageID string) error

// Get messages from a channel
func (p *Platform) GetMessages(channelID string, limit uint32) ([]Message, error)

// Get a specific message
func (p *Platform) GetMessage(messageID string) (*Message, error)

// Search messages
func (p *Platform) SearchMessages(query string, limit uint32) ([]Message, error)

// Pagination
func (p *Platform) GetMessagesBefore(channelID, beforeID string, limit uint32) ([]Message, error)
func (p *Platform) GetMessagesAfter(channelID, afterID string, limit uint32) ([]Message, error)
```

### Channels

```go
// Get all channels
func (p *Platform) GetChannels() ([]Channel, error)

// Get a specific channel
func (p *Platform) GetChannel(channelID string) (*Channel, error)

// Get channel by name
func (p *Platform) GetChannelByName(teamID, name string) (*Channel, error)

// Get channel members
func (p *Platform) GetChannelMembers(channelID string) ([]User, error)

// Create a direct message channel
func (p *Platform) CreateDirectChannel(userID string) (*Channel, error)

// Create a group message channel
func (p *Platform) CreateGroupChannel(userIDs []string) (*Channel, error)

// Add/remove members
func (p *Platform) AddChannelMember(channelID, userID string) error
func (p *Platform) RemoveChannelMember(channelID, userID string) error
```

### Users

```go
// Get current user
func (p *Platform) GetCurrentUser() (*User, error)

// Get a specific user
func (p *Platform) GetUser(userID string) (*User, error)

// Get user by username or email
func (p *Platform) GetUserByUsername(username string) (*User, error)
func (p *Platform) GetUserByEmail(email string) (*User, error)

// Batch lookup
func (p *Platform) GetUsersByIDs(userIDs []string) ([]User, error)

// User status
func (p *Platform) GetUserStatus(userID string) (*UserStatus, error)
func (p *Platform) GetUsersStatus(userIDs []string) (map[string]string, error)
func (p *Platform) SetStatus(status string) error // "online", "away", "dnd", "offline"

// Custom status
func (p *Platform) SetCustomStatus(emoji, text string, expiresAt int64) error
func (p *Platform) RemoveCustomStatus() error
```

### Teams

```go
// Get all teams
func (p *Platform) GetTeams() ([]Team, error)

// Get a specific team
func (p *Platform) GetTeam(teamID string) (*Team, error)

// Get team by name
func (p *Platform) GetTeamByName(name string) (*Team, error)

// Set active team
func (p *Platform) SetTeamID(teamID string) error
```

### Threads

```go
// Get a thread (root post + all replies)
func (p *Platform) GetThread(postID string) ([]Message, error)

// Follow/unfollow a thread
func (p *Platform) FollowThread(threadID string) error
func (p *Platform) UnfollowThread(threadID string) error

// Mark as read/unread
func (p *Platform) MarkThreadRead(threadID string) error
func (p *Platform) MarkThreadUnread(threadID, postID string) error
```

### Reactions

```go
// Add a reaction
func (p *Platform) AddReaction(messageID, emojiName string) error

// Remove a reaction
func (p *Platform) RemoveReaction(messageID, emojiName string) error

// Get custom emoji list
func (p *Platform) GetEmojis(page, perPage uint32) ([]Emoji, error)
```

### Pinned Posts

```go
// Pin a post
func (p *Platform) PinPost(messageID string) error

// Unpin a post
func (p *Platform) UnpinPost(messageID string) error

// Get pinned posts in a channel
func (p *Platform) GetPinnedPosts(channelID string) ([]Message, error)
```

### Files

```go
// Upload a file
func (p *Platform) UploadFile(channelID, filePath string) (string, error) // returns file ID

// Download a file
func (p *Platform) DownloadFile(fileID string) ([]byte, error)

// Get file thumbnail
func (p *Platform) GetFileThumbnail(fileID string) ([]byte, error)

// Get file metadata
func (p *Platform) GetFileMetadata(fileID string) (*FileInfo, error)
```

### Search

```go
// Search users
func (p *Platform) SearchUsers(query string) ([]User, error)

// Search channels
func (p *Platform) SearchChannels(query string) ([]Channel, error)

// Search messages (with advanced operators)
func (p *Platform) SearchMessages(query string, limit uint32) ([]Message, error)
```

### Preferences & Notifications

```go
// Get user preferences
func (p *Platform) GetUserPreferences(userID string) (map[string]interface{}, error)

// Set user preferences
func (p *Platform) SetUserPreferences(userID string, prefs map[string]interface{}) error

// Mute/unmute a channel
func (p *Platform) MuteChannel(channelID string) error
func (p *Platform) UnmuteChannel(channelID string) error

// Update channel notification properties
func (p *Platform) UpdateChannelNotifyProps(channelID string, props map[string]interface{}) error
```

### Events

```go
// Create an event stream (buffered channel)
func (p *Platform) NewEventStream(ctx context.Context, bufferSize int) (*EventStream, error)

// Subscribe to WebSocket events
func (p *Platform) SubscribeEvents() error

// Unsubscribe from events
func (p *Platform) UnsubscribeEvents() error

// Poll for a single event (if you're not using EventStream)
func (p *Platform) PollEvent() (*Event, error)

// Send typing indicator
func (p *Platform) SendTypingIndicator(channelID, parentID string) error
```

### Event Router

The EventRouter makes handling WebSocket events easier:

```go
// Create a new event router
router := NewEventRouter()

// Register handlers for specific event types
router.OnMessagePosted(func(event *Event) {
    fmt.Printf("New message: %s\n", event.Data["message"])
})

router.OnMessageUpdated(func(event *Event) { ... })
router.OnMessageDeleted(func(event *Event) { ... })
router.OnUserStatusChanged(func(event *Event) { ... })
router.OnUserTyping(func(event *Event) { ... })
router.OnChannelCreated(func(event *Event) { ... })
router.OnUserJoinedChannel(func(event *Event) { ... })
router.OnUserLeftChannel(func(event *Event) { ... })

// ... and 36 more event types

// Run the router (blocks until context is cancelled)
if err := router.Run(ctx, stream); err != nil {
    log.Fatal(err)
}
```

## Examples

### Mattermost Demo

A comprehensive example demonstrating all core features:

```bash
cd examples/mattermost_demo
go build
./mattermost_demo -server https://mattermost.example.com -team team-id -token your-token
```

### Simple Bot

An interactive bot that responds to commands:

```bash
cd examples/simple_bot
go build
./simple_bot -server https://mattermost.example.com -team team-id -token your-token
```

The bot responds to:
- `!hello` - Greets the user
- `!echo <text>` - Echoes the text back
- `!help` - Shows available commands

## Configuration

### Authentication

Token authentication (recommended for bots and automation):
```go
config := comm.NewPlatformConfig(serverURL).
    WithToken("your-personal-access-token").
    WithTeamID("team-id")
```

Username/password authentication:
```go
config := comm.NewPlatformConfig(serverURL).
    WithPassword("user@example.com", "password").
    WithTeamID("team-id")
```

With MFA (if the server requires it):
```go
config := comm.NewPlatformConfig(serverURL).
    WithPassword("user@example.com", "password").
    WithMFAToken("123456").
    WithTeamID("team-id")
```

The team ID is optional but recommended - it sets the default team for operations.

### CGO Flags

The library uses cgo to interface with the C library. The following flags are set in the Go code:

```go
/*
#cgo LDFLAGS: -L../../../target/release -lcommunicator
#cgo CFLAGS: -I../../../include
*/
```

You may need to adjust these paths if you're using the library in a different location, or set `CGO_LDFLAGS` and `CGO_CFLAGS` environment variables:

```bash
export CGO_LDFLAGS="-L/path/to/libcommunicator/target/release -lcommunicator"
export CGO_CFLAGS="-I/path/to/libcommunicator/include"
```

## Event Types

The library supports all 44 Mattermost WebSocket event types. The `EventRouter` provides convenient handlers for the most common ones:

**Message Events:**
- `OnMessagePosted` - New message in any channel
- `OnMessageUpdated` - Message edited
- `OnMessageDeleted` - Message deleted
- `OnReactionAdded` - Emoji reaction added
- `OnReactionRemoved` - Emoji reaction removed

**User Events:**
- `OnUserStatusChanged` - User online/away/DND/offline status changed
- `OnUserTyping` - User started typing
- `OnUserAdded` - User added to team
- `OnUserRemoved` - User removed from team
- `OnUserUpdated` - User profile updated

**Channel Events:**
- `OnChannelCreated` - New channel created
- `OnChannelDeleted` - Channel deleted
- `OnChannelUpdated` - Channel properties changed
- `OnChannelViewed` - Channel marked as read
- `OnUserJoinedChannel` - User joined a channel
- `OnUserLeftChannel` - User left a channel
- `OnUserAddedToChannel` - User was added to a channel (by someone else)
- `OnUserRemovedFromChannel` - User was removed from a channel

**Thread Events:**
- `OnThreadUpdated` - Thread modified
- `OnThreadReadChanged` - Thread read state changed
- `OnThreadFollowChanged` - Thread follow state changed

**Team Events:**
- `OnTeamUpdated` - Team properties changed

**Preference Events:**
- `OnPreferenceChanged` - User preference changed
- `OnPreferencesChanged` - Multiple preferences changed
- `OnPreferencesDeleted` - Preferences removed

**And more**: The router supports all event types. If there's no specific handler method, you can use the generic event handler to catch everything:

```go
router := comm.NewEventRouter()

// Catch all events
router.SetDefaultHandler(func(event *comm.Event) {
    fmt.Printf("Event: %s, Data: %+v\n", event.Type, event.Data)
})
```

## Error Handling

Errors are normal Go errors - no special handling needed:

```go
user, err := platform.GetCurrentUser()
if err != nil {
    log.Printf("Failed to get user: %v", err)
    return
}
```

The Rust library includes detailed error information (error codes, Mattermost error IDs, request IDs) but currently these aren't fully exposed through the Go API. You get the error message, which is usually enough.

## Memory Management

The bindings handle most memory management automatically:

```go
// This is fine - cleanup happens automatically via finalizers
platform, _ := comm.NewMattermostPlatform(serverURL)

// But this is better - explicit cleanup is more predictable
defer platform.Destroy()
```

**Best practice**: Use `defer` for cleanup. It's more explicit and doesn't rely on GC timing.

```go
comm.Init()
defer comm.Cleanup()

platform, _ := comm.NewMattermostPlatform(serverURL)
defer platform.Destroy()

platform.Connect(config)
defer platform.Disconnect()

stream, _ := platform.NewEventStream(ctx, 100)
defer stream.Close()
```

## Thread Safety

All operations are thread-safe. Go ahead and call from multiple goroutines:

```go
// This is totally fine
go platform.SendMessage(channelID, "Hello from goroutine 1")
go platform.SendMessage(channelID, "Hello from goroutine 2")
go platform.GetChannels()
```

The Rust core uses proper locking, and the Go bindings don't add any thread-local state.

## Real-World Usage Tips

### Building a Bot

For a bot that responds to messages:

1. Connect and subscribe to events
2. Use `EventRouter` to handle message events
3. Keep track of your bot's user ID to avoid responding to yourself
4. Use `defer` to ensure cleanup happens on shutdown

See `examples/simple_bot` for a complete example.

### Handling Disconnections

The WebSocket automatically reconnects if the connection drops. Your event stream will keep working - you might just see a brief gap in events during reconnection.

If you want to detect disconnections, check `GetConnectionInfo()`:

```go
info, _ := platform.GetConnectionInfo()
fmt.Printf("Connected: %v, Server: %s\n", info.Connected, info.ServerURL)
```

### DM Channel IDs

Direct message channel IDs in Mattermost look like this: `user1id__user2id` (two user IDs separated by `__`). The library handles this automatically when you call `CreateDirectChannel()`, but it's useful to know if you're debugging.

### Rate Limiting

The Rust core automatically handles rate limiting with exponential backoff. If you hit rate limits, requests will automatically retry after the appropriate delay. You don't need to do anything special.

### Caching

The library includes a multi-layer cache for users, channels, and teams. This means repeated calls to `GetUser()` or `GetChannel()` are fast. The cache automatically invalidates on WebSocket events, so you always get fresh data when things change.

### Context Cancellation

Event streams respect context cancellation:

```go
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
defer cancel()

stream, _ := platform.NewEventStream(ctx, 100)
router := comm.NewEventRouter()
router.OnMessagePosted(func(event *Event) { /* ... */ })

// This will stop after 5 minutes
router.Run(ctx, stream)
```

### Search Operators

Message search supports advanced operators:

```go
// Search in a specific channel
platform.SearchMessages("in:town-square hello", 50)

// Search from a specific user
platform.SearchMessages("from:@username error", 50)

// Search on a date
platform.SearchMessages("on:2024-01-15 deployment", 50)

// Combine operators
platform.SearchMessages("in:engineering from:@john before:2024-12-31", 50)
```

## Debugging

If something isn't working:

1. **Check the error message**: They're usually pretty descriptive
2. **Verify your token/credentials**: Make sure they have the right permissions
3. **Check the server URL**: Should be like `https://mattermost.example.com` (no trailing slash, no `/api/v4`)
4. **Enable MFA if needed**: If the server requires MFA, you need to provide the token

The Rust library is completely silent (no logging) by design. If you need to debug the underlying library, you'll need to modify the Rust code to add telemetry callbacks.

## Building from Source

If you're modifying the bindings:

```bash
# Build the Rust library first
cd ../..
cargo build --release

# Set library path
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH

# Build the Go bindings
cd bindings/go/libcommunicator
go build

# Build examples
cd ../examples/mattermost_demo
go build
```

On macOS, use `DYLD_LIBRARY_PATH` instead of `LD_LIBRARY_PATH`.

## Common Issues

**"library not found" error**: Make sure `LD_LIBRARY_PATH` (or `DYLD_LIBRARY_PATH` on macOS) includes the directory with the compiled library.

**"undefined reference" errors**: The library wasn't built yet. Run `cargo build --release` in the project root.

**MFA errors**: If you get "MFA required" errors, include the MFA token in your config: `config.WithPassword(email, password).WithMFAToken("123456")`

**WebSocket disconnects**: Normal - the library automatically reconnects. If it keeps disconnecting immediately, check your authentication.

## Contributing

The Go bindings should stay idiomatic - don't just blindly wrap the C API. Some guidelines:

- Use Go error handling (return `error`, not error codes)
- Use Go types (strings, slices, maps - not C types)
- Hide `unsafe` operations inside the binding layer
- Provide proper cleanup with finalizers (but also explicit cleanup methods)
- Document all exported functions and types
- Add examples for non-obvious functionality

See the main project README for general contribution guidelines.

## License

Same as the main libcommunicator project.
