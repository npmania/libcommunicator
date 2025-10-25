package libcommunicator

/*
#cgo LDFLAGS: -L../../../target/release -lcommunicator
#cgo CFLAGS: -I../../../include
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

// Version information
type Version struct {
	Major uint32
	Minor uint32
	Patch uint32
	Full  string
}

// ErrorCode represents library error codes
type ErrorCode int

const (
	Success             ErrorCode = 0
	ErrorUnknown        ErrorCode = 1
	ErrorInvalidArg     ErrorCode = 2
	ErrorNullPointer    ErrorCode = 3
	ErrorOutOfMemory    ErrorCode = 4
	ErrorInvalidUTF8    ErrorCode = 5
	ErrorNetwork        ErrorCode = 6
	ErrorAuthFailed     ErrorCode = 7
	ErrorNotFound       ErrorCode = 8
	ErrorPermDenied     ErrorCode = 9
	ErrorTimeout        ErrorCode = 10
	ErrorInvalidState   ErrorCode = 11
)

var initialized bool

// Init initializes the library
// Must be called before using any other functions
func Init() error {
	if initialized {
		return nil
	}

	code := C.communicator_init()
	if code != C.COMMUNICATOR_SUCCESS {
		return getLastError()
	}

	initialized = true
	return nil
}

// Cleanup cleans up the library
// Should be called when done using the library
func Cleanup() {
	if !initialized {
		return
	}

	C.communicator_cleanup()
	initialized = false
}

// GetVersion returns the library version information
func GetVersion() Version {
	return Version{
		Major: uint32(C.communicator_version_major()),
		Minor: uint32(C.communicator_version_minor()),
		Patch: uint32(C.communicator_version_patch()),
		Full:  C.GoString(C.communicator_version()),
	}
}

// getLastError retrieves the last error from the library
func getLastError() error {
	code := C.communicator_last_error_code()
	if code == C.COMMUNICATOR_SUCCESS {
		return nil
	}

	msg := C.communicator_last_error_message()
	if msg == nil {
		codeStr := C.communicator_error_code_string(code)
		return fmt.Errorf("libcommunicator error %d: %s", code, C.GoString(codeStr))
	}

	defer C.communicator_free_string(msg)
	return fmt.Errorf("libcommunicator error %d: %s", code, C.GoString(msg))
}

// clearError clears the last error
func clearError() {
	C.communicator_clear_error()
}

// freeString frees a C string allocated by the library
func freeString(s *C.char) {
	if s != nil {
		C.communicator_free_string(s)
	}
}

// cString converts a Go string to a C string
// The caller is responsible for freeing the returned string
func cString(s string) *C.char {
	return C.CString(s)
}

// cStringFree converts a Go string to a C string and returns a cleanup function
func cStringFree(s string) (*C.char, func()) {
	cs := C.CString(s)
	return cs, func() { C.free(unsafe.Pointer(cs)) }
}

// ensureInitialized checks if the library is initialized
func ensureInitialized() error {
	if !initialized {
		return errors.New("library not initialized, call Init() first")
	}
	return nil
}
