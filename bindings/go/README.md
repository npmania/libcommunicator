# libcommunicator Go Bindings

Go bindings for libcommunicator, a unified communication library for chat platforms (Mattermost, Slack, etc.).

## Features

- **Idiomatic Go API**: Clean, type-safe interface following Go conventions
- **FFI Integration**: Uses cgo to interface with the Rust core library
- **Event Streaming**: Go channels and contexts for real-time events
- **Memory Safe**: Automatic resource cleanup with finalizers
- **Thread Safe**: All operations are thread-safe
- **Context Support**: Cancellation and timeout support

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

### Initialization

```go
// Initialize the library (must be called first)
func Init() error

// Cleanup the library (call when done)
func Cleanup()

// Get version information
func GetVersion() Version
```

### Platform

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
```

### Messaging

```go
// Send a message
func (p *Platform) SendMessage(channelID, text string) (*Message, error)

// Get messages from a channel
func (p *Platform) GetMessages(channelID string, limit uint32) ([]Message, error)
```

### Channels

```go
// Get all channels
func (p *Platform) GetChannels() ([]Channel, error)

// Get a specific channel
func (p *Platform) GetChannel(channelID string) (*Channel, error)

// Get channel members
func (p *Platform) GetChannelMembers(channelID string) ([]User, error)

// Create a direct message channel
func (p *Platform) CreateDirectChannel(userID string) (*Channel, error)
```

### Users

```go
// Get current user
func (p *Platform) GetCurrentUser() (*User, error)

// Get a specific user
func (p *Platform) GetUser(userID string) (*User, error)
```

### Events

```go
// Create an event stream
func (p *Platform) NewEventStream(ctx context.Context, bufferSize int) (*EventStream, error)

// Subscribe to events
func (p *Platform) SubscribeEvents() error

// Unsubscribe from events
func (p *Platform) UnsubscribeEvents() error

// Poll for a single event
func (p *Platform) PollEvent() (*Event, error)
```

### Event Router

```go
// Create a new event router
func NewEventRouter() *EventRouter

// Register handlers for specific event types
func (r *EventRouter) OnMessagePosted(handler EventHandler)
func (r *EventRouter) OnMessageUpdated(handler EventHandler)
func (r *EventRouter) OnMessageDeleted(handler EventHandler)
func (r *EventRouter) OnUserStatusChanged(handler EventHandler)
func (r *EventRouter) OnUserTyping(handler EventHandler)
func (r *EventRouter) OnChannelCreated(handler EventHandler)
func (r *EventRouter) OnUserJoinedChannel(handler EventHandler)
func (r *EventRouter) OnUserLeftChannel(handler EventHandler)

// Run the router (blocks until context is cancelled)
func (r *EventRouter) Run(ctx context.Context, stream *EventStream) error
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

Token authentication:
```go
config := comm.NewPlatformConfig(serverURL).
    WithToken("your-token").
    WithTeamID("team-id")
```

Username/password authentication:
```go
config := comm.NewPlatformConfig(serverURL).
    WithPassword("user@example.com", "password").
    WithTeamID("team-id")
```

### CGO Flags

The library uses cgo to interface with the C library. The following flags are set:

```go
/*
#cgo LDFLAGS: -L../../../target/release -lcommunicator
#cgo CFLAGS: -I../../../include
*/
```

You may need to adjust these paths depending on your project structure.

## Error Handling

All functions return idiomatic Go errors:

```go
user, err := platform.GetCurrentUser()
if err != nil {
    log.Printf("Failed to get user: %v", err)
    return
}
```

## Memory Management

Resources are automatically cleaned up when objects are garbage collected, but it's recommended to explicitly call cleanup methods:

```go
platform, _ := comm.NewMattermostPlatform(serverURL)
defer platform.Destroy()  // Explicit cleanup
```

## Thread Safety

All platform operations are thread-safe and can be called from multiple goroutines.

## Contributing

See the main project README for contribution guidelines.

## License

Same as the main libcommunicator project.
