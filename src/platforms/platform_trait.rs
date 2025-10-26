//! Platform trait defining the interface all platform adapters must implement

use crate::error::Result;
use crate::types::{Channel, ConnectionInfo, Message, PlatformCapabilities, Team, User};
use crate::types::user::UserStatus;
use async_trait::async_trait;
use std::collections::HashMap;

/// Configuration for connecting to a platform
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    /// Server URL or endpoint
    pub server: String,
    /// Authentication credentials (e.g., token, username/password)
    pub credentials: HashMap<String, String>,
    /// Optional team/workspace/guild identifier
    /// Only applicable for platforms that support organizational hierarchies
    /// (check PlatformCapabilities.has_workspaces)
    pub team_id: Option<String>,
    /// Additional platform-specific configuration
    pub extra: HashMap<String, String>,
}

impl PlatformConfig {
    /// Create a new platform configuration
    pub fn new(server: impl Into<String>) -> Self {
        PlatformConfig {
            server: server.into(),
            credentials: HashMap::new(),
            team_id: None,
            extra: HashMap::new(),
        }
    }

    /// Add a credential
    pub fn with_credential(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.credentials.insert(key.into(), value.into());
        self
    }

    /// Set team/workspace ID
    pub fn with_team(mut self, team_id: impl Into<String>) -> Self {
        self.team_id = Some(team_id.into());
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Event types that can be received from a platform
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// A new message was posted
    MessagePosted(Message),
    /// A message was updated/edited
    MessageUpdated(Message),
    /// A message was deleted
    MessageDeleted { message_id: String, channel_id: String },
    /// A user's status changed
    UserStatusChanged { user_id: String, status: crate::types::user::UserStatus },
    /// A user started typing
    UserTyping { user_id: String, channel_id: String },
    /// A channel was created
    ChannelCreated(Channel),
    /// A channel was updated
    ChannelUpdated(Channel),
    /// A channel was deleted
    ChannelDeleted { channel_id: String },
    /// User joined a channel
    UserJoinedChannel { user_id: String, channel_id: String },
    /// User left a channel
    UserLeftChannel { user_id: String, channel_id: String },
    /// Connection state changed
    ConnectionStateChanged(crate::types::connection::ConnectionState),
    /// A reaction was added to a message
    ReactionAdded {
        message_id: String,
        user_id: String,
        emoji_name: String,
        channel_id: String,
    },
    /// A reaction was removed from a message
    ReactionRemoved {
        message_id: String,
        user_id: String,
        emoji_name: String,
        channel_id: String,
    },
    /// A direct message channel was created
    DirectChannelAdded { channel_id: String },
    /// A group message channel was created
    GroupChannelAdded { channel_id: String },
    /// A user preference was changed
    PreferenceChanged {
        category: String,
        name: String,
        value: String,
    },
    /// An ephemeral message was received (temporary, typically bot responses)
    EphemeralMessage {
        message: String,
        channel_id: String,
    },
    /// A new user joined the team/server
    UserAdded { user_id: String },
    /// A user's profile was updated
    UserUpdated { user_id: String },
    /// A user's role was updated
    UserRoleUpdated { user_id: String },
    /// A user viewed a channel
    ChannelViewed {
        user_id: String,
        channel_id: String,
    },
    /// A thread was updated (metadata changed)
    ThreadUpdated {
        thread_id: String,
        channel_id: String,
    },
    /// Thread read status changed
    ThreadReadChanged {
        thread_id: String,
        user_id: String,
        channel_id: String,
    },
    /// Thread follow status changed
    ThreadFollowChanged {
        thread_id: String,
        user_id: String,
        channel_id: String,
        following: bool,
    },
    /// A post was marked as unread
    PostUnread {
        post_id: String,
        channel_id: String,
        user_id: String,
    },
    /// A custom emoji was added
    EmojiAdded {
        emoji_id: String,
        emoji_name: String,
    },
    /// User was added to a team
    AddedToTeam {
        team_id: String,
        user_id: String,
    },
    /// User left a team
    LeftTeam {
        team_id: String,
        user_id: String,
    },
    /// Server configuration changed
    ConfigChanged,
    /// Server license changed
    LicenseChanged,
    /// Channel was converted (e.g., public to private)
    ChannelConverted { channel_id: String },
    /// Channel member was updated
    ChannelMemberUpdated {
        channel_id: String,
        user_id: String,
    },
    /// Team was deleted
    TeamDeleted { team_id: String },
    /// Team was updated
    TeamUpdated { team_id: String },
    /// Member role was updated in a channel
    MemberRoleUpdated {
        channel_id: String,
        user_id: String,
    },
    /// Plugin was disabled
    PluginDisabled { plugin_id: String },
    /// Plugin was enabled
    PluginEnabled { plugin_id: String },
    /// Plugin statuses changed
    PluginStatusesChanged,
    /// User preferences were deleted
    PreferencesDeleted {
        category: String,
        name: String,
    },
    /// WebSocket action response
    Response {
        status: String,
        seq_reply: i64,
        error: Option<String>,
    },
    /// Dialog was opened
    DialogOpened { dialog_id: String },
    /// Role was updated
    RoleUpdated { role_id: String },
}

/// Trait that all platform adapters must implement
///
/// This defines the common interface for interacting with different chat platforms
/// (Mattermost, Slack, Discord, etc.)
///
/// Not all methods are supported by all platforms. Use `capabilities()` to check
/// what features are available before calling optional methods.
#[async_trait]
pub trait Platform: Send + Sync {
    /// Get the capabilities of this platform
    ///
    /// Returns information about what features this platform supports.
    /// Consumers should check capabilities before calling optional methods.
    fn capabilities(&self) -> &PlatformCapabilities;

    /// Connect to the platform and authenticate
    ///
    /// # Arguments
    /// * `config` - Configuration including server URL and credentials
    ///
    /// # Returns
    /// Connection information on success
    async fn connect(&mut self, config: PlatformConfig) -> Result<ConnectionInfo>;

    /// Disconnect from the platform
    async fn disconnect(&mut self) -> Result<()>;

    /// Get current connection information
    ///
    /// Returns None if not connected
    fn connection_info(&self) -> Option<&ConnectionInfo>;

    /// Check if currently connected
    fn is_connected(&self) -> bool {
        self.connection_info()
            .map(|info| info.is_connected())
            .unwrap_or(false)
    }

    /// Send a message to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel to send the message to
    /// * `text` - The message text
    ///
    /// # Returns
    /// The created message
    async fn send_message(&self, channel_id: &str, text: &str) -> Result<Message>;

    /// Get a list of channels the user has access to
    async fn get_channels(&self) -> Result<Vec<Channel>>;

    /// Get details about a specific channel
    async fn get_channel(&self, channel_id: &str) -> Result<Channel>;

    /// Get recent messages from a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `limit` - Maximum number of messages to retrieve
    ///
    /// # Returns
    /// List of messages, most recent first
    async fn get_messages(&self, channel_id: &str, limit: usize) -> Result<Vec<Message>>;

    /// Get a list of users in a channel
    async fn get_channel_members(&self, channel_id: &str) -> Result<Vec<User>>;

    /// Get details about a specific user
    async fn get_user(&self, user_id: &str) -> Result<User>;

    /// Get details about the currently authenticated user
    async fn get_current_user(&self) -> Result<User>;

    /// Create a direct message channel with another user
    ///
    /// # Arguments
    /// * `user_id` - The user to create a DM channel with
    ///
    /// # Returns
    /// The created or existing DM channel
    async fn create_direct_channel(&self, user_id: &str) -> Result<Channel>;

    /// Get all teams/workspaces the user belongs to
    ///
    /// # Returns
    /// List of teams
    ///
    /// # Errors
    /// Returns `ErrorCode::Unsupported` if the platform doesn't support teams/workspaces.
    /// Check `capabilities().has_workspaces` before calling.
    async fn get_teams(&self) -> Result<Vec<Team>>;

    /// Get details about a specific team
    ///
    /// # Arguments
    /// * `team_id` - The team ID
    ///
    /// # Returns
    /// The team details
    ///
    /// # Errors
    /// Returns `ErrorCode::Unsupported` if the platform doesn't support teams/workspaces.
    /// Check `capabilities().has_workspaces` before calling.
    async fn get_team(&self, team_id: &str) -> Result<Team>;

    /// Set the current user's status
    ///
    /// # Arguments
    /// * `status` - The status to set (online, away, dnd, offline)
    /// * `custom_message` - Optional custom status message (e.g., "In a meeting", "Working remotely")
    ///
    /// # Returns
    /// Result indicating success
    ///
    /// # Notes
    /// Not all platforms support custom status messages. If provided but not supported,
    /// the custom message will be silently ignored. Check `capabilities().supports_custom_status`.
    async fn set_status(&self, status: UserStatus, custom_message: Option<&str>) -> Result<()>;

    /// Get a user's status
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    ///
    /// # Returns
    /// The user's status
    async fn get_user_status(&self, user_id: &str) -> Result<UserStatus>;

    /// Subscribe to real-time events (WebSocket, webhook, etc.)
    ///
    /// This method should establish a connection for receiving real-time events.
    /// Events should be delivered through the event callback.
    async fn subscribe_events(&mut self) -> Result<()>;

    /// Unsubscribe from real-time events
    async fn unsubscribe_events(&mut self) -> Result<()>;

    /// Poll for the next event (if available)
    ///
    /// This is a non-blocking check for new events.
    /// Returns None if no events are available.
    async fn poll_event(&mut self) -> Result<Option<PlatformEvent>>;

    // ========================================================================
    // Extended Platform Methods
    // ========================================================================

    /// Send a reply to a message (threaded conversation)
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `text` - The message text
    /// * `root_id` - The ID of the message to reply to
    ///
    /// # Returns
    /// The created reply message
    ///
    /// # Notes
    /// Not all platforms support threading. Check `capabilities().has_threads` first.
    async fn send_reply(&self, channel_id: &str, text: &str, root_id: &str) -> Result<Message> {
        let _ = (channel_id, text, root_id);
        Err(crate::error::Error::unsupported("Threaded messages not supported by this platform"))
    }

    /// Update/edit a message
    ///
    /// # Arguments
    /// * `message_id` - The message ID to update
    /// * `new_text` - The new message text
    ///
    /// # Returns
    /// The updated message
    ///
    /// # Notes
    /// Not all platforms support message editing. Check `capabilities().supports_message_editing` first.
    async fn update_message(&self, message_id: &str, new_text: &str) -> Result<Message> {
        let _ = (message_id, new_text);
        Err(crate::error::Error::unsupported("Message editing not supported by this platform"))
    }

    /// Delete a message
    ///
    /// # Arguments
    /// * `message_id` - The message ID to delete
    ///
    /// # Notes
    /// Not all platforms support message deletion. Check `capabilities().supports_message_deletion` first.
    async fn delete_message(&self, message_id: &str) -> Result<()> {
        let _ = message_id;
        Err(crate::error::Error::unsupported("Message deletion not supported by this platform"))
    }

    /// Get a specific message by ID
    ///
    /// # Arguments
    /// * `message_id` - The message ID
    ///
    /// # Returns
    /// The message
    async fn get_message(&self, message_id: &str) -> Result<Message> {
        let _ = message_id;
        Err(crate::error::Error::unsupported("Get message by ID not supported by this platform"))
    }

    /// Search for messages
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    /// List of matching messages
    ///
    /// # Notes
    /// Not all platforms support search. Check `capabilities().supports_search` first.
    async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<Message>> {
        let _ = (query, limit);
        Err(crate::error::Error::unsupported("Message search not supported by this platform"))
    }

    /// Get messages before a specific message (pagination)
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `before_id` - Get messages before this message ID
    /// * `limit` - Maximum number of messages to retrieve
    ///
    /// # Returns
    /// List of messages
    async fn get_messages_before(&self, channel_id: &str, before_id: &str, limit: usize) -> Result<Vec<Message>> {
        let _ = (channel_id, before_id, limit);
        Err(crate::error::Error::unsupported("Message pagination not supported by this platform"))
    }

    /// Get messages after a specific message (pagination)
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `after_id` - Get messages after this message ID
    /// * `limit` - Maximum number of messages to retrieve
    ///
    /// # Returns
    /// List of messages
    async fn get_messages_after(&self, channel_id: &str, after_id: &str, limit: usize) -> Result<Vec<Message>> {
        let _ = (channel_id, after_id, limit);
        Err(crate::error::Error::unsupported("Message pagination not supported by this platform"))
    }

    /// Add a reaction to a message
    ///
    /// # Arguments
    /// * `message_id` - The message ID to react to
    /// * `emoji` - The emoji name (e.g., "thumbsup", "smile", "heart")
    ///
    /// # Notes
    /// Not all platforms support reactions. Check `capabilities().supports_reactions` first.
    async fn add_reaction(&self, message_id: &str, emoji: &str) -> Result<()> {
        let _ = (message_id, emoji);
        Err(crate::error::Error::unsupported("Reactions not supported by this platform"))
    }

    /// Remove a reaction from a message
    ///
    /// # Arguments
    /// * `message_id` - The message ID
    /// * `emoji` - The emoji name to remove
    ///
    /// # Notes
    /// Not all platforms support reactions. Check `capabilities().supports_reactions` first.
    async fn remove_reaction(&self, message_id: &str, emoji: &str) -> Result<()> {
        let _ = (message_id, emoji);
        Err(crate::error::Error::unsupported("Reactions not supported by this platform"))
    }

    /// Get a list of custom emojis available on the platform
    ///
    /// # Arguments
    /// * `page` - The page number to retrieve (0-indexed)
    /// * `per_page` - Number of emojis per page
    ///
    /// # Returns
    /// A list of custom emojis
    ///
    /// # Notes
    /// - This returns custom emojis only, not standard Unicode emojis
    /// - Not all platforms may support custom emojis
    /// - Default implementation returns an unsupported error
    async fn get_emojis(&self, page: u32, per_page: u32) -> Result<Vec<crate::types::Emoji>> {
        let _ = (page, per_page);
        Err(crate::error::Error::unsupported("Custom emojis not supported by this platform"))
    }

    /// Get a channel by name
    ///
    /// # Arguments
    /// * `team_id` - The team ID (required for platforms with workspaces)
    /// * `channel_name` - The channel name
    ///
    /// # Returns
    /// The channel
    async fn get_channel_by_name(&self, team_id: &str, channel_name: &str) -> Result<Channel> {
        let _ = (team_id, channel_name);
        Err(crate::error::Error::unsupported("Get channel by name not supported by this platform"))
    }

    /// Create a group direct message channel
    ///
    /// # Arguments
    /// * `user_ids` - List of user IDs to include in the group
    ///
    /// # Returns
    /// The created group channel
    ///
    /// # Notes
    /// Not all platforms support group messages. Check `capabilities().supports_group_messages` first.
    async fn create_group_channel(&self, user_ids: Vec<String>) -> Result<Channel> {
        let _ = user_ids;
        Err(crate::error::Error::unsupported("Group channels not supported by this platform"))
    }

    /// Add a user to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `user_id` - The user ID to add
    async fn add_channel_member(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let _ = (channel_id, user_id);
        Err(crate::error::Error::unsupported("Channel member management not supported by this platform"))
    }

    /// Remove a user from a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `user_id` - The user ID to remove
    async fn remove_channel_member(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let _ = (channel_id, user_id);
        Err(crate::error::Error::unsupported("Channel member management not supported by this platform"))
    }

    /// Get a user by username
    ///
    /// # Arguments
    /// * `username` - The username
    ///
    /// # Returns
    /// The user
    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let _ = username;
        Err(crate::error::Error::unsupported("User lookup by username not supported by this platform"))
    }

    /// Get a user by email
    ///
    /// # Arguments
    /// * `email` - The email address
    ///
    /// # Returns
    /// The user
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let _ = email;
        Err(crate::error::Error::unsupported("User lookup by email not supported by this platform"))
    }

    /// Get multiple users by their IDs (batch operation)
    ///
    /// # Arguments
    /// * `user_ids` - List of user IDs
    ///
    /// # Returns
    /// List of users
    async fn get_users_by_ids(&self, user_ids: Vec<String>) -> Result<Vec<User>> {
        let _ = user_ids;
        Err(crate::error::Error::unsupported("Batch user lookup not supported by this platform"))
    }

    /// Set a custom status message
    ///
    /// # Arguments
    /// * `emoji` - Optional emoji for the status
    /// * `text` - Status text message
    /// * `expires_at` - Optional expiration timestamp (Unix timestamp in seconds)
    ///
    /// # Notes
    /// Not all platforms support custom status. Check `capabilities().supports_custom_status` first.
    async fn set_custom_status(&self, emoji: Option<&str>, text: &str, expires_at: Option<i64>) -> Result<()> {
        let _ = (emoji, text, expires_at);
        Err(crate::error::Error::unsupported("Custom status not supported by this platform"))
    }

    /// Remove/clear the current user's custom status
    ///
    /// # Notes
    /// Not all platforms support custom status. Check `capabilities().supports_custom_status` first.
    async fn remove_custom_status(&self) -> Result<()> {
        Err(crate::error::Error::unsupported("Custom status not supported by this platform"))
    }

    /// Get status for multiple users (batch operation)
    ///
    /// # Arguments
    /// * `user_ids` - List of user IDs
    ///
    /// # Returns
    /// Map of user_id to status
    async fn get_users_status(&self, user_ids: Vec<String>) -> Result<std::collections::HashMap<String, UserStatus>> {
        let _ = user_ids;
        Err(crate::error::Error::unsupported("Batch user status not supported by this platform"))
    }

    /// Request statuses for all users via WebSocket (async operation)
    ///
    /// This method sends a WebSocket request to get statuses for all users.
    /// Unlike `get_users_status`, this is non-blocking and returns immediately with a sequence number.
    /// The actual status data will arrive later as a `Response` event with matching `seq_reply`.
    ///
    /// # Returns
    /// The sequence number of the request. Match this with `seq_reply` in Response events.
    ///
    /// # Notes
    /// - Requires an active WebSocket connection (call `subscribe_events` first)
    /// - Not all platforms support WebSocket-based status queries
    /// - The response will be a `PlatformEvent::Response` with status data
    async fn request_all_statuses(&self) -> Result<i64> {
        Err(crate::error::Error::unsupported("WebSocket status queries not supported by this platform"))
    }

    /// Request statuses for specific users via WebSocket (async operation)
    ///
    /// This method sends a WebSocket request to get statuses for specific users.
    /// Unlike `get_users_status`, this is non-blocking and returns immediately with a sequence number.
    /// The actual status data will arrive later as a `Response` event with matching `seq_reply`.
    ///
    /// # Arguments
    /// * `user_ids` - List of user IDs to get statuses for
    ///
    /// # Returns
    /// The sequence number of the request. Match this with `seq_reply` in Response events.
    ///
    /// # Notes
    /// - Requires an active WebSocket connection (call `subscribe_events` first)
    /// - Not all platforms support WebSocket-based status queries
    /// - The response will be a `PlatformEvent::Response` with status data
    async fn request_users_statuses(&self, user_ids: Vec<String>) -> Result<i64> {
        let _ = user_ids;
        Err(crate::error::Error::unsupported("WebSocket status queries not supported by this platform"))
    }

    /// Send a typing indicator to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel to send typing indicator to
    /// * `parent_id` - Optional parent post ID for thread typing indicators
    ///
    /// # Notes
    /// Not all platforms support typing indicators. This is a best-effort operation
    /// that may fail silently on platforms without typing indicator support.
    /// Typing indicators are typically short-lived (cleared after a few seconds of no activity).
    async fn send_typing_indicator(&self, channel_id: &str, parent_id: Option<&str>) -> Result<()> {
        let _ = (channel_id, parent_id);
        Err(crate::error::Error::unsupported("Typing indicators not supported by this platform"))
    }

    /// Get a team by name
    ///
    /// # Arguments
    /// * `team_name` - The team name
    ///
    /// # Returns
    /// The team
    ///
    /// # Notes
    /// Only applicable for platforms with workspaces. Check `capabilities().has_workspaces` first.
    async fn get_team_by_name(&self, team_name: &str) -> Result<Team> {
        let _ = team_name;
        Err(crate::error::Error::unsupported("Team lookup by name not supported by this platform"))
    }

    /// Set the active team/workspace ID
    ///
    /// # Arguments
    /// * `team_id` - The team ID to set as active (or None to unset)
    ///
    /// # Notes
    /// Only applicable for platforms with workspaces. Check `capabilities().has_workspaces` first.
    /// This affects operations that are team-scoped, such as getting channels or searching messages.
    async fn set_team_id(&self, team_id: Option<String>) -> Result<()> {
        let _ = team_id;
        Err(crate::error::Error::unsupported("Setting team ID not supported by this platform"))
    }

    // ========================================================================
    // File Operations
    // ========================================================================

    /// Upload a file to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID where the file will be uploaded
    /// * `file_path` - Path to the file to upload
    ///
    /// # Returns
    /// The file ID of the uploaded file, which can be used to attach the file to a message
    ///
    /// # Notes
    /// Not all platforms support file uploads. Check `capabilities().supports_file_attachments` first.
    /// The file is uploaded to the server but not yet attached to a message. Use the returned file ID
    /// when sending a message to attach the file.
    async fn upload_file(&self, channel_id: &str, file_path: &std::path::Path) -> Result<String> {
        let _ = (channel_id, file_path);
        Err(crate::error::Error::unsupported("File uploads not supported by this platform"))
    }

    /// Download a file by its ID
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file to download
    ///
    /// # Returns
    /// The file contents as bytes
    ///
    /// # Notes
    /// Not all platforms support file downloads. Check `capabilities().supports_file_attachments` first.
    async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let _ = file_id;
        Err(crate::error::Error::unsupported("File downloads not supported by this platform"))
    }

    /// Get metadata for a file without downloading it
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file
    ///
    /// # Returns
    /// Attachment metadata including filename, size, MIME type, etc.
    ///
    /// # Notes
    /// This allows checking file information without downloading the full file content.
    /// Not all platforms support this operation. Check `capabilities().supports_file_attachments` first.
    async fn get_file_metadata(&self, file_id: &str) -> Result<crate::types::Attachment> {
        let _ = file_id;
        Err(crate::error::Error::unsupported("File metadata not supported by this platform"))
    }

    /// Download a thumbnail for a file
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file
    ///
    /// # Returns
    /// The thumbnail image as bytes
    ///
    /// # Notes
    /// Thumbnails are typically only available for image and video files.
    /// The operation will return an error if the file doesn't have a thumbnail.
    /// Not all platforms support thumbnails.
    async fn get_file_thumbnail(&self, file_id: &str) -> Result<Vec<u8>> {
        let _ = file_id;
        Err(crate::error::Error::unsupported("File thumbnails not supported by this platform"))
    }

    // ========================================================================
    // Thread Operations
    // ========================================================================

    /// Get a thread (root post and all replies)
    ///
    /// Fetches a complete thread including the root post and all replies.
    ///
    /// # Arguments
    /// * `post_id` - The ID of any post in the thread (typically the root post)
    ///
    /// # Returns
    /// Vector of messages in the thread, typically ordered chronologically
    ///
    /// # Notes
    /// Not all platforms support threading. Check `capabilities().has_threads` first.
    /// The returned messages should include the root post plus all replies.
    async fn get_thread(&self, post_id: &str) -> Result<Vec<Message>> {
        let _ = post_id;
        Err(crate::error::Error::unsupported("Thread operations not supported by this platform"))
    }

    /// Start following a thread
    ///
    /// Makes the authenticated user follow a thread to receive notifications for new replies.
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID (typically the root post ID)
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Notes
    /// Not all platforms support thread following. This is a best-effort operation.
    /// Some platforms may automatically follow threads when you participate in them.
    async fn follow_thread(&self, thread_id: &str) -> Result<()> {
        let _ = thread_id;
        Err(crate::error::Error::unsupported("Thread following not supported by this platform"))
    }

    /// Stop following a thread
    ///
    /// Makes the authenticated user unfollow a thread to stop receiving notifications.
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID (typically the root post ID)
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Notes
    /// Not all platforms support thread following.
    async fn unfollow_thread(&self, thread_id: &str) -> Result<()> {
        let _ = thread_id;
        Err(crate::error::Error::unsupported("Thread following not supported by this platform"))
    }

    /// Mark a thread as read
    ///
    /// Marks all messages in a thread as read up to the current time.
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID (typically the root post ID)
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Notes
    /// Not all platforms support read receipts or thread read status.
    /// This method marks the thread as read up to the current timestamp.
    async fn mark_thread_read(&self, thread_id: &str) -> Result<()> {
        let _ = thread_id;
        Err(crate::error::Error::unsupported("Thread read status not supported by this platform"))
    }

    /// Mark a thread as unread
    ///
    /// Marks a thread as unread, typically from a specific post onwards.
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID (typically the root post ID)
    /// * `post_id` - The post ID to mark as unread from
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Notes
    /// Not all platforms support marking threads as unread.
    /// The behavior may vary - some platforms mark from the specified post, others mark the entire thread.
    async fn mark_thread_unread(&self, thread_id: &str, post_id: &str) -> Result<()> {
        let _ = (thread_id, post_id);
        Err(crate::error::Error::unsupported("Thread read status not supported by this platform"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_config_builder() {
        let config = PlatformConfig::new("https://chat.example.com")
            .with_credential("token", "secret-token")
            .with_team("team-123")
            .with_extra("timeout", "30");

        assert_eq!(config.server, "https://chat.example.com");
        assert_eq!(config.credentials.get("token"), Some(&"secret-token".to_string()));
        assert_eq!(config.team_id, Some("team-123".to_string()));
        assert_eq!(config.extra.get("timeout"), Some(&"30".to_string()));
    }
}
