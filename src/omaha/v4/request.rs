use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use validator::{Validate, ValidationError};
use std::fmt;

#[derive(Debug, Clone, Deserialize)]
pub struct Extension {
	pub id: String,
	pub fp: String,
	pub version: String,
}

// Used to match the inner JSON structure
#[derive(Debug, Deserialize, Validate)]
struct CachedItem {
	#[serde(default)]
	sha256: String,
}

#[derive(Debug, Deserialize)]
struct App {
	#[serde(rename = "appid")]
	app_id: String,
	version: String,
	#[serde(default, rename = "cached_items")]
	cached_items: Vec<CachedItem>,
}

#[derive(Debug, Deserialize, Validate)]
struct RequestWrapper {
	#[serde(rename = "@os")]
	os: String,
	#[serde(rename = "@updater")]
	updater: String,
	#[serde(default)]
	apps: Vec<App>,
	#[validate(required)]
	protocol: Option<String>,
	#[serde(default)]
	acceptformat: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct JSONRequest {
	#[validate]
	request: Option<RequestWrapper>,
}

pub struct UpdateRequest(pub Vec<Extension>);

impl<'de> Deserialize<'de> for UpdateRequest {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let raw = JSONRequest::deserialize(deserializer).map_err(de::Error::custom)?;

		// Validate top-level optional request
		if raw.request.is_none() {
			return Err(de::Error::custom("missing required field: request"));
		}
		let request = raw.request.unwrap();

		// Validate protocol is present
		if request.protocol.is_none() {
			return Err(de::Error::custom("missing required field: protocol"));
		}

		// Per original Go logic: build Extensions from Apps
		let mut extensions = Vec::new();
		for app in request.apps {
			let fp = app.cached_items.first().map(|c| c.sha256.clone()).unwrap_or_default();
			extensions.push(Extension {
				id: app.app_id,
				fp,
				version: app.version,
			});
		}

		Ok(UpdateRequest(extensions))
	}
}