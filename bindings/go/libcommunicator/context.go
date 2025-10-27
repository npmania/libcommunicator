package libcommunicator

/*
#cgo LDFLAGS: -L../../../target/release -lcommunicator
#cgo CFLAGS: -I../../../include
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"runtime"
	"unsafe"
)

// Context represents a libcommunicator context instance
// Contexts provide isolated configuration and logging environments
type Context struct {
	handle C.CommunicatorContext
}

// LogLevel represents the severity level of a log message
type LogLevel int

const (
	LogDebug   LogLevel = 0
	LogInfo    LogLevel = 1
	LogWarning LogLevel = 2
	LogError   LogLevel = 3
)

// String returns the string representation of the log level
func (l LogLevel) String() string {
	switch l {
	case LogDebug:
		return "DEBUG"
	case LogInfo:
		return "INFO"
	case LogWarning:
		return "WARNING"
	case LogError:
		return "ERROR"
	default:
		return "UNKNOWN"
	}
}

// LogCallback is a function type for receiving log messages from the library
type LogCallback func(level LogLevel, message string)

// NewContext creates a new context with the given ID
// The ID should be unique and is used for identification purposes
func NewContext(id string) (*Context, error) {
	cID := C.CString(id)
	defer C.free(unsafe.Pointer(cID))

	handle := C.communicator_context_create(cID)
	if handle == nil {
		return nil, getLastError()
	}

	ctx := &Context{handle: handle}
	runtime.SetFinalizer(ctx, (*Context).Destroy)
	return ctx, nil
}

// Initialize initializes the context
// Must be called before using the context
func (c *Context) Initialize() error {
	if c.handle == nil {
		return ErrInvalidContext
	}

	code := C.communicator_context_initialize(c.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// IsInitialized checks if the context is initialized
func (c *Context) IsInitialized() (bool, error) {
	if c.handle == nil {
		return false, ErrInvalidContext
	}

	result := C.communicator_context_is_initialized(c.handle)
	if result < 0 {
		return false, getLastError()
	}

	return result == 1, nil
}

// SetConfig sets a configuration value
func (c *Context) SetConfig(key, value string) error {
	if c.handle == nil {
		return ErrInvalidContext
	}

	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))

	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))

	code := C.communicator_context_set_config(c.handle, cKey, cValue)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// GetConfig retrieves a configuration value
// Returns an empty string if the key doesn't exist
func (c *Context) GetConfig(key string) (string, error) {
	if c.handle == nil {
		return "", ErrInvalidContext
	}

	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))

	cValue := C.communicator_context_get_config(c.handle, cKey)
	if cValue == nil {
		// Check if error or just missing key
		if err := getLastError(); err != nil {
			return "", err
		}
		return "", nil
	}

	defer C.communicator_free_string(cValue)
	return C.GoString(cValue), nil
}

// Shutdown shuts down the context
// Should be called before destroying the context
func (c *Context) Shutdown() error {
	if c.handle == nil {
		return ErrInvalidContext
	}

	code := C.communicator_context_shutdown(c.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// Destroy destroys the context and frees its memory
// After calling this, the context must not be used
func (c *Context) Destroy() {
	if c.handle != nil {
		C.communicator_context_destroy(c.handle)
		c.handle = nil
	}
}

// SetLogCallback sets a callback function to receive log messages
// The callback will be called from the library's thread context
// Note: Due to cgo restrictions, this is not yet fully implemented
func (c *Context) SetLogCallback(callback LogCallback) error {
	if c.handle == nil {
		return ErrInvalidContext
	}

	// TODO: Implement callback bridging
	// This requires storing the Go callback and creating a C callback wrapper
	// that calls into Go. This is complex due to cgo callback restrictions.
	return ErrUnsupported
}

// ClearLogCallback clears any previously set log callback
func (c *Context) ClearLogCallback() error {
	if c.handle == nil {
		return ErrInvalidContext
	}

	code := C.communicator_context_clear_log_callback(c.handle)
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	return nil
}

// ErrInvalidContext is returned when operations are attempted on a nil or destroyed context
var ErrInvalidContext = newError(ErrorInvalidState, "invalid context handle")

// ErrUnsupported is returned for unsupported operations
var ErrUnsupported = newError(ErrorUnsupported, "operation not supported")

// newError creates a new error with the given code and message
func newError(code ErrorCode, message string) error {
	return &LibError{Code: code, Message: message}
}

// LibError represents a library error with an error code
type LibError struct {
	Code    ErrorCode
	Message string
}

func (e *LibError) Error() string {
	return e.Message
}
