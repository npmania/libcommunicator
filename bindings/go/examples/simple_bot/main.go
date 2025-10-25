package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"strings"
	"syscall"

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
		fmt.Println("Usage: simple_bot -server <url> -team <team_id> [-token <token> | -login <login> -password <password>]")
		os.Exit(1)
	}

	if *token == "" && (*loginID == "" || *password == "") {
		fmt.Println("Error: Must provide either -token or both -login and -password")
		os.Exit(1)
	}

	fmt.Println("=== Simple Bot Demo ===")
	fmt.Printf("Server: %s\n", *serverURL)
	fmt.Printf("Team ID: %s\n\n", *teamID)

	// Initialize the library
	if err := comm.Init(); err != nil {
		log.Fatalf("Failed to initialize: %v", err)
	}
	defer comm.Cleanup()

	version := comm.GetVersion()
	fmt.Printf("Library version: %s\n", version.Full)

	// Create platform
	platform, err := comm.NewMattermostPlatform(*serverURL)
	if err != nil {
		log.Fatalf("Failed to create platform: %v", err)
	}
	defer platform.Destroy()

	// Connect
	config := comm.NewPlatformConfig(*serverURL).WithTeamID(*teamID)
	if *token != "" {
		config.WithToken(*token)
	} else {
		config.WithPassword(*loginID, *password)
	}

	if err := platform.Connect(config); err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer platform.Disconnect()

	// Get current user
	currentUser, err := platform.GetCurrentUser()
	if err != nil {
		log.Fatalf("Failed to get current user: %v", err)
	}
	fmt.Printf("Bot user: @%s (%s)\n", currentUser.Username, currentUser.Name)
	fmt.Println("Bot is now running. Press Ctrl+C to stop.")
	fmt.Println("Listening for messages...\n")

	// Set up context for graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Handle signals
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-sigChan
		fmt.Println("\nShutting down...")
		cancel()
	}()

	// Create event stream (0 = use default 100ms poll interval)
	stream, err := platform.NewEventStream(ctx, 100, 0)
	if err != nil {
		log.Fatalf("Failed to create event stream: %v", err)
	}
	defer stream.Close()

	// Create event router
	router := comm.NewEventRouter()

	// Handle message posted events
	router.OnMessagePosted(func(event *comm.Event) {
		// Parse the message from event data
		if data, ok := event.Data.(map[string]interface{}); ok {
			senderID, _ := data["sender_id"].(string) // Changed from user_id to sender_id
			channelID, _ := data["channel_id"].(string)
			text, _ := data["text"].(string)

			// Ignore messages from the bot itself
			if senderID == currentUser.ID {
				return
			}

			fmt.Printf("[MESSAGE] Channel: %s, User: %s, Text: %s\n", channelID, senderID, text)

			// Simple echo bot: respond to messages that start with "!echo"
			if strings.HasPrefix(text, "!echo ") {
				response := strings.TrimPrefix(text, "!echo ")
				_, err := platform.SendMessage(channelID, response)
				if err != nil {
					log.Printf("Failed to send message: %v", err)
				} else {
					fmt.Printf("[BOT] Echoed: %s\n", response)
				}
			}

			// Respond to "!hello"
			if strings.TrimSpace(text) == "!hello" {
				_, err := platform.SendMessage(channelID, fmt.Sprintf("Hello! I'm @%s, a bot powered by libcommunicator!", currentUser.Username))
				if err != nil {
					log.Printf("Failed to send message: %v", err)
				}
			}

			// Respond to "!help"
			if strings.TrimSpace(text) == "!help" {
				helpText := `Available commands:
- !hello - Say hello
- !echo <text> - Echo the text back
- !help - Show this help message`
				_, err := platform.SendMessage(channelID, helpText)
				if err != nil {
					log.Printf("Failed to send message: %v", err)
				}
			}
		}
	})

	// Handle user typing events
	router.OnUserTyping(func(event *comm.Event) {
		fmt.Printf("[TYPING] User: %s, Channel: %s\n", event.UserID, event.ChannelID)
	})

	// Handle user status changes
	router.OnUserStatusChanged(func(event *comm.Event) {
		fmt.Printf("[STATUS] User: %s, Status: %s\n", event.UserID, event.Status)
	})

	// Handle channel events
	router.OnChannelCreated(func(event *comm.Event) {
		fmt.Printf("[CHANNEL] Channel created\n")
	})

	router.OnUserJoinedChannel(func(event *comm.Event) {
		fmt.Printf("[JOIN] User: %s joined channel: %s\n", event.UserID, event.ChannelID)
	})

	router.OnUserLeftChannel(func(event *comm.Event) {
		fmt.Printf("[LEAVE] User: %s left channel: %s\n", event.UserID, event.ChannelID)
	})

	// Run the router
	if err := router.Run(ctx, stream); err != nil {
		if err != context.Canceled {
			log.Printf("Router error: %v", err)
		}
	}

	fmt.Println("Bot stopped.")
}
