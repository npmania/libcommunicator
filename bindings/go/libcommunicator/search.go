package libcommunicator

/*
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"encoding/json"
	"unsafe"
)

// UserSearchRequest represents parameters for advanced user search
type UserSearchRequest struct {
	Term           string  `json:"term"`
	TeamID         *string `json:"team_id,omitempty"`
	NotInChannelID *string `json:"not_in_channel_id,omitempty"`
	InChannelID    *string `json:"in_channel_id,omitempty"`
	AllowInactive  *bool   `json:"allow_inactive,omitempty"`
	WithoutTeam    *bool   `json:"without_team,omitempty"`
	Limit          *uint32 `json:"limit,omitempty"`
}

// FileSearchRequest represents parameters for file search
type FileSearchRequest struct {
	Terms          string   `json:"terms"`
	ChannelID      *string  `json:"channel_id,omitempty"`
	Extensions     []string `json:"ext,omitempty"`
	TimeZoneOffset *int32   `json:"time_zone_offset,omitempty"`
}

// PostSearchOptions represents advanced search options for posts
type PostSearchOptions struct {
	Terms                  string `json:"terms"`
	IsOrSearch             bool   `json:"is_or_search"`
	IncludeDeletedChannels bool   `json:"include_deleted_channels"`
	TimeZoneOffset         int32  `json:"time_zone_offset"`
	Page                   uint32 `json:"page"`
	PerPage                uint32 `json:"per_page"`
}

// SearchUsers performs advanced user search with filtering
// Returns a JSON array string of User objects
func (p *Platform) SearchUsers(request *UserSearchRequest) ([]User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	requestJSON, err := json.Marshal(request)
	if err != nil {
		return nil, err
	}

	cRequest := C.CString(string(requestJSON))
	defer C.free(unsafe.Pointer(cRequest))

	result := C.communicator_platform_search_users(p.handle, cRequest)
	if result == nil {
		return nil, getLastError()
	}
	defer C.communicator_free_string(result)

	var users []User
	if err := json.Unmarshal([]byte(C.GoString(result)), &users); err != nil {
		return nil, err
	}

	return users, nil
}

// AutocompleteUsers autocompletes users for mentions
// Pass empty strings for teamID or channelID if not needed
func (p *Platform) AutocompleteUsers(name, teamID, channelID string, limit uint32) ([]User, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	var cTeamID *C.char
	if teamID != "" {
		cTeamID = C.CString(teamID)
		defer C.free(unsafe.Pointer(cTeamID))
	}

	var cChannelID *C.char
	if channelID != "" {
		cChannelID = C.CString(channelID)
		defer C.free(unsafe.Pointer(cChannelID))
	}

	result := C.communicator_platform_autocomplete_users(p.handle, cName, cTeamID, cChannelID, C.uint32_t(limit))
	if result == nil {
		return nil, getLastError()
	}
	defer C.communicator_free_string(result)

	var users []User
	if err := json.Unmarshal([]byte(C.GoString(result)), &users); err != nil {
		return nil, err
	}

	return users, nil
}

// SearchChannels searches for channels in a team
func (p *Platform) SearchChannels(teamID, term string) ([]Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cTeamID := C.CString(teamID)
	defer C.free(unsafe.Pointer(cTeamID))

	cTerm := C.CString(term)
	defer C.free(unsafe.Pointer(cTerm))

	result := C.communicator_platform_search_channels(p.handle, cTeamID, cTerm)
	if result == nil {
		return nil, getLastError()
	}
	defer C.communicator_free_string(result)

	var channels []Channel
	if err := json.Unmarshal([]byte(C.GoString(result)), &channels); err != nil {
		return nil, err
	}

	return channels, nil
}

// AutocompleteChannels autocompletes channels for references
func (p *Platform) AutocompleteChannels(teamID, name string) ([]Channel, error) {
	if p.handle == nil {
		return nil, ErrInvalidHandle
	}

	cTeamID := C.CString(teamID)
	defer C.free(unsafe.Pointer(cTeamID))

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	result := C.communicator_platform_autocomplete_channels(p.handle, cTeamID, cName)
	if result == nil {
		return nil, getLastError()
	}
	defer C.communicator_free_string(result)

	var channels []Channel
	if err := json.Unmarshal([]byte(C.GoString(result)), &channels); err != nil {
		return nil, err
	}

	return channels, nil
}

// SearchFiles searches for files with advanced filtering
// Returns a JSON string with file search results
// Note: This function is not yet fully supported by the Platform trait
func (p *Platform) SearchFiles(request *FileSearchRequest) (string, error) {
	if p.handle == nil {
		return "", ErrInvalidHandle
	}

	requestJSON, err := json.Marshal(request)
	if err != nil {
		return "", err
	}

	cRequest := C.CString(string(requestJSON))
	defer C.free(unsafe.Pointer(cRequest))

	result := C.communicator_platform_search_files(p.handle, cRequest)
	if result == nil {
		return "", getLastError()
	}
	defer C.communicator_free_string(result)

	return C.GoString(result), nil
}

// SearchPostsAdvanced searches for posts with advanced filtering
// Returns a JSON string with post search results
// Note: This function is not yet fully supported by the Platform trait
func (p *Platform) SearchPostsAdvanced(options *PostSearchOptions) (string, error) {
	if p.handle == nil {
		return "", ErrInvalidHandle
	}

	requestJSON, err := json.Marshal(options)
	if err != nil {
		return "", err
	}

	cRequest := C.CString(string(requestJSON))
	defer C.free(unsafe.Pointer(cRequest))

	result := C.communicator_platform_search_posts_advanced(p.handle, cRequest)
	if result == nil {
		return "", getLastError()
	}
	defer C.communicator_free_string(result)

	return C.GoString(result), nil
}
