package libcommunicator

/*
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"encoding/json"
	"runtime"
)

// Platform represents a chat platform (Mattermost, Slack, etc.)
type Platform struct {
	handle C.CommunicatorPlatform
}

// NewMattermostPlatform creates a new Mattermost platform instance
func NewMattermostPlatform(serverURL string) (*Platform, error) {
	if err := ensureInitialized(); err != nil {
		return nil, err
	}

	cs, free := cStringFree(serverURL)
	defer free()

	handle := C.communicator_mattermost_create(cs)
	if handle == nil {
		return nil, getLastError()
	}

	p := &Platform{handle: handle}

	// Set up finalizer to ensure cleanup
	runtime.SetFinalizer(p, func(p *Platform) {
		p.Destroy()
	})

	return p, nil
}

// Connect connects to the platform and authenticates
func (p *Platform) Connect(config *PlatformConfig) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	// Marshal config to JSON
	jsonBytes, err := json.Marshal(config)
	if err != nil {
		return err
	}

	cs, free := cStringFree(string(jsonBytes))
	defer free()

	code := C.communicator_platform_connect(p.handle, cs)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// Disconnect disconnects from the platform
func (p *Platform) Disconnect() error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	code := C.communicator_platform_disconnect(p.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// IsConnected returns whether the platform is connected
func (p *Platform) IsConnected() bool {
	if p.handle == nil {
		return false
	}

	result := C.communicator_platform_is_connected(p.handle)
	return result == 1
}

// GetConnectionInfo returns connection information
func (p *Platform) GetConnectionInfo() (*ConnectionInfo, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cstr := C.communicator_platform_get_connection_info(p.handle)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var info ConnectionInfo
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &info); err != nil {
		return nil, err
	}

	return &info, nil
}

// SendMessage sends a message to a channel
func (p *Platform) SendMessage(channelID, text string) (*Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	csText, freeText := cStringFree(text)
	defer freeText()

	cstr := C.communicator_platform_send_message(p.handle, csChannelID, csText)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var msg Message
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &msg); err != nil {
		return nil, err
	}

	return &msg, nil
}

// GetChannels returns all channels for the current user
func (p *Platform) GetChannels() ([]Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cstr := C.communicator_platform_get_channels(p.handle)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var channels []Channel
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &channels); err != nil {
		return nil, err
	}

	return channels, nil
}

// GetChannel returns a specific channel by ID
func (p *Platform) GetChannel(channelID string) (*Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(channelID)
	defer free()

	cstr := C.communicator_platform_get_channel(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var channel Channel
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &channel); err != nil {
		return nil, err
	}

	return &channel, nil
}

// GetMessages returns recent messages from a channel
func (p *Platform) GetMessages(channelID string, limit uint32) ([]Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(channelID)
	defer free()

	cstr := C.communicator_platform_get_messages(p.handle, cs, C.uint32_t(limit))
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var messages []Message
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &messages); err != nil {
		return nil, err
	}

	return messages, nil
}

// GetChannelMembers returns members of a channel
func (p *Platform) GetChannelMembers(channelID string) ([]User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(channelID)
	defer free()

	cstr := C.communicator_platform_get_channel_members(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var users []User
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &users); err != nil {
		return nil, err
	}

	return users, nil
}

// GetUser returns a specific user by ID
func (p *Platform) GetUser(userID string) (*User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(userID)
	defer free()

	cstr := C.communicator_platform_get_user(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var user User
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &user); err != nil {
		return nil, err
	}

	return &user, nil
}

// GetCurrentUser returns the current authenticated user
func (p *Platform) GetCurrentUser() (*User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cstr := C.communicator_platform_get_current_user(p.handle)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var user User
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &user); err != nil {
		return nil, err
	}

	return &user, nil
}

// CreateDirectChannel creates a direct message channel with another user
func (p *Platform) CreateDirectChannel(userID string) (*Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(userID)
	defer free()

	cstr := C.communicator_platform_create_direct_channel(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var channel Channel
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &channel); err != nil {
		return nil, err
	}

	return &channel, nil
}

// SubscribeEvents subscribes to real-time events
func (p *Platform) SubscribeEvents() error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	code := C.communicator_platform_subscribe_events(p.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// UnsubscribeEvents unsubscribes from real-time events
func (p *Platform) UnsubscribeEvents() error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	code := C.communicator_platform_unsubscribe_events(p.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// PollEvent polls for the next event
// Returns nil, nil if no events are available
func (p *Platform) PollEvent() (*Event, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cstr := C.communicator_platform_poll_event(p.handle)
	if cstr == nil {
		// Check if it's an error or just no events
		if C.communicator_last_error_code() != C.COMMUNICATOR_SUCCESS {
			return nil, getLastError()
		}
		return nil, nil // No events available
	}
	defer freeString(cstr)

	var event Event
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &event); err != nil {
		return nil, err
	}

	return &event, nil
}

// Destroy destroys the platform and frees its resources
func (p *Platform) Destroy() {
	if p.handle != nil {
		C.communicator_platform_destroy(p.handle)
		p.handle = nil
	}
}

var (
	// ErrInvalidHandle is returned when the platform handle is invalid
	ErrInvalidHandle = &PlatformError{Code: ErrorNullPointer, Message: "invalid platform handle"}
)

// PlatformError represents a platform error
type PlatformError struct {
	Code    ErrorCode
	Message string
}

func (e *PlatformError) Error() string {
	return e.Message
}
