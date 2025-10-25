.PHONY: all build examples mattermost_demo simple_bot clean test

# Build the Rust library first
RUST_LIB := ../../target/release/libcommunicator.so
LIB_DIR := $(shell pwd)/../../target/release
INCLUDE_DIR := $(shell pwd)/../../include

# Export library path for linking
export LD_LIBRARY_PATH := $(LIB_DIR):$(LD_LIBRARY_PATH)
export CGO_LDFLAGS := -L$(LIB_DIR) -lcommunicator
export CGO_CFLAGS := -I$(INCLUDE_DIR)

all: build examples

# Ensure Rust library is built
$(RUST_LIB):
	@echo "Building Rust library..."
	cd ../.. && cargo build --release

# Build the Go package
build: $(RUST_LIB)
	@echo "Building Go package..."
	cd libcommunicator && go build

# Build all examples
examples: mattermost_demo simple_bot

# Build mattermost_demo example
mattermost_demo: $(RUST_LIB)
	@echo "Building mattermost_demo..."
	cd examples/mattermost_demo && \
		go mod edit -replace libcommunicator=../../libcommunicator && \
		go get libcommunicator && \
		CGO_LDFLAGS="$(CGO_LDFLAGS)" CGO_CFLAGS="$(CGO_CFLAGS)" go build -o mattermost_demo

# Build simple_bot example
simple_bot: $(RUST_LIB)
	@echo "Building simple_bot..."
	cd examples/simple_bot && \
		go mod edit -replace libcommunicator=../../libcommunicator && \
		go get libcommunicator && \
		CGO_LDFLAGS="$(CGO_LDFLAGS)" CGO_CFLAGS="$(CGO_CFLAGS)" go build -o simple_bot

# Run tests
test: $(RUST_LIB)
	@echo "Running Go tests..."
	cd libcommunicator && \
		CGO_LDFLAGS="$(CGO_LDFLAGS)" CGO_CFLAGS="$(CGO_CFLAGS)" go test -v

# Clean build artifacts
clean:
	@echo "Cleaning Go build artifacts..."
	rm -f examples/mattermost_demo/mattermost_demo
	rm -f examples/simple_bot/simple_bot
	cd libcommunicator && go clean

# Show help
help:
	@echo "Go Bindings Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all              - Build everything (default)"
	@echo "  build            - Build the Go package"
	@echo "  examples         - Build all examples"
	@echo "  mattermost_demo  - Build mattermost_demo example"
	@echo "  simple_bot       - Build simple_bot example"
	@echo "  test             - Run tests"
	@echo "  clean            - Clean build artifacts"
	@echo "  help             - Show this help message"
	@echo ""
	@echo "Environment:"
	@echo "  LIB_DIR=$(LIB_DIR)"
	@echo "  INCLUDE_DIR=$(INCLUDE_DIR)"
