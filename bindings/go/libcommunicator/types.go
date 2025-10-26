package libcommunicator

import "time"

// ConnectionState represents the state of a platform connection
type ConnectionState string

const (
	StateDisconnected ConnectionState = "disconnected"
	StateConnecting   ConnectionState = "connecting"
	StateConnected    ConnectionState = "connected"
	StateReconnecting ConnectionState = "reconnecting"
	StateError        ConnectionState = "error"
)

// ChannelType represents the type of channel
type ChannelType string

const (
	ChannelTypePublic        ChannelType = "public"
	ChannelTypePrivate       ChannelType = "private"
	ChannelTypeDirectMessage ChannelType = "direct_message"
	ChannelTypeGroupMessage  ChannelType = "group_message"
)

// User represents a user on the platform
type User struct {
	ID       string `json:"id"`
	Username string `json:"username"`
	Email    string `json:"email,omitempty"`
	Name     string `json:"name,omitempty"`
	Status   string `json:"status,omitempty"`
}

// Channel represents a communication channel
type Channel struct {
	ID          string      `json:"id"`
	Name        string      `json:"name"`
	DisplayName string      `json:"display_name,omitempty"`
	Type        ChannelType `json:"type"`
	TeamID      string      `json:"team_id,omitempty"`
}

// TeamType represents the type/visibility of a team
type TeamType string

const (
	TeamTypeOpen   TeamType = "open"
	TeamTypeInvite TeamType = "invite"
)

// Team represents a team/workspace on the platform
type Team struct {
	ID              string      `json:"id"`
	Name            string      `json:"name"`
	DisplayName     string      `json:"display_name"`
	Description     string      `json:"description,omitempty"`
	TeamType        TeamType    `json:"team_type"`
	AllowedDomains  string      `json:"allowed_domains,omitempty"`
	AllowOpenInvite bool        `json:"allow_open_invite"`
	Metadata        interface{} `json:"metadata,omitempty"`
}

// Attachment represents a file attachment
type Attachment struct {
	ID           string  `json:"id"`
	Filename     string  `json:"filename"`
	MimeType     string  `json:"mime_type"`
	Size         uint64  `json:"size"`
	URL          string  `json:"url"`
	ThumbnailURL *string `json:"thumbnail_url,omitempty"` // Added to match Rust
}

// Message represents a chat message
type Message struct {
	ID          string       `json:"id"`
	ChannelID   string       `json:"channel_id"`
	SenderID    string       `json:"sender_id"` // Changed from UserID to match Rust
	Text        string       `json:"text"`
	CreatedAt   time.Time    `json:"created_at"`
	EditedAt    *time.Time   `json:"edited_at,omitempty"` // Changed from UpdatedAt to match Rust
	Attachments []Attachment `json:"attachments,omitempty"`
	Metadata    interface{}  `json:"metadata,omitempty"` // Added to match Rust
}

// Reaction represents an emoji reaction to a message
type Reaction struct {
	UserID    string    `json:"user_id"`
	PostID    string    `json:"post_id"`
	EmojiName string    `json:"emoji_name"`
	CreatedAt time.Time `json:"created_at"`
}

// ConnectionInfo represents connection information
type ConnectionInfo struct {
	State     ConnectionState `json:"state"`
	ServerURL string          `json:"server_url"`
	UserID    string          `json:"user_id,omitempty"`
	TeamID    string          `json:"team_id,omitempty"`
}

// Event represents a platform event
type Event struct {
	Type string      `json:"type"`
	Data interface{} `json:"data,omitempty"`

	// Event-specific fields
	MessageID string `json:"message_id,omitempty"`
	ChannelID string `json:"channel_id,omitempty"`
	UserID    string `json:"user_id,omitempty"`
	Status    string `json:"status,omitempty"`
	State     string `json:"state,omitempty"`
	EmojiName string `json:"emoji_name,omitempty"`
}

// EventType constants
const (
	EventMessagePosted         = "message_posted"
	EventMessageUpdated        = "message_updated"
	EventMessageDeleted        = "message_deleted"
	EventUserStatusChanged     = "user_status_changed"
	EventUserTyping            = "user_typing"
	EventChannelCreated        = "channel_created"
	EventChannelUpdated        = "channel_updated"
	EventChannelDeleted        = "channel_deleted"
	EventUserJoinedChannel     = "user_joined_channel"
	EventUserLeftChannel       = "user_left_channel"
	EventConnectionStateChange = "connection_state_changed"
	EventReactionAdded         = "reaction_added"
	EventReactionRemoved       = "reaction_removed"
)

// PlatformConfig holds configuration for connecting to a platform
type PlatformConfig struct {
	Server      string            `json:"server"`
	Credentials map[string]string `json:"credentials"`
	TeamID      string            `json:"team_id,omitempty"`
}

// NewPlatformConfig creates a new platform configuration
func NewPlatformConfig(serverURL string) *PlatformConfig {
	return &PlatformConfig{
		Server:      serverURL,
		Credentials: make(map[string]string),
	}
}

// WithToken sets token authentication
func (c *PlatformConfig) WithToken(token string) *PlatformConfig {
	c.Credentials["token"] = token
	return c
}

// WithPassword sets username/password authentication
func (c *PlatformConfig) WithPassword(loginID, password string) *PlatformConfig {
	c.Credentials["login_id"] = loginID
	c.Credentials["password"] = password
	return c
}

// WithTeamID sets the team ID
func (c *PlatformConfig) WithTeamID(teamID string) *PlatformConfig {
	c.TeamID = teamID
	return c
}
