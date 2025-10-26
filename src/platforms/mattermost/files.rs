//! File operations for Mattermost
//!
//! This module provides functions for uploading, downloading, and managing files
//! on a Mattermost server.

use std::path::Path;

use reqwest::multipart;

use crate::error::{Error, ErrorCode, Result};

use super::client::MattermostClient;
use super::types::FileInfo;

impl MattermostClient {
    /// Upload a file to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID where the file will be uploaded
    /// * `file_path` - Path to the file to upload
    /// * `client_id` - Optional client ID for tracking the upload
    ///
    /// # Returns
    /// A Result containing the FileInfo metadata for the uploaded file
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = mattermost::MattermostClient::new("https://example.com")?;
    /// use std::path::Path;
    /// let file_info = client.upload_file("channel_id", Path::new("document.pdf"), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file(
        &self,
        channel_id: &str,
        file_path: &Path,
        client_id: Option<&str>,
    ) -> Result<FileInfo> {
        // Read the file from disk
        let file_data = tokio::fs::read(file_path).await.map_err(|e| {
            Error::new(
                ErrorCode::InvalidArgument,
                format!("Failed to read file: {e}"),
            )
        })?;

        // Get the filename
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                Error::new(ErrorCode::InvalidArgument, "Invalid file path")
            })?;

        // Upload the file bytes
        self.upload_file_bytes(channel_id, filename, file_data, client_id)
            .await
    }

    /// Upload file bytes to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID where the file will be uploaded
    /// * `filename` - The name of the file
    /// * `file_data` - The file contents as bytes
    /// * `client_id` - Optional client ID for tracking the upload
    ///
    /// # Returns
    /// A Result containing the FileInfo metadata for the uploaded file
    pub async fn upload_file_bytes(
        &self,
        channel_id: &str,
        filename: &str,
        file_data: Vec<u8>,
        client_id: Option<&str>,
    ) -> Result<FileInfo> {
        // Build the multipart form
        let file_part = multipart::Part::bytes(file_data).file_name(filename.to_string());

        let mut form = multipart::Form::new()
            .text("channel_id", channel_id.to_string())
            .part("files", file_part);

        if let Some(cid) = client_id {
            form = form.text("client_ids", cid.to_string());
        }

        // Send the request
        let url = self.api_url("/files");
        let mut request = self.http_client.post(&url);

        if let Some(token) = self.get_token().await {
            request = request.bearer_auth(token);
        }

        let response = request.multipart(form).send().await.map_err(|e| {
            Error::new(ErrorCode::NetworkError, format!("Upload failed: {e}"))
        })?;

        // Parse the response
        #[derive(serde::Deserialize)]
        struct UploadResponse {
            file_infos: Vec<FileInfo>,
            #[allow(dead_code)]
            client_ids: Option<Vec<String>>,
        }

        let upload_response: UploadResponse = self.handle_response(response).await?;

        upload_response
            .file_infos
            .into_iter()
            .next()
            .ok_or_else(|| Error::new(ErrorCode::Unknown, "No file info returned from upload"))
    }

    /// Download a file by its ID
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file to download
    ///
    /// # Returns
    /// A Result containing the file contents as bytes
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = mattermost::MattermostClient::new("https://example.com")?;
    /// let file_bytes = client.download_file("file_id").await?;
    /// tokio::fs::write("downloaded.pdf", &file_bytes).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let endpoint = format!("/files/{file_id}");
        let response = self.get(&endpoint).await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::new(
                ErrorCode::NetworkError,
                format!("Failed to download file: {error_text}"),
            ));
        }

        response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            Error::new(
                ErrorCode::NetworkError,
                format!("Failed to read file data: {e}"),
            )
        })
    }

    /// Get file metadata without downloading the file
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file
    ///
    /// # Returns
    /// A Result containing the FileInfo metadata
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = mattermost::MattermostClient::new("https://example.com")?;
    /// let file_info = client.get_file_info("file_id").await?;
    /// println!("File size: {} bytes", file_info.size);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_file_info(&self, file_id: &str) -> Result<FileInfo> {
        let endpoint = format!("/files/{file_id}/info");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Download a file thumbnail by its ID
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file
    ///
    /// # Returns
    /// A Result containing the thumbnail image bytes
    ///
    /// # Notes
    /// Thumbnails are only available for image and video files.
    /// Returns an error if the file doesn't have a thumbnail.
    pub async fn get_file_thumbnail(&self, file_id: &str) -> Result<Vec<u8>> {
        let endpoint = format!("/files/{file_id}/thumbnail");
        let response = self.get(&endpoint).await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::new(
                ErrorCode::NetworkError,
                format!("Failed to download thumbnail: {error_text}"),
            ));
        }

        response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            Error::new(
                ErrorCode::NetworkError,
                format!("Failed to read thumbnail data: {e}"),
            )
        })
    }

    /// Download a file preview by its ID
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file
    ///
    /// # Returns
    /// A Result containing the preview image bytes
    ///
    /// # Notes
    /// Previews are larger than thumbnails but smaller than the original file.
    /// Available for image and video files.
    pub async fn get_file_preview(&self, file_id: &str) -> Result<Vec<u8>> {
        let endpoint = format!("/files/{file_id}/preview");
        let response = self.get(&endpoint).await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::new(
                ErrorCode::NetworkError,
                format!("Failed to download preview: {error_text}"),
            ));
        }

        response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            Error::new(
                ErrorCode::NetworkError,
                format!("Failed to read preview data: {e}"),
            )
        })
    }

    /// Get a public link for a file
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file
    ///
    /// # Returns
    /// A Result containing the public link URL as a String
    ///
    /// # Notes
    /// The link allows access to the file without authentication.
    /// Requires proper permissions on the server.
    pub async fn get_file_link(&self, file_id: &str) -> Result<String> {
        let endpoint = format!("/files/{file_id}/link");
        let response = self.get(&endpoint).await?;

        #[derive(serde::Deserialize)]
        struct LinkResponse {
            link: String,
        }

        let link_response: LinkResponse = self.handle_response(response).await?;
        Ok(link_response.link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations_exist() {
        // This test just ensures the module compiles and basic types exist
        // Integration tests would require a real Mattermost server
    }
}
