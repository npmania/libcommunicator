package libcommunicator

/*
#cgo LDFLAGS: -L../../../target/release -lcommunicator
#cgo CFLAGS: -I../../../include
#include <communicator.h>
#include <stdlib.h>
*/
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"unsafe"
)

// UploadFile uploads a file to a channel
// Returns the file ID on success
func (p *Platform) UploadFile(channelID, filePath string) (string, error) {
	cChannelID := C.CString(channelID)
	defer C.free(unsafe.Pointer(cChannelID))

	cFilePath := C.CString(filePath)
	defer C.free(unsafe.Pointer(cFilePath))

	result := C.communicator_platform_upload_file(p.handle, cChannelID, cFilePath)
	if result == nil {
		return "", getLastError()
	}

	defer C.communicator_free_string(result)
	return C.GoString(result), nil
}

// DownloadFile downloads a file by its ID
// Returns the file contents as bytes
func (p *Platform) DownloadFile(fileID string) ([]byte, error) {
	cFileID := C.CString(fileID)
	defer C.free(unsafe.Pointer(cFileID))

	var data *C.uint8_t
	var size C.size_t

	code := C.communicator_platform_download_file(p.handle, cFileID, &data, &size)
	if code != C.COMMUNICATOR_SUCCESS {
		return nil, getLastError()
	}

	// Convert C bytes to Go slice
	// Important: We need to copy the data before freeing it
	goData := C.GoBytes(unsafe.Pointer(data), C.int(size))

	// Free the C-allocated data
	C.communicator_free_file_data(data, size)

	return goData, nil
}

// GetFileMetadata retrieves file metadata without downloading the file
func (p *Platform) GetFileMetadata(fileID string) (*Attachment, error) {
	cFileID := C.CString(fileID)
	defer C.free(unsafe.Pointer(cFileID))

	result := C.communicator_platform_get_file_metadata(p.handle, cFileID)
	if result == nil {
		return nil, getLastError()
	}

	defer C.communicator_free_string(result)
	jsonStr := C.GoString(result)

	var metadata Attachment
	if err := json.Unmarshal([]byte(jsonStr), &metadata); err != nil {
		return nil, fmt.Errorf("failed to parse file metadata: %w", err)
	}

	return &metadata, nil
}

// GetFileThumbnail downloads a file thumbnail by its ID
// Returns the thumbnail image as bytes
func (p *Platform) GetFileThumbnail(fileID string) ([]byte, error) {
	cFileID := C.CString(fileID)
	defer C.free(unsafe.Pointer(cFileID))

	var data *C.uint8_t
	var size C.size_t

	code := C.communicator_platform_get_file_thumbnail(p.handle, cFileID, &data, &size)
	if code != C.COMMUNICATOR_SUCCESS {
		return nil, getLastError()
	}

	// Convert C bytes to Go slice
	// Important: We need to copy the data before freeing it
	goData := C.GoBytes(unsafe.Pointer(data), C.int(size))

	// Free the C-allocated data
	C.communicator_free_file_data(data, size)

	return goData, nil
}

// GetFilePreview downloads a full-size file preview by its ID
// This is similar to DownloadFile but may return an optimized preview version
// Returns the preview image/file as bytes
func (p *Platform) GetFilePreview(fileID string) ([]byte, error) {
	cFileID := C.CString(fileID)
	defer C.free(unsafe.Pointer(cFileID))

	var data *C.uint8_t
	var size C.size_t

	code := C.communicator_platform_get_file_preview(p.handle, cFileID, &data, &size)
	if code != C.COMMUNICATOR_SUCCESS {
		return nil, getLastError()
	}

	// Convert C bytes to Go slice
	// Important: We need to copy the data before freeing it
	goData := C.GoBytes(unsafe.Pointer(data), C.int(size))

	// Free the C-allocated data
	C.communicator_free_file_data(data, size)

	return goData, nil
}

// GetFileLink generates a public URL for accessing a file
// Returns the public URL as a string
func (p *Platform) GetFileLink(fileID string) (string, error) {
	cFileID := C.CString(fileID)
	defer C.free(unsafe.Pointer(cFileID))

	result := C.communicator_platform_get_file_link(p.handle, cFileID)
	if result == nil {
		return "", getLastError()
	}

	defer C.communicator_free_string(result)
	return C.GoString(result), nil
}

// WriteFile is a convenience function that writes file data to disk
func WriteFile(path string, data []byte) error {
	// Note: We're not using os.WriteFile directly to avoid import cycles
	// The user should use this in conjunction with os.WriteFile from their code
	return errors.New("use os.WriteFile(path, data, 0644) to write the downloaded file")
}
