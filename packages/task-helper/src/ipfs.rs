use reqwest::multipart;
use serde::Deserialize;

use crate::command::IpfsKind;

/// Result of uploading a file to IPFS
#[derive(Debug, Clone)]
pub struct IpfsFile {
    /// The content identifier (CID) of the uploaded file
    pub cid: String,

    /// The IPFS URI (e.g., "ipfs://Qm...")
    pub uri: String,

    /// The gateway URL for accessing the file via HTTP
    pub gateway_url: String,
}

/// Upload a single file to an IPFS HTTP API and return the CID and URIs.
impl IpfsFile {
    pub async fn upload(
        kind: IpfsKind,
        bytes: Vec<u8>,
        filename: &str,
        // The base URL for the IPFS gateway (e.g., "http://127.0.0.1:8080")
        // This also applies to Pinata uploads
        gateway_base: &str,
        // The base URL for the IPFS API (e.g., "http://127.0.0.1:5001")
        kubo_api_base: &str,
        kubo_wrap_with_directory: bool,
    ) -> anyhow::Result<Self> {
        match kind {
            IpfsKind::Kubo => {
                Self::upload_kubo(
                    bytes,
                    filename,
                    kubo_api_base,
                    gateway_base,
                    kubo_wrap_with_directory,
                )
                .await
            }
            IpfsKind::Pinata => {
                let jwt = std::env::var("REMOTE_IPFS_PINATA_JWT").map_err(|_| {
                    anyhow::anyhow!(
                        "REMOTE_IPFS_PINATA_JWT environment variable not set for Pinata IPFS upload"
                    )
                })?;
                Self::upload_pinata(bytes, filename, &jwt, gateway_base).await
            }
        }
    }

    pub async fn upload_pinata(
        bytes: Vec<u8>,
        filename: &str,
        // The JWT for Pinata authentication
        jwt: &str,
        // The base URL for the IPFS gateway (e.g., "http://my-pinata-gateway.com")
        gateway_base: &str,
    ) -> anyhow::Result<Self> {
        let part = multipart::Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")?;

        let form = multipart::Form::new()
            .part("file", part)
            .text("network", "public")
            .text("name", filename.to_string());

        let client = reqwest::Client::new();

        let resp = client
            .post("https://uploads.pinata.cloud/v3/files")
            .bearer_auth(jwt)
            .multipart(form)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload to Pinata: {}", e))?;

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct PinataResponse {
            data: PinataResponseData,
        }
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct PinataResponseData {
            // pub id: String,
            // pub name: String,
            pub cid: String,
            // pub created_at: String,
            // pub size: f64,
            // pub number_of_files: f64,
            // pub mime_type: String,
            // pub user_id: String,
            // pub group_id: String,
            // pub is_duplicate: bool,
        }

        let resp: PinataResponse = resp
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("Pinata error: {}", e))?
            .json()
            .await?;

        let PinataResponseData { cid, .. } = resp.data;

        // Direct file upload - the CID points directly to the file content
        let uri = format!("ipfs://{}", cid);
        let gateway_url = format!("{}/ipfs/{}", gateway_base, cid);

        Ok(Self {
            cid,
            uri,
            gateway_url,
        })
    }

    pub async fn upload_kubo(
        bytes: Vec<u8>,
        filename: &str,
        // The base URL for the IPFS API (e.g., "http://127.0.0.1:5001")
        api_base: &str,
        // The base URL for the IPFS gateway (e.g., "http://127.0.0.1:8080")
        gateway_base: &str,
        wrap_with_directory: bool,
    ) -> anyhow::Result<Self> {
        // Request CIDv1 with base32 encoding for modern, case-insensitive URIs
        // pin=true keeps the file in the local IPFS repository
        // Strip trailing slash from api_base to avoid double slashes
        let api_base = api_base.trim_end_matches('/');
        let url = format!(
            "{}/api/v0/add?cid-version=1&pin=true&wrap-with-directory={}",
            api_base, wrap_with_directory
        );

        let part = multipart::Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")?;

        let form = multipart::Form::new().part("file", part);

        let client = reqwest::Client::new();

        let resp = client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to IPFS API at {}: {}", api_base, e))?;

        // The /api/v0/add endpoint returns NDJSON (newline-delimited JSON)
        // - For a single file: one JSON line with the file's CID
        // - With wrap-with-directory=true: two lines (file CID, then directory CID)
        // We want the last line, which is the root CID
        let text = resp
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("IPFS API error: {}", e))?
            .text()
            .await?;

        let last_line = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .next_back()
            .ok_or_else(|| anyhow::anyhow!("Empty response from IPFS API"))?;

        /// Response from the IPFS Kubo API's `/api/v0/add` endpoint.
        /// Kubo returns field names in PascalCase, so we use serde rename.
        #[derive(Debug, Deserialize)]
        struct AddResponse {
            #[serde(rename = "Name")]
            _name: Option<String>,

            #[serde(rename = "Hash")]
            hash: String, // The CID

            #[serde(rename = "Size")]
            _size: Option<String>,
        }

        let parsed: AddResponse = serde_json::from_str(last_line)
            .map_err(|e| anyhow::anyhow!("Failed to parse IPFS response: {}", e))?;

        // Build the URIs based on whether we wrapped with a directory
        // Strip trailing slash from gateway_base to avoid double slashes
        let gateway_base = gateway_base.trim_end_matches('/');
        let (cid, uri, gateway_url) = if wrap_with_directory {
            // When wrapping, the last line contains the directory CID
            // The URI should include the filename as a path
            let root_cid = parsed.hash;
            let uri = format!("ipfs://{}/{}", root_cid, filename);
            let gateway = format!("{}/ipfs/{}/{}", gateway_base, root_cid, filename);
            (root_cid, uri, gateway)
        } else {
            // Direct file upload - the CID points directly to the file content
            let cid = parsed.hash;
            let uri = format!("ipfs://{}", cid);
            let gateway = format!("{}/ipfs/{}", gateway_base, cid);
            (cid, uri, gateway)
        };

        Ok(Self {
            cid,
            uri,
            gateway_url,
        })
    }
}
