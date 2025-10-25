# Makefile for libcommunicator C examples

# Detect OS for library extension
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LIB_EXT = so
    LIB_PREFIX = lib
endif
ifeq ($(UNAME_S),Darwin)
    LIB_EXT = dylib
    LIB_PREFIX = lib
endif

# Paths
LIB_DIR = ../target/release
LIB_NAME = $(LIB_PREFIX)communicator.$(LIB_EXT)
INCLUDE_DIR = ../include

# Compiler settings
CC = gcc
CFLAGS = -I$(INCLUDE_DIR) -Wall -Wextra
LDFLAGS = -L$(LIB_DIR) -lcommunicator

# Set rpath for Linux/macOS
ifeq ($(UNAME_S),Linux)
    LDFLAGS += -Wl,-rpath,$(shell realpath $(LIB_DIR))
endif
ifeq ($(UNAME_S),Darwin)
    LDFLAGS += -Wl,-rpath,@loader_path/$(LIB_DIR)
endif

# Targets
all: version_example ffi_patterns_demo mattermost_demo

version_example: version_example.c
	@echo "Building $@..."
	@if [ ! -f "$(LIB_DIR)/$(LIB_NAME)" ]; then \
		echo "Error: Library not found. Please run 'cargo build --release' first."; \
		exit 1; \
	fi
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)
	@echo "Done! Run with: ./version_example"

ffi_patterns_demo: ffi_patterns_demo.c
	@echo "Building $@..."
	@if [ ! -f "$(LIB_DIR)/$(LIB_NAME)" ]; then \
		echo "Error: Library not found. Please run 'cargo build --release' first."; \
		exit 1; \
	fi
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)
	@echo "Done! Run with: ./ffi_patterns_demo"

mattermost_demo: mattermost_demo.c
	@echo "Building $@..."
	@if [ ! -f "$(LIB_DIR)/$(LIB_NAME)" ]; then \
		echo "Error: Library not found. Please run 'cargo build --release' first."; \
		exit 1; \
	fi
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)
	@echo "Done! Run with: ./mattermost_demo <server_url> <login_id_or_token> <password_or_empty> <team_id>"

clean:
	rm -f version_example ffi_patterns_demo mattermost_demo

.PHONY: all clean
