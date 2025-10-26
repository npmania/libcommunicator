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
	postID := flag.String("post", "", "Post ID (root of thread to demonstrate)")
	flag.Parse()

	if *serverURL == "" || *teamID == "" {
		fmt.Println("Usage: threads_demo -server <url> -team <team_id> -post <post_id> [-token <token> | -login <login> -password <password>]")
		fmt.Println("\nExamples:")
		fmt.Println("  threads_demo -server https://mattermost.example.com -team abc123 -post post123 -token mytoken")
		os.Exit(1)
	}

	if *token == "" && (*loginID == "" || *password == "") {
		fmt.Println("Error: Must provide either -token or both -login and -password")
		os.Exit(1)
	}

	if *postID == "" {
		fmt.Println("Error: Must provide -post <post_id> to demonstrate thread operations")
		os.Exit(1)
	}

	fmt.Println("=== Thread Operations Demo (Go) ===")
	fmt.Printf("Server: %s\n", *serverURL)
	fmt.Printf("Team ID: %s\n", *teamID)
	fmt.Printf("Post ID: %s\n\n", *postID)

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

	fmt.Printf("   ✓ Connected as user: %s\n", connInfo.UserID)
	fmt.Printf("   Team: %s\n\n", connInfo.TeamID)

	// ========================================================================
	// 4. Get thread (root post + all replies)
	// ========================================================================
	fmt.Printf("4. Fetching thread for post %s...\n", *postID)
	messages, err := platform.GetThread(*postID)
	if err != nil {
		log.Printf("   Failed to get thread: %v", err)
		fmt.Println("   (This is normal if the post doesn't exist or isn't in a thread)\n")
	} else {
		fmt.Printf("   ✓ Retrieved %d messages in thread\n", len(messages))
		if len(messages) > 0 {
			fmt.Println("   Thread messages:")
			for i, msg := range messages {
				// First message is typically the root
				msgType := "Root"
				if i > 0 {
					msgType = "Reply"
				}
				fmt.Printf("     [%d] %s: %s (ID: %s)\n", i+1, msgType, truncate(msg.Text, 60), msg.ID)
			}
		}
		fmt.Println()
	}

	// ========================================================================
	// 5. Follow thread
	// ========================================================================
	fmt.Printf("5. Following thread %s...\n", *postID)
	err = platform.FollowThread(*postID)
	if err != nil {
		log.Printf("   Failed to follow thread: %v\n", err)
		fmt.Println("   (This might fail if you're already following or if threads aren't supported)\n")
	} else {
		fmt.Println("   ✓ Now following thread\n")
	}

	// ========================================================================
	// 6. Mark thread as read
	// ========================================================================
	fmt.Printf("6. Marking thread %s as read...\n", *postID)
	err = platform.MarkThreadRead(*postID)
	if err != nil {
		log.Printf("   Failed to mark thread as read: %v\n", err)
	} else {
		fmt.Println("   ✓ Thread marked as read\n")
	}

	// ========================================================================
	// 7. Mark thread as unread (from first reply if available)
	// ========================================================================
	if len(messages) > 1 {
		firstReplyID := messages[1].ID
		fmt.Printf("7. Marking thread as unread from post %s...\n", firstReplyID)
		err = platform.MarkThreadUnread(*postID, firstReplyID)
		if err != nil {
			log.Printf("   Failed to mark thread as unread: %v\n", err)
		} else {
			fmt.Println("   ✓ Thread marked as unread\n")
		}
	} else {
		fmt.Println("7. Skipping mark as unread (no replies in thread)\n")
	}

	// ========================================================================
	// 8. Unfollow thread
	// ========================================================================
	fmt.Printf("8. Unfollowing thread %s...\n", *postID)
	err = platform.UnfollowThread(*postID)
	if err != nil {
		log.Printf("   Failed to unfollow thread: %v\n", err)
	} else {
		fmt.Println("   ✓ Unfollowed thread\n")
	}

	// ========================================================================
	// 9. Disconnect
	// ========================================================================
	fmt.Println("9. Disconnecting...")
	if err := platform.Disconnect(); err != nil {
		log.Printf("Failed to disconnect: %v", err)
	} else {
		fmt.Println("   ✓ Disconnected\n")
	}

	fmt.Println("=== Demo Complete ===")
}

// truncate truncates a string to maxLen characters
func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen-3] + "..."
}
