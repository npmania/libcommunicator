package libcommunicator

/*
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"encoding/json"
	"runtime"
	"unsafe"
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

// ConnectWithMFA connects to the platform with Multi-Factor Authentication
// This is a convenience method for platforms that require MFA during login.
// The config should include mfa_token in the Credentials map.
//
// Example:
//   config := &libcommunicator.PlatformConfig{
//       Server: "https://mattermost.example.com",
//       Credentials: map[string]string{
//           "login_id":  "user@example.com",
//           "password":  "password123",
//           "mfa_token": "123456",  // 6-digit MFA code
//       },
//   }
//   err := platform.ConnectWithMFA(config)
func (p *Platform) ConnectWithMFA(config *PlatformConfig) error {
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

	code := C.communicator_platform_connect_with_mfa(p.handle, cs)
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

// RequestAllStatuses requests statuses for all users via WebSocket (async operation)
//
// This is a non-blocking operation that returns immediately with a sequence number.
// The actual status data will arrive later as a Response event with matching SeqReply.
// Requires an active WebSocket connection (call SubscribeEvents first).
//
// Returns the sequence number on success, or error on failure.
func (p *Platform) RequestAllStatuses() (int64, error) {
	if p.handle == nil {
		return -1, ErrInvalidHandle
	}

	seq := C.communicator_platform_request_all_statuses(p.handle)
	if seq == -1 {
		return -1, getLastError()
	}

	return int64(seq), nil
}

// RequestUsersStatuses requests statuses for specific users via WebSocket (async operation)
//
// This is a non-blocking operation that returns immediately with a sequence number.
// The actual status data will arrive later as a Response event with matching SeqReply.
// Requires an active WebSocket connection (call SubscribeEvents first).
//
// Parameters:
//   - userIDs: List of user IDs to get statuses for
//
// Returns the sequence number on success, or error on failure.
func (p *Platform) RequestUsersStatuses(userIDs []string) (int64, error) {
	if p.handle == nil {
		return -1, ErrInvalidHandle
	}

	// Marshal user IDs to JSON
	jsonBytes, err := json.Marshal(userIDs)
	if err != nil {
		return -1, err
	}

	cs, free := cStringFree(string(jsonBytes))
	defer free()

	seq := C.communicator_platform_request_users_statuses(p.handle, cs)
	if seq == -1 {
		return -1, getLastError()
	}

	return int64(seq), nil
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

// SendReply sends a reply to a message (threaded conversation)
func (p *Platform) SendReply(channelID, text, rootID string) (*Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	csText, freeText := cStringFree(text)
	defer freeText()

	csRootID, freeRootID := cStringFree(rootID)
	defer freeRootID()

	cstr := C.communicator_platform_send_reply(p.handle, csChannelID, csText, csRootID)
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

// UpdateMessage updates/edits a message
func (p *Platform) UpdateMessage(messageID, newText string) (*Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csMessageID, freeMessageID := cStringFree(messageID)
	defer freeMessageID()

	csText, freeText := cStringFree(newText)
	defer freeText()

	cstr := C.communicator_platform_update_message(p.handle, csMessageID, csText)
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

// DeleteMessage deletes a message
func (p *Platform) DeleteMessage(messageID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	cs, free := cStringFree(messageID)
	defer free()

	code := C.communicator_platform_delete_message(p.handle, cs)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// GetMessage gets a specific message by ID
func (p *Platform) GetMessage(messageID string) (*Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(messageID)
	defer free()

	cstr := C.communicator_platform_get_message(p.handle, cs)
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

// SearchMessages searches for messages
func (p *Platform) SearchMessages(query string, limit uint32) ([]Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(query)
	defer free()

	cstr := C.communicator_platform_search_messages(p.handle, cs, C.uint32_t(limit))
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

// GetMessagesBefore gets messages before a specific message (pagination)
func (p *Platform) GetMessagesBefore(channelID, beforeID string, limit uint32) ([]Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	csBeforeID, freeBeforeID := cStringFree(beforeID)
	defer freeBeforeID()

	cstr := C.communicator_platform_get_messages_before(p.handle, csChannelID, csBeforeID, C.uint32_t(limit))
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

// GetMessagesAfter gets messages after a specific message (pagination)
func (p *Platform) GetMessagesAfter(channelID, afterID string, limit uint32) ([]Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	csAfterID, freeAfterID := cStringFree(afterID)
	defer freeAfterID()

	cstr := C.communicator_platform_get_messages_after(p.handle, csChannelID, csAfterID, C.uint32_t(limit))
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

// AddReaction adds a reaction to a message
func (p *Platform) AddReaction(messageID, emojiName string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csMessageID, freeMessageID := cStringFree(messageID)
	defer freeMessageID()

	csEmojiName, freeEmojiName := cStringFree(emojiName)
	defer freeEmojiName()

	result := C.communicator_platform_add_reaction(p.handle, csMessageID, csEmojiName)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// RemoveReaction removes a reaction from a message
func (p *Platform) RemoveReaction(messageID, emojiName string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csMessageID, freeMessageID := cStringFree(messageID)
	defer freeMessageID()

	csEmojiName, freeEmojiName := cStringFree(emojiName)
	defer freeEmojiName()

	result := C.communicator_platform_remove_reaction(p.handle, csMessageID, csEmojiName)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// PinPost pins a message/post to its channel
func (p *Platform) PinPost(messageID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csMessageID, freeMessageID := cStringFree(messageID)
	defer freeMessageID()

	result := C.communicator_platform_pin_post(p.handle, csMessageID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// UnpinPost unpins a message/post from its channel
func (p *Platform) UnpinPost(messageID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csMessageID, freeMessageID := cStringFree(messageID)
	defer freeMessageID()

	result := C.communicator_platform_unpin_post(p.handle, csMessageID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// GetPinnedPosts gets all pinned messages/posts for a channel
func (p *Platform) GetPinnedPosts(channelID string) ([]Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	cstr := C.communicator_platform_get_pinned_posts(p.handle, csChannelID)
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

// GetEmojis retrieves a list of custom emojis from the platform
func (p *Platform) GetEmojis(page, perPage uint32) ([]Emoji, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cstr := C.communicator_platform_get_emojis(p.handle, C.uint32_t(page), C.uint32_t(perPage))
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var emojis []Emoji
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &emojis); err != nil {
		return nil, err
	}

	return emojis, nil
}

// GetChannelByName gets a channel by name
func (p *Platform) GetChannelByName(teamID, channelName string) (*Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csTeamID, freeTeamID := cStringFree(teamID)
	defer freeTeamID()

	csChannelName, freeChannelName := cStringFree(channelName)
	defer freeChannelName()

	cstr := C.communicator_platform_get_channel_by_name(p.handle, csTeamID, csChannelName)
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

// CreateGroupChannel creates a group direct message channel
func (p *Platform) CreateGroupChannel(userIDs []string) (*Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	// Marshal user IDs to JSON
	jsonBytes, err := json.Marshal(userIDs)
	if err != nil {
		return nil, err
	}

	cs, free := cStringFree(string(jsonBytes))
	defer free()

	cstr := C.communicator_platform_create_group_channel(p.handle, cs)
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

// AddChannelMember adds a user to a channel
func (p *Platform) AddChannelMember(channelID, userID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	csUserID, freeUserID := cStringFree(userID)
	defer freeUserID()

	code := C.communicator_platform_add_channel_member(p.handle, csChannelID, csUserID)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// RemoveChannelMember removes a user from a channel
func (p *Platform) RemoveChannelMember(channelID, userID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	csUserID, freeUserID := cStringFree(userID)
	defer freeUserID()

	code := C.communicator_platform_remove_channel_member(p.handle, csChannelID, csUserID)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// ViewChannel marks a channel as viewed (read) by the current user
func (p *Platform) ViewChannel(channelID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	cs, free := cStringFree(channelID)
	defer free()

	code := C.communicator_platform_view_channel(p.handle, cs)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// GetChannelUnread gets unread message information for a specific channel
func (p *Platform) GetChannelUnread(channelID string) (*ChannelUnread, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(channelID)
	defer free()

	cstr := C.communicator_platform_get_channel_unread(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var unread ChannelUnread
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &unread); err != nil {
		return nil, err
	}

	return &unread, nil
}

// GetTeamUnreads gets unread counts for all channels in a specific team
func (p *Platform) GetTeamUnreads(teamID string) ([]ChannelUnread, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(teamID)
	defer free()

	cstr := C.communicator_platform_get_team_unreads(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var unreads []ChannelUnread
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &unreads); err != nil {
		return nil, err
	}

	return unreads, nil
}

// GetUserByUsername gets a user by username
func (p *Platform) GetUserByUsername(username string) (*User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(username)
	defer free()

	cstr := C.communicator_platform_get_user_by_username(p.handle, cs)
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

// GetUserByEmail gets a user by email
func (p *Platform) GetUserByEmail(email string) (*User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(email)
	defer free()

	cstr := C.communicator_platform_get_user_by_email(p.handle, cs)
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

// GetUsersByIDs gets multiple users by their IDs (batch operation)
func (p *Platform) GetUsersByIDs(userIDs []string) ([]User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	// Marshal user IDs to JSON
	jsonBytes, err := json.Marshal(userIDs)
	if err != nil {
		return nil, err
	}

	cs, free := cStringFree(string(jsonBytes))
	defer free()

	cstr := C.communicator_platform_get_users_by_ids(p.handle, cs)
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

// CustomStatus represents a custom status for a user
type CustomStatus struct {
	Emoji     string `json:"emoji,omitempty"`
	Text      string `json:"text"`
	ExpiresAt *int64 `json:"expires_at,omitempty"` // Unix timestamp
}

// SetCustomStatus sets a custom status message
func (p *Platform) SetCustomStatus(status CustomStatus) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	// Marshal status to JSON
	jsonBytes, err := json.Marshal(status)
	if err != nil {
		return err
	}

	cs, free := cStringFree(string(jsonBytes))
	defer free()

	code := C.communicator_platform_set_custom_status(p.handle, cs)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// RemoveCustomStatus removes/clears the current user's custom status
func (p *Platform) RemoveCustomStatus() error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	code := C.communicator_platform_remove_custom_status(p.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// SetStatus sets the current user's status
// Valid status values: "online", "away", "dnd", "offline"
func (p *Platform) SetStatus(status string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	cs, free := cStringFree(status)
	defer free()

	code := C.communicator_platform_set_status(p.handle, cs)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// GetUserStatus gets a user's status
// Returns the status string: "online", "away", "dnd", "offline", or "unknown"
func (p *Platform) GetUserStatus(userID string) (string, error) {
	if p.handle == nil {
		return "", ErrInvalidHandle
	}

	cs, free := cStringFree(userID)
	defer free()

	cstr := C.communicator_platform_get_user_status(p.handle, cs)
	if cstr == nil {
		return "", getLastError()
	}
	defer freeString(cstr)

	// Parse the JSON response: {"status": "online"}
	var statusResponse struct {
		Status string `json:"status"`
	}
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &statusResponse); err != nil {
		return "", err
	}

	return statusResponse.Status, nil
}

// SendTypingIndicator sends a typing indicator to a channel
// For regular channel typing, pass empty string for parentID
// For thread typing, pass the parent post ID
func (p *Platform) SendTypingIndicator(channelID string, parentID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csChannelID, freeChannel := cStringFree(channelID)
	defer freeChannel()

	var csParentID *C.char
	var freeParent func()
	if parentID != "" {
		csParentID, freeParent = cStringFree(parentID)
		defer freeParent()
	}

	code := C.communicator_platform_send_typing_indicator(p.handle, csChannelID, csParentID)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// GetUsersStatus gets status for multiple users (batch operation)
// Returns a map of user IDs to status strings
func (p *Platform) GetUsersStatus(userIDs []string) (map[string]string, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	// Marshal user IDs to JSON
	jsonBytes, err := json.Marshal(userIDs)
	if err != nil {
		return nil, err
	}

	cs, free := cStringFree(string(jsonBytes))
	defer free()

	cstr := C.communicator_platform_get_users_status(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var statusMap map[string]string
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &statusMap); err != nil {
		return nil, err
	}

	return statusMap, nil
}

// GetTeams gets all teams the user belongs to
func (p *Platform) GetTeams() ([]Team, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cstr := C.communicator_platform_get_teams(p.handle)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var teams []Team
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &teams); err != nil {
		return nil, err
	}

	return teams, nil
}

// GetTeam gets a specific team by ID
func (p *Platform) GetTeam(teamID string) (*Team, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(teamID)
	defer free()

	cstr := C.communicator_platform_get_team(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var team Team
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &team); err != nil {
		return nil, err
	}

	return &team, nil
}

// GetTeamByName gets a team by name
func (p *Platform) GetTeamByName(teamName string) (*Team, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cs, free := cStringFree(teamName)
	defer free()

	cstr := C.communicator_platform_get_team_by_name(p.handle, cs)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	var team Team
	if err := json.Unmarshal([]byte(C.GoString(cstr)), &team); err != nil {
		return nil, err
	}

	return &team, nil
}

// SetTeamID sets the active team/workspace ID
// Pass an empty string or nil pointer to unset the team ID
func (p *Platform) SetTeamID(teamID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	var cs *C.char
	var free func()
	if teamID == "" {
		cs = nil
	} else {
		cs, free = cStringFree(teamID)
		defer free()
	}

	code := C.communicator_platform_set_team_id(p.handle, cs)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// ==============================================================================
// Thread Operations
// ==============================================================================

// GetThread fetches a thread (root post and all replies)
func (p *Platform) GetThread(postID string) ([]Message, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csPostID, freePostID := cStringFree(postID)
	defer freePostID()

	cstr := C.communicator_platform_get_thread(p.handle, csPostID)
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

// FollowThread makes the authenticated user follow a thread
func (p *Platform) FollowThread(threadID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csThreadID, freeThreadID := cStringFree(threadID)
	defer freeThreadID()

	result := C.communicator_platform_follow_thread(p.handle, csThreadID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// UnfollowThread makes the authenticated user unfollow a thread
func (p *Platform) UnfollowThread(threadID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csThreadID, freeThreadID := cStringFree(threadID)
	defer freeThreadID()

	result := C.communicator_platform_unfollow_thread(p.handle, csThreadID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// MarkThreadRead marks a thread as read up to the current time
func (p *Platform) MarkThreadRead(threadID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csThreadID, freeThreadID := cStringFree(threadID)
	defer freeThreadID()

	result := C.communicator_platform_mark_thread_read(p.handle, csThreadID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// MarkThreadUnread marks a thread as unread from a specific post
func (p *Platform) MarkThreadUnread(threadID, postID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csThreadID, freeThreadID := cStringFree(threadID)
	defer freeThreadID()

	csPostID, freePostID := cStringFree(postID)
	defer freePostID()

	result := C.communicator_platform_mark_thread_unread(p.handle, csThreadID, csPostID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// CreateChannel creates a new regular channel (public or private)
func (p *Platform) CreateChannel(teamID, name, displayName string, isPrivate bool) (*Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csTeamID, freeTeamID := cStringFree(teamID)
	defer freeTeamID()

	csName, freeName := cStringFree(name)
	defer freeName()

	csDisplayName, freeDisplayName := cStringFree(displayName)
	defer freeDisplayName()

	var privateInt C.int
	if isPrivate {
		privateInt = 1
	}

	result := C.communicator_platform_create_channel(p.handle, csTeamID, csName, csDisplayName, privateInt)
	if result == nil {
		return nil, getLastError()
	}
	defer C.communicator_free_string(result)

	jsonStr := C.GoString(result)
	var channel Channel
	if err := json.Unmarshal([]byte(jsonStr), &channel); err != nil {
		return nil, &PlatformError{Code: ErrorUnknown, Message: "failed to parse channel JSON: " + err.Error()}
	}

	return &channel, nil
}

// UpdateChannel updates channel information (partial update)
// Pass empty string for fields that should not be updated
func (p *Platform) UpdateChannel(channelID, displayName, purpose, header string) (*Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	var csDisplayName *C.char
	if displayName != "" {
		csDisplayName, _ = cStringFree(displayName)
		defer C.free(unsafe.Pointer(csDisplayName))
	}

	var csPurpose *C.char
	if purpose != "" {
		csPurpose, _ = cStringFree(purpose)
		defer C.free(unsafe.Pointer(csPurpose))
	}

	var csHeader *C.char
	if header != "" {
		csHeader, _ = cStringFree(header)
		defer C.free(unsafe.Pointer(csHeader))
	}

	result := C.communicator_platform_update_channel(p.handle, csChannelID, csDisplayName, csPurpose, csHeader)
	if result == nil {
		return nil, getLastError()
	}
	defer C.communicator_free_string(result)

	jsonStr := C.GoString(result)
	var channel Channel
	if err := json.Unmarshal([]byte(jsonStr), &channel); err != nil {
		return nil, &PlatformError{Code: ErrorUnknown, Message: "failed to parse channel JSON: " + err.Error()}
	}

	return &channel, nil
}

// DeleteChannel deletes (archives) a channel
func (p *Platform) DeleteChannel(channelID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	csChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	result := C.communicator_platform_delete_channel(p.handle, csChannelID)
	if result != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
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
