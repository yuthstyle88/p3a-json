use serde::{Deserialize};
use quick_xml::de::from_str as from_xml_str;

/// Extension and Extensions – placeholder: adapt to your types
#[derive(Debug, Clone)]
pub struct Extension {
	// Fill in fields as appropriate
}
pub type Extensions = Vec<Extension>;

/// Protocol trait to be implemented by each protocol version
pub trait Protocol {
	fn get_version(&self) -> &str;

	fn parse_request(&self, data: &[u8], content_type: &str) -> Result<Extensions, String>;

	fn format_update_response(&self, exts: &Extensions, content_type: &str) -> Result<Vec<u8>, String>;

	fn format_webstore_response(&self, exts: &Extensions, content_type: &str) -> Result<Vec<u8>, String>;
}

/// DetectProtocolVersion logic in Rust
pub fn detect_protocol_version(data: &[u8], content_type: &str) -> Result<String, String> {
	if data.is_empty() {
		return Ok("3.1".to_string());
	}

	if is_json_request(content_type) {
		#[derive(Deserialize)]
		struct ProtocolRequest {
			request: ProtocolField,
		}
		#[derive(Deserialize)]
		struct ProtocolField {
			protocol: String,
		}

		let req: ProtocolRequest = serde_json::from_slice(data)
			.map_err(|e| format!("error parsing JSON request: {}", e))?;
		if req.request.protocol.is_empty() {
			return Err("malformed JSON request, missing 'protocol' field".to_string());
		}
		return Ok(req.request.protocol);
	}

	// Try XML
	#[derive(Deserialize)]
	struct XmlRequest {
		#[serde(rename = "protocol", default)]
		protocol: String,
	}

	let s = std::str::from_utf8(data).map_err(|e| format!("invalid UTF-8: {}", e))?;
	#[derive(Deserialize)]
	struct Root {
		#[serde(rename = "protocol", attr)]
		protocol: String,
	}
	let root: Root = from_xml_str(s)
		.map_err(|e| format!("error parsing XML: {}", e))?;

	if root.protocol.is_empty() {
		return Err("protocol attribute not found in request element".to_string());
	}
	Ok(root.protocol)
}

/// Checks if the request content-type is JSON
pub fn is_json_request(content_type: &str) -> bool {
	content_type == "application/json"
}