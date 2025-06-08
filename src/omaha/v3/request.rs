use serde::{Deserialize, Deserializer};
use quick_xml::de::{from_str as from_xml_str};
use std::fmt;

#[derive(Debug, Clone, Deserialize)]
pub struct Extension {
	pub id: String,
	pub fp: String,
	pub version: String,
}

/// UpdateRequest represents an Omaha v3 update request
pub type UpdateRequest = Vec<Extension>;

// ========== JSON Parser ==========

pub fn parse_update_request_json(data: &[u8]) -> Result<UpdateRequest, String> {
	#[derive(Deserialize)]
	struct Package {
		fp: Option<String>,
	}
	#[derive(Deserialize)]
	struct Packages {
		#[serde(rename = "package")]
		package: Option<Vec<Package>>,
	}
	#[derive(Deserialize)]
	struct App {
		#[serde(rename = "appid")]
		app_id: String,
		#[serde(default)]
		fp: Option<String>,
		version: String,
		#[serde(default)]
		packages: Option<Packages>,
	}
	#[derive(Deserialize)]
	struct RequestWrapper {
		#[serde(rename = "@os")]
		_os: Option<String>,
		#[serde(rename = "@updater")]
		_updater: Option<String>,
		#[serde(default)]
		app: Vec<App>,
		#[serde(default)]
		protocol: Option<String>,
	}
	#[derive(Deserialize)]
	struct JSONRequest {
		request: RequestWrapper,
	}

	let req: JSONRequest = serde_json::from_slice(data).map_err(|e| e.to_string())?;

	let mut extensions = vec![];
	for app in req.request.app {
		// FP: Prefer "fp" at app level, else first package.fp
		let fp = app.fp
			.or_else(|| {
				app.packages
					.as_ref()
					.and_then(|pkgs| pkgs.package.as_ref()?.get(0).and_then(|pkg| pkg.fp.clone()))
			})
			.unwrap_or_default();
		extensions.push(Extension {
			id: app.app_id,
			fp,
			version: app.version,
		});
	}
	Ok(extensions)
}

// ========== XML Parser ==========

pub fn parse_update_request_xml(data: &[u8]) -> Result<UpdateRequest, String> {
	let s = std::str::from_utf8(data).map_err(|e| e.to_string())?;
	// Find protocol version from attributes
	let start_tag = s.find('<').ok_or("Invalid XML")?;
	let protocol_version = s[start_tag..]
		.split_whitespace()
		.find_map(|part| {
			if part.starts_with("protocol=") {
				Some(part.trim_start_matches("protocol=").trim_matches(&['"', '\''][..]).to_string())
			} else {
				None
			}
		}).unwrap_or_else(|| "3.1".to_string());

	if protocol_version == "3.0" {
		#[derive(Deserialize)]
		struct Package {
			#[serde(rename = "fp", default)]
			fp: Option<String>,
		}
		#[derive(Deserialize)]
		struct Packages {
			#[serde(rename = "package", default)]
			packages: Vec<Package>,
		}
		#[derive(Deserialize)]
		struct App {
			#[serde(rename = "appid", default)]
			appid: String,
			#[serde(rename = "version", default)]
			version: String,
			#[serde(rename = "packages", default)]
			packages: Option<Packages>,
		}
		#[derive(Deserialize)]
		struct RequestWrapper {
			#[serde(rename = "app", default)]
			app: Vec<App>,
		}
		let wrap: RequestWrapper = from_xml_str(s).map_err(|e| e.to_string())?;
		let mut exts = vec![];
		for app in wrap.app {
			// fp is first package.fp
			let fp = app.packages
				.and_then(|pkgs| pkgs.packages.get(0).and_then(|pkg| pkg.fp.clone()))
				.unwrap_or_default();
			exts.push(Extension {
				id: app.appid,
				fp,
				version: app.version,
			});
		}
		Ok(exts)
	} else {
		#[derive(Deserialize)]
		struct App {
			#[serde(rename = "appid", default)]
			appid: String,
			#[serde(rename = "fp", default)]
			fp: Option<String>,
			#[serde(rename = "version", default)]
			version: String,
		}
		#[derive(Deserialize)]
		struct RequestWrapper {
			#[serde(rename = "app", default)]
			app: Vec<App>,
		}
		let wrap: RequestWrapper = from_xml_str(s).map_err(|e| e.to_string())?;
		let mut exts = vec![];
		for app in wrap.app {
			exts.push(Extension {
				id: app.appid,
				fp: app.fp.unwrap_or_default(),
				version: app.version,
			});
		}
		Ok(exts)
	}
}