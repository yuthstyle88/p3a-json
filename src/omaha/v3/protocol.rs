use serde::{Deserialize, Serialize};
use quick_xml::de::from_str as from_xml_str;
use quick_xml::se::to_string as to_xml_string;

use std::collections::HashSet;
use std::collections::HashMap;

/// Supported v3 protocol versions
pub static SUPPORTED_V3_VERSIONS: &[&str] = &["3.0", "3.1"];

#[derive(Debug, Clone)]
pub struct VersionedHandler {
	version: String,
}

impl VersionedHandler {
	pub fn new(version: &str) -> Result<Self, String> {
		if SUPPORTED_V3_VERSIONS.contains(&version) {
			Ok(Self { version: version.to_string() })
		} else {
			Err(format!("unsupported protocol version: {}", version))
		}
	}

	pub fn get_version(&self) -> &str {
		&self.version
	}

	pub fn parse_request(
		&self,
		data: &[u8],
		content_type: &str,
	) -> Result<Extensions, String> {
		if content_type == "application/json" {
			let req: UpdateRequest = serde_json::from_slice(data)
				.map_err(|e| e.to_string())?;
			Ok(req.into_extensions())
		} else if content_type == "application/xml" {
			let s = std::str::from_utf8(data).map_err(|e| e.to_string())?;
			let req: UpdateRequest = from_xml_str(s)
				.map_err(|e| e.to_string())?;
			Ok(req.into_extensions())
		} else {
			Err("unsupported content type".into())
		}
	}

	pub fn format_update_response(
		&self,
		extensions: Extensions,
		content_type: &str,
	) -> Result<Vec<u8>, String> {
		let resp = UpdateResponse::from(extensions);
		if content_type == "application/json" {
			serde_json::to_vec(&resp).map_err(|e| e.to_string())
		} else if content_type == "application/xml" {
			let xml = to_xml_string(&resp).map_err(|e| e.to_string())?;
			Ok(xml.into_bytes())
		} else {
			Err("unsupported content type".into())
		}
	}

	pub fn format_webstore_response(
		&self,
		extensions: Extensions,
		content_type: &str,
	) -> Result<Vec<u8>, String> {
		let resp = WebStoreResponse::from(UpdateResponse::from(extensions));
		if content_type == "application/json" {
			serde_json::to_vec(&resp).map_err(|e| e.to_string())
		} else if content_type == "application/xml" {
			let xml = to_xml_string(&resp).map_err(|e| e.to_string())?;
			Ok(xml.into_bytes())
		} else {
			Err("unsupported content type".into())
		}
	}
}

/// ---- ด้านล่างคือตัวอย่าง struct สำหรับ request/response และ extension ----

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Extension {
	pub id: String,
	pub version: String,
	// ... add other fields as needed
}

pub type Extensions = Vec<Extension>;

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateRequest {
	pub extensions: Extensions,
}

impl UpdateRequest {
	pub fn into_extensions(self) -> Extensions {
		self.extensions
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateResponse {
	#[serde(flatten)]
	pub extensions: HashMap<String, Extension>,
}

impl From<Extensions> for UpdateResponse {
	fn from(exts: Extensions) -> Self {
		let mut map = HashMap::new();
		for ext in exts {
			map.insert(ext.id.clone(), ext);
		}
		UpdateResponse { extensions: map }
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebStoreResponse {
	pub response: UpdateResponse,
}

impl From<UpdateResponse> for WebStoreResponse {
	fn from(resp: UpdateResponse) -> Self {
		WebStoreResponse { response: resp }
	}
}