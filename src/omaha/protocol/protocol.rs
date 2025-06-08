use crate::payload::MyRequest;
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
pub fn detect_protocol_version(data: &MyRequest) -> String {
	if data.request.protocol.to_string().is_empty() {
		return "3.1".to_string();
	}
	data.request.protocol.clone()
}

/// Checks if the request content-type is JSON
pub fn is_json_request(content_type: &str) -> bool {
	content_type == "application/json"
}