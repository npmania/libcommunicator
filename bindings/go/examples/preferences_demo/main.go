package main

import (
	"flag"
	"fmt"
	"log"
	"os"

	comm "libcommunicator"
)

func main() {
	// Parse command-line arguments
	serverURL := flag.String("server", "", "Mattermost server URL")
	token := flag.String("token", "", "Authentication token")
	loginID := flag.String("login", "", "Login ID (email/username)")
	password := flag.String("password", "", "Password")
	teamID := flag.String("team", "", "Team ID")
	channelID := flag.String("channel", "", "Channel ID (for notification demo)")
	flag.Parse()

	if *serverURL == "" || *teamID == "" {
		fmt.Println("Usage: preferences_demo -server <url> -team <team_id> [-channel <channel_id>] [-token <token> | -login <login> -password <password>]")
		fmt.Println("\nExamples:")
		fmt.Println("  preferences_demo -server https://mattermost.example.com -team abc123 -token mytoken")
		fmt.Println("  preferences_demo -server https://mattermost.example.com -team abc123 -channel channel123 -login user@example.com -password mypassword")
		os.Exit(1)
	}

	if *token == "" && (*loginID == "" || *password == "") {
		fmt.Println("Error: Must provide either -token or both -login and -password")
		os.Exit(1)
	}

	fmt.Println("=== Preferences & Notifications Demo (Go) ===")
	fmt.Printf("Server: %s\n", *serverURL)
	fmt.Printf("Team ID: %s\n\n", *teamID)

	// ========================================================================
	// 1. Initialize the library
	// ========================================================================
	fmt.Println("1. Initializing library...")
	if err := comm.Init(); err != nil {
		log.Fatalf("Failed to initialize library: %v", err)
	}
	defer comm.Cleanup()

	version := comm.GetVersion()
	fmt.Printf("   Library version: %s\n", version.Full)
	fmt.Println("   ✓ Initialized\n")

	// ========================================================================
	// 2. Create platform and connect
	// ========================================================================
	fmt.Println("2. Creating Mattermost platform...")
	platform, err := comm.NewMattermostPlatform(*serverURL)
	if err != nil {
		log.Fatalf("Failed to create platform: %v", err)
	}
	defer platform.Destroy()
	fmt.Println("   ✓ Platform created\n")

	fmt.Println("3. Connecting to Mattermost...")
	config := comm.NewPlatformConfig(*serverURL)
	config.TeamID = *teamID

	if *token != "" {
		config = config.WithToken(*token)
	} else {
		config = config.WithPassword(*loginID, *password)
	}

	err = platform.Connect(config)
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}

	connInfo, err := platform.GetConnectionInfo()
	if err != nil {
		log.Fatalf("Failed to get connection info: %v", err)
	}

	userID := connInfo.UserID
	fmt.Printf("   ✓ Connected as user: %s\n", connInfo.UserID)
	fmt.Printf("   Team: %s\n\n", connInfo.TeamID)

	// ========================================================================
	// 4. Get user preferences
	// ========================================================================
	fmt.Printf("4. Fetching preferences for user %s...\n", userID)
	prefs, err := platform.GetUserPreferences(userID)
	if err != nil {
		log.Printf("   Failed to get preferences: %v\n", err)
	} else {
		fmt.Printf("   ✓ Retrieved %d preferences\n", len(prefs))
		if len(prefs) > 0 {
			fmt.Println("   Sample preferences:")
			count := 0
			for _, pref := range prefs {
				if count >= 5 {
					fmt.Printf("   ... and %d more\n", len(prefs)-5)
					break
				}
				fmt.Printf("     - %s/%s = %s\n", pref.Category, pref.Name, pref.Value)
				count++
			}
		}
		fmt.Println()
	}

	// ========================================================================
	// 5. Set a custom preference
	// ========================================================================
	fmt.Println("5. Setting a custom preference...")
	customPrefs := []comm.UserPreference{
		{
			UserID:   userID,
			Category: "custom_app_settings",
			Name:     "demo_setting",
			Value:    "enabled",
		},
		{
			UserID:   userID,
			Category: "custom_app_settings",
			Name:     "demo_timestamp",
			Value:    fmt.Sprintf("%d", 1234567890),
		},
	}

	err = platform.SetUserPreferences(userID, customPrefs)
	if err != nil {
		log.Printf("   Failed to set preferences: %v\n", err)
	} else {
		fmt.Println("   ✓ Set custom preferences")
		fmt.Println("     - custom_app_settings/demo_setting = enabled")
		fmt.Println("     - custom_app_settings/demo_timestamp = 1234567890")
		fmt.Println()
	}

	// ========================================================================
	// 6. Channel notification settings (if channel provided)
	// ========================================================================
	if *channelID != "" {
		fmt.Printf("\n6. Channel Notification Settings for %s\n", *channelID)
		fmt.Println("   ----------------------------------------")

		// Mute the channel
		fmt.Println("   a) Muting channel...")
		err = platform.MuteChannel(*channelID)
		if err != nil {
			log.Printf("      Failed to mute channel: %v\n", err)
		} else {
			fmt.Println("      ✓ Channel muted")
		}

		// Wait for user input
		fmt.Println("\n   Channel is now muted. Press Enter to unmute...")
		fmt.Scanln()

		// Unmute the channel
		fmt.Println("   b) Unmuting channel...")
		err = platform.UnmuteChannel(*channelID)
		if err != nil {
			log.Printf("      Failed to unmute channel: %v\n", err)
		} else {
			fmt.Println("      ✓ Channel unmuted")
		}

		// Set custom notification properties
		fmt.Println("\n   c) Setting custom notification properties...")
		customProps := comm.NewChannelNotifyProps().
			WithDesktop(comm.NotificationLevelMention).
			WithPush(comm.NotificationLevelAll).
			WithEmail(true).
			WithMarkUnread(comm.NotificationLevelMention)

		err = platform.UpdateChannelNotifyProps(*channelID, customProps)
		if err != nil {
			log.Printf("      Failed to update channel notification properties: %v\n", err)
		} else {
			fmt.Println("      ✓ Updated channel notification properties:")
			fmt.Println("        - Desktop: mention only")
			fmt.Println("        - Push: all messages")
			fmt.Println("        - Email: enabled")
			fmt.Println("        - Mark unread: mention only")
		}

		fmt.Println()
	} else {
		fmt.Println("\n6. Skipping channel notification demo (no -channel provided)")
		fmt.Println("   Run with -channel <channel_id> to test channel notifications\n")
	}

	// ========================================================================
	// 7. Disconnect
	// ========================================================================
	fmt.Println("7. Disconnecting...")
	if err := platform.Disconnect(); err != nil {
		log.Printf("   Warning: Disconnect failed: %v\n", err)
	} else {
		fmt.Println("   ✓ Disconnected\n")
	}

	fmt.Println("=== Demo Complete ===")
}
