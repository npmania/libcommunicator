package libcommunicator

/*
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"encoding/json"
)

// UserPreference represents a single user preference setting
type UserPreference struct {
	UserID   string `json:"user_id"`
	Category string `json:"category"`
	Name     string `json:"name"`
	Value    string `json:"value"`
}

// NotificationLevel represents the notification level for a channel
type NotificationLevel string

const (
	NotificationLevelAll     NotificationLevel = "all"
	NotificationLevelMention NotificationLevel = "mention"
	NotificationLevelNone    NotificationLevel = "none"
)

// ChannelNotifyProps represents channel notification properties
type ChannelNotifyProps struct {
	Desktop               *string `json:"desktop,omitempty"`
	Push                  *string `json:"push,omitempty"`
	Email                 *string `json:"email,omitempty"`
	MarkUnread            *string `json:"mark_unread,omitempty"`
	IgnoreChannelMentions *string `json:"ignore_channel_mentions,omitempty"`
}

// NewChannelNotifyProps creates a new ChannelNotifyProps with default values
func NewChannelNotifyProps() *ChannelNotifyProps {
	return &ChannelNotifyProps{}
}

// WithDesktop sets the desktop notification level
func (c *ChannelNotifyProps) WithDesktop(level NotificationLevel) *ChannelNotifyProps {
	s := string(level)
	c.Desktop = &s
	return c
}

// WithPush sets the push notification level
func (c *ChannelNotifyProps) WithPush(level NotificationLevel) *ChannelNotifyProps {
	s := string(level)
	c.Push = &s
	return c
}

// WithEmail sets the email notification setting
func (c *ChannelNotifyProps) WithEmail(enabled bool) *ChannelNotifyProps {
	var s string
	if enabled {
		s = "true"
	} else {
		s = "false"
	}
	c.Email = &s
	return c
}

// WithMarkUnread sets the mark unread behavior
func (c *ChannelNotifyProps) WithMarkUnread(level NotificationLevel) *ChannelNotifyProps {
	s := string(level)
	c.MarkUnread = &s
	return c
}

// GetUserPreferences retrieves all preferences for a user
func (p *Platform) GetUserPreferences(userID string) ([]UserPreference, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cUserID, freeUserID := cStringFree(userID)
	defer freeUserID()

	cstr := C.communicator_platform_get_user_preferences(p.handle, cUserID)
	if cstr == nil {
		return nil, getLastError()
	}
	defer freeString(cstr)

	jsonStr := C.GoString(cstr)

	var prefs []UserPreference
	if err := json.Unmarshal([]byte(jsonStr), &prefs); err != nil {
		return nil, err
	}

	return prefs, nil
}

// SetUserPreferences sets user preferences
func (p *Platform) SetUserPreferences(userID string, prefs []UserPreference) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	// Marshal preferences to JSON
	jsonBytes, err := json.Marshal(prefs)
	if err != nil {
		return err
	}

	cUserID, freeUserID := cStringFree(userID)
	defer freeUserID()

	cJSON, freeJSON := cStringFree(string(jsonBytes))
	defer freeJSON()

	code := C.communicator_platform_set_user_preferences(p.handle, cUserID, cJSON)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// MuteChannel mutes a channel for the current user
func (p *Platform) MuteChannel(channelID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	cChannelID, free := cStringFree(channelID)
	defer free()

	code := C.communicator_platform_mute_channel(p.handle, cChannelID)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// UnmuteChannel unmutes a channel for the current user
func (p *Platform) UnmuteChannel(channelID string) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	cChannelID, free := cStringFree(channelID)
	defer free()

	code := C.communicator_platform_unmute_channel(p.handle, cChannelID)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// UpdateChannelNotifyProps updates channel notification properties
func (p *Platform) UpdateChannelNotifyProps(channelID string, props *ChannelNotifyProps) error {
	if p.handle == nil {
		return ErrInvalidHandle
	}

	// Marshal props to JSON
	jsonBytes, err := json.Marshal(props)
	if err != nil {
		return err
	}

	cChannelID, freeChannelID := cStringFree(channelID)
	defer freeChannelID()

	cJSON, freeJSON := cStringFree(string(jsonBytes))
	defer freeJSON()

	code := C.communicator_platform_update_channel_notify_props(p.handle, cChannelID, cJSON)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// Helper function to create a muted channel configuration
func MutedChannelNotifyProps() *ChannelNotifyProps {
	desktop := "none"
	push := "none"
	email := "false"
	markUnread := "mention"
	ignoreMentions := "on"

	return &ChannelNotifyProps{
		Desktop:               &desktop,
		Push:                  &push,
		Email:                 &email,
		MarkUnread:            &markUnread,
		IgnoreChannelMentions: &ignoreMentions,
	}
}

// Helper function to create an unmuted (default) channel configuration
func UnmutedChannelNotifyProps() *ChannelNotifyProps {
	desktop := "default"
	push := "default"
	email := "default"
	markUnread := "all"
	ignoreMentions := "off"

	return &ChannelNotifyProps{
		Desktop:               &desktop,
		Push:                  &push,
		Email:                 &email,
		MarkUnread:            &markUnread,
		IgnoreChannelMentions: &ignoreMentions,
	}
}
