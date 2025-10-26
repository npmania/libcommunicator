package main

import (
	"encoding/json"
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
	flag.Parse()

	if *serverURL == "" || *teamID == "" {
		fmt.Println("Usage: mattermost_demo -server <url> -team <team_id> [-token <token> | -login <login> -password <password>]")
		fmt.Println("\nExamples:")
		fmt.Println("  Token auth:    mattermost_demo -server https://mattermost.example.com -team abc123 -token mytoken")
		fmt.Println("  Password auth: mattermost_demo -server https://mattermost.example.com -team abc123 -login user@example.com -password mypass")
		os.Exit(1)
	}

	if *token == "" && (*loginID == "" || *password == "") {
		fmt.Println("Error: Must provide either -token or both -login and -password")
		os.Exit(1)
	}

	fmt.Println("=== Mattermost Platform Demo (Go) ===")
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
	fmt.Printf("   Library version: %s (%d.%d.%d)\n", version.Full, version.Major, version.Minor, version.Patch)
	fmt.Println("   ✓ Initialized\n")

	// ========================================================================
	// 2. Create Mattermost platform instance
	// ========================================================================
	fmt.Println("2. Creating Mattermost platform...")
	platform, err := comm.NewMattermostPlatform(*serverURL)
	if err != nil {
		log.Fatalf("Failed to create platform: %v", err)
	}
	defer platform.Destroy()
	fmt.Println("   ✓ Platform created\n")

	// ========================================================================
	// 3. Connect and authenticate
	// ========================================================================
	fmt.Println("3. Connecting to Mattermost...")
	config := comm.NewPlatformConfig(*serverURL).WithTeamID(*teamID)

	if *token != "" {
		config.WithToken(*token)
	} else {
		config.WithPassword(*loginID, *password)
	}

	configJSON, _ := json.MarshalIndent(config, "   ", "  ")
	fmt.Printf("   Config: %s\n", string(configJSON))

	if err := platform.Connect(config); err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	fmt.Println("   ✓ Connected\n")

	// ========================================================================
	// 4. Check connection status
	// ========================================================================
	fmt.Println("4. Checking connection status...")
	isConnected := platform.IsConnected()
	fmt.Printf("   Connected: %v\n", isConnected)

	if connInfo, err := platform.GetConnectionInfo(); err == nil {
		connJSON, _ := json.MarshalIndent(connInfo, "   ", "  ")
		fmt.Printf("   Connection Info: %s\n", string(connJSON))
	}
	fmt.Println()

	// ========================================================================
	// 5. Get current user
	// ========================================================================
	fmt.Println("5. Getting current user info...")
	user, err := platform.GetCurrentUser()
	if err != nil {
		log.Printf("Failed to get current user: %v", err)
	} else {
		userJSON, _ := json.MarshalIndent(user, "   ", "  ")
		fmt.Printf("   Current User: %s\n", string(userJSON))
		fmt.Println("   ✓ Retrieved user info\n")
	}

	// ========================================================================
	// 6. Get channels
	// ========================================================================
	fmt.Println("6. Getting channels...")
	channels, err := platform.GetChannels()
	if err != nil {
		log.Printf("Failed to get channels: %v", err)
	} else {
		channelsJSON, _ := json.MarshalIndent(channels, "   ", "  ")
		fmt.Printf("   Channels (%d): %s\n", len(channels), string(channelsJSON))
		fmt.Println("   ✓ Retrieved channels\n")

		// If we have channels, get messages from the first one
		if len(channels) > 0 {
			fmt.Printf("7. Getting messages from channel '%s'...\n", channels[0].Name)
			messages, err := platform.GetMessages(channels[0].ID, 10)
			if err != nil {
				log.Printf("Failed to get messages: %v", err)
			} else {
				fmt.Printf("   Retrieved %d messages\n", len(messages))
				for i, msg := range messages {
					fmt.Printf("   [%d] %s: %s\n", i+1, msg.SenderID, msg.Text)
				}
				fmt.Println()
			}

			// Get channel members
			fmt.Printf("8. Getting members of channel '%s'...\n", channels[0].Name)
			members, err := platform.GetChannelMembers(channels[0].ID)
			if err != nil {
				log.Printf("Failed to get channel members: %v", err)
			} else {
				fmt.Printf("   Retrieved %d members\n", len(members))
				for i, member := range members {
					fmt.Printf("   [%d] %s (%s)\n", i+1, member.Username, member.Name)
				}
				fmt.Println()
			}
		}
	}

	// ========================================================================
	// 9. Get custom emojis
	// ========================================================================
	fmt.Println("9. Getting custom emojis...")
	emojis, err := platform.GetEmojis(0, 20) // Get first page, 20 emojis
	if err != nil {
		log.Printf("Failed to get emojis: %v", err)
	} else {
		fmt.Printf("   Retrieved %d custom emojis\n", len(emojis))
		if len(emojis) > 0 {
			fmt.Println("   First few emojis:")
			for i, emoji := range emojis {
				if i >= 5 {
					break
				}
				fmt.Printf("   [%d] :%s: (ID: %s, Creator: %s)\n", i+1, emoji.Name, emoji.ID, emoji.CreatorID)
			}
		} else {
			fmt.Println("   No custom emojis found on this server")
		}
		fmt.Println()
	}

	// ========================================================================
	// Optional: Send a message and add reactions (uncomment to test)
	// ========================================================================
	/*
	if len(channels) > 0 {
		fmt.Println("9. Sending a test message...")
		testMessage := "Hello from libcommunicator Go bindings!"
		msg, err := platform.SendMessage(channels[0].ID, testMessage)
		if err != nil {
			log.Printf("Failed to send message: %v", err)
		} else {
			msgJSON, _ := json.MarshalIndent(msg, "   ", "  ")
			fmt.Printf("   Sent Message: %s\n", string(msgJSON))
			fmt.Println("   ✓ Message sent\n")

			// Add reactions to the message
			fmt.Println("10. Adding reactions to the message...")
			if err := platform.AddReaction(msg.ID, "thumbsup"); err != nil {
				log.Printf("Failed to add thumbsup reaction: %v", err)
			} else {
				fmt.Println("   ✓ Added 👍 reaction")
			}

			if err := platform.AddReaction(msg.ID, "heart"); err != nil {
				log.Printf("Failed to add heart reaction: %v", err)
			} else {
				fmt.Println("   ✓ Added ❤️ reaction")
			}

			// Remove a reaction
			fmt.Println("\n11. Removing thumbsup reaction...")
			if err := platform.RemoveReaction(msg.ID, "thumbsup"); err != nil {
				log.Printf("Failed to remove thumbsup reaction: %v", err)
			} else {
				fmt.Println("   ✓ Removed 👍 reaction\n")
			}
		}
	}
	*/

	// ========================================================================
	// Optional: File operations (uncomment to test)
	// ========================================================================
	/*
	if len(channels) > 0 {
		fmt.Println("12. Testing file operations...")

		// Upload a file
		fmt.Println("   Uploading a test file...")
		testFilePath := "/path/to/your/test-file.txt" // Change this to an actual file path
		fileID, err := platform.UploadFile(channels[0].ID, testFilePath)
		if err != nil {
			log.Printf("Failed to upload file: %v", err)
		} else {
			fmt.Printf("   ✓ File uploaded with ID: %s\n", fileID)

			// Get file metadata
			fmt.Println("\n   Getting file metadata...")
			metadata, err := platform.GetFileMetadata(fileID)
			if err != nil {
				log.Printf("Failed to get file metadata: %v", err)
			} else {
				metadataJSON, _ := json.MarshalIndent(metadata, "      ", "  ")
				fmt.Printf("   File Metadata: %s\n", string(metadataJSON))
				fmt.Println("   ✓ Got file metadata")
			}

			// Download the file
			fmt.Println("\n   Downloading file...")
			fileData, err := platform.DownloadFile(fileID)
			if err != nil {
				log.Printf("Failed to download file: %v", err)
			} else {
				fmt.Printf("   ✓ Downloaded file (%d bytes)\n", len(fileData))

				// Optionally save the downloaded file
				// err := os.WriteFile("downloaded-file.txt", fileData, 0644)
				// if err != nil {
				//     log.Printf("Failed to save file: %v", err)
				// } else {
				//     fmt.Println("   ✓ Saved file to downloaded-file.txt")
				// }
			}

			// Get file thumbnail (for images/videos)
			fmt.Println("\n   Getting file thumbnail (if available)...")
			thumbnailData, err := platform.GetFileThumbnail(fileID)
			if err != nil {
				log.Printf("Thumbnail not available or failed: %v", err)
			} else {
				fmt.Printf("   ✓ Downloaded thumbnail (%d bytes)\n", len(thumbnailData))

				// Optionally save the thumbnail
				// err := os.WriteFile("thumbnail.jpg", thumbnailData, 0644)
				// if err != nil {
				//     log.Printf("Failed to save thumbnail: %v", err)
				// } else {
				//     fmt.Println("   ✓ Saved thumbnail to thumbnail.jpg")
				// }
			}

			fmt.Println()
		}
	}
	*/

	// ========================================================================
	// Disconnect
	// ========================================================================
	fmt.Println("10. Disconnecting...")
	if err := platform.Disconnect(); err != nil {
		log.Printf("Failed to disconnect: %v", err)
	} else {
		fmt.Println("   ✓ Disconnected\n")
	}

	fmt.Println("=== Demo Complete ===")
}
