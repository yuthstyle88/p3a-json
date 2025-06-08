use serde::Serialize;
use serde_json;
use quick_xml::se::{to_string as to_xml_string};

#[derive(Debug, Clone, Serialize)]
pub struct Extension {
	pub id: String,
	pub version: String,
	pub status: Option<String>,          // "ok" or "noupdate"
	pub sha256: String,
	pub fp: String,
	pub patch_list: Option<std::collections::HashMap<String, PatchInfo>>,
	// ... add other fields as necessary ...
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchInfo {
	pub namediff: String,
	pub hashdiff: String,
	pub sizediff: i32,
}

fn get_update_status(ext: &Extension) -> &str {
	if ext.status.as_deref() == Some("noupdate") {
		"noupdate"
	} else {
		"ok"
	}
}

// =========== Omaha v3 UpdateResponse JSON ===========

#[derive(Serialize)]
struct JsonURL {
	#[serde(skip_serializing_if = "String::is_empty", rename = "codebase")]
	codebase: String,
	#[serde(skip_serializing_if = "String::is_empty", rename = "codebasediff")]
	codebasediff: String,
}
#[derive(Serialize)]
struct JsonURLs { url: Vec<JsonURL> }
#[derive(Serialize)]
struct JsonPackage {
	name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	namediff: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	sizediff: Option<i32>,
	fp: String,
	#[serde(rename = "hash_sha256")]
	sha256: String,
	#[serde(skip_serializing_if = "Option::is_none", rename = "hashdiff_sha256")]
	diff_sha256: Option<String>,
	required: bool,
}
#[derive(Serialize)]
struct JsonPackages { package: Vec<JsonPackage> }
#[derive(Serialize)]
struct JsonManifest {
	version: String,
	packages: JsonPackages,
}
#[derive(Serialize)]
struct JsonUpdateCheck {
	status: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	urls: Option<JsonURLs>,
	#[serde(skip_serializing_if = "Option::is_none")]
	manifest: Option<JsonManifest>,
}
#[derive(Serialize)]
struct JsonApp {
	appid: String,
	status: String,
	updatecheck: JsonUpdateCheck,
}
#[derive(Serialize)]
struct JsonResponseWrapper {
	protocol: &'static str,
	server: &'static str,
	app: Vec<JsonApp>,
}
#[derive(Serialize)]
struct JsonResponse {
	response: JsonResponseWrapper,
}

pub fn marshal_update_response_json(exts: &[Extension]) -> String {
	let mut apps = vec![];
	for ext in exts {
		let status = get_update_status(ext).to_string();
		let extension_name = format!("extension_{}.crx", ext.version.replace('.', "_"));
		let url = format!("https://{}/release/{}/{}",
						  get_s3_extension_bucket_host(&ext.id),
						  ext.id,
						  extension_name
		);
		let diff_url = format!("https://{}/release/{}/patches/{}/",
							   get_s3_extension_bucket_host(&ext.id),
							   ext.id,
							   ext.sha256
		);

		let mut urls = vec![];
		urls.push(JsonURL { codebase: url.clone(), codebasediff: "".to_string() });

		let mut namediff = None;
		let mut sizediff = None;
		let mut diff_sha256 = None;

		// Compose patch info if available
		if let Some(list) = &ext.patch_list {
			if let Some(patch_info) = list.get(&ext.fp) {
				urls.push(JsonURL { codebase: "".to_string(), codebasediff: diff_url });
				namediff = Some(patch_info.namediff.clone());
				diff_sha256 = Some(patch_info.hashdiff.clone());
				sizediff = Some(patch_info.sizediff);
			}
		}

		let mut manifest = None;
		if status == "ok" {
			manifest = Some(JsonManifest {
				version: ext.version.clone(),
				packages: JsonPackages {
					package: vec![
						JsonPackage {
							name: extension_name,
							namediff,
							sizediff,
							fp: ext.sha256.clone(),
							sha256: ext.sha256.clone(),
							diff_sha256,
							required: true,
						}
					]
				}
			});
		}

		let app = JsonApp {
			appid: ext.id.clone(),
			status: "ok".to_string(),
			updatecheck: JsonUpdateCheck {
				status: status,
				urls: Some(JsonURLs { url: urls }),
				manifest,
			},
		};
		apps.push(app);
	}
	let wrapper = JsonResponseWrapper {
		protocol: "3.1",
		server: "prod",
		app: apps,
	};
	let response = JsonResponse { response: wrapper };
	serde_json::to_string_pretty(&response).unwrap()
}

// =========== Omaha v3 UpdateResponse XML ===========

#[derive(Serialize)]
#[serde(rename = "response")]
struct XmlResponseWrapper {
	#[serde(rename = "protocol", attr)]
	protocol: &'static str,
	#[serde(rename = "server", attr)]
	server: &'static str,
	#[serde(rename = "app")]
	app: Vec<XmlApp>,
}
#[derive(Serialize)]
struct XmlApp {
	#[serde(rename = "appid", attr)]
	appid: String,
	#[serde(rename = "updatecheck")]
	updatecheck: XmlUpdateCheck,
}
#[derive(Serialize)]
struct XmlUpdateCheck {
	#[serde(rename = "status", attr)]
	status: String,
	#[serde(skip_serializing_if = "Option::is_none", rename = "urls")]
	urls: Option<XmlURLs>,
	#[serde(skip_serializing_if = "Option::is_none", rename = "manifest")]
	manifest: Option<XmlManifest>,
}
#[derive(Serialize)]
struct XmlURLs {
	#[serde(rename = "url")]
	url: Vec<XmlURL>,
}
#[derive(Serialize)]
struct XmlURL {
	#[serde(rename = "codebase", attr)]
	codebase: String,
}
#[derive(Serialize)]
struct XmlManifest {
	#[serde(rename = "version", attr)]
	version: String,
	#[serde(rename = "packages")]
	packages: XmlPackages,
}
#[derive(Serialize)]
struct XmlPackages {
	#[serde(rename = "package")]
	package: Vec<XmlPackage>,
}
#[derive(Serialize)]
struct XmlPackage {
	#[serde(rename = "name", attr)]
	name: String,
	#[serde(rename = "hash_sha256", attr)]
	sha256: String,
	#[serde(rename = "required", attr)]
	required: bool,
}

pub fn marshal_update_response_xml(exts: &[Extension]) -> String {
	let mut apps = vec![];
	for ext in exts {
		let status = get_update_status(ext).to_string();
		let extension_name = format!("extension_{}.crx", ext.version.replace(".", "_"));
		let url = format!("https://{}/release/{}/{}",
						  get_s3_extension_bucket_host(&ext.id),
						  ext.id,
						  extension_name
		);
		let mut urls = None;
		let mut packages = None;
		let mut manifest = None;

		if status == "ok" {
			urls = Some(XmlURLs { url: vec![XmlURL { codebase: url.clone() }] });
			packages = Some(XmlPackages {
				package: vec![XmlPackage {
					name: extension_name,
					sha256: ext.sha256.clone(),
					required: true,
				}]
			});
			manifest = Some(XmlManifest {
				version: ext.version.clone(),
				packages: packages.unwrap(),
			});
		}

		let app = XmlApp {
			appid: ext.id.clone(),
			updatecheck: XmlUpdateCheck {
				status,
				urls,
				manifest,
			}
		};
		apps.push(app);
	}

	let response = XmlResponseWrapper {
		protocol: "3.1",
		server: "prod",
		app: apps,
	};

	to_xml_string(&response).unwrap()
}

// =========== WebStoreResponse JSON ===========

#[derive(Serialize)]
struct JsonGUpdate {
	protocol: &'static str,
	server: &'static str,
	app: Vec<JsonStoreApp>,
}
#[derive(Serialize)]
struct JsonStoreApp {
	appid: String,
	status: String,
	updatecheck: JsonStoreUpdateCheck,
}
#[derive(Serialize)]
struct JsonStoreUpdateCheck {
	status: String,
	codebase: String,
	version: String,
	#[serde(rename = "hash_sha256")]
	sha256: String,
}
#[derive(Serialize)]
struct JsonGUpdateResponse {
	gupdate: JsonGUpdate,
}

pub fn marshal_webstore_response_json(exts: &[Extension]) -> String {
	let mut apps = vec![];
	for ext in exts {
		let extension_name = format!("extension_{}.crx", ext.version.replace('.', "_"));
		let codebase = format!("https://{}/release/{}/{}",
							   get_s3_extension_bucket_host(&ext.id),
							   ext.id,
							   extension_name,
		);
		let app = JsonStoreApp {
			appid: ext.id.clone(),
			status: "ok".to_string(),
			updatecheck: JsonStoreUpdateCheck {
				status: "ok".to_string(),
				sha256: ext.sha256.clone(),
				version: ext.version.clone(),
				codebase,
			}
		};
		apps.push(app);
	}
	let gupdate = JsonGUpdate {
		protocol: "3.1",
		server: "prod",
		app: apps,
	};
	let response = JsonGUpdateResponse { gupdate };
	serde_json::to_string_pretty(&response).unwrap()
}

// =========== WebStoreResponse XML ===========

#[derive(Serialize)]
#[serde(rename = "gupdate")]
struct XmlGUpdate {
	#[serde(rename = "protocol", attr)]
	protocol: &'static str,
	#[serde(rename = "server", attr)]
	server: &'static str,
	#[serde(rename = "app")]
	app: Vec<XmlStoreApp>,
}
#[derive(Serialize)]
struct XmlStoreApp {
	#[serde(rename = "appid", attr)]
	appid: String,
	#[serde(rename = "status", attr)]
	status: String,
	#[serde(rename = "updatecheck")]
	updatecheck: XmlStoreUpdateCheck,
}
#[derive(Serialize)]
struct XmlStoreUpdateCheck {
	#[serde(rename = "status", attr)]
	status: String,
	#[serde(rename = "codebase", attr)]
	codebase: String,
	#[serde(rename = "version", attr)]
	version: String,
	#[serde(rename = "hash_sha256", attr)]
	sha256: String,
}

pub fn marshal_webstore_response_xml(exts: &[Extension]) -> String {
	let mut apps = vec![];
	for ext in exts {
		let extension_name = format!("extension_{}.crx", ext.version.replace('.', "_"));
		let codebase = format!("https://{}/release/{}/{}",
							   get_s3_extension_bucket_host(&ext.id),
							   ext.id,
							   extension_name,
		);
		let app = XmlStoreApp {
			appid: ext.id.clone(),
			status: "ok".to_string(),
			updatecheck: XmlStoreUpdateCheck {
				status: "ok".to_string(),
				sha256: ext.sha256.clone(),
				version: ext.version.clone(),
				codebase,
			}
		};
		apps.push(app);
	}
	let gupdate = XmlGUpdate {
		protocol: "3.1",
		server: "prod",
		app: apps,
	};
	to_xml_string(&gupdate).unwrap()
}

// ===== Helper (stub implementation) =====
fn get_s3_extension_bucket_host(_id: &str) -> &'static str {
	"your-bucket.amazonaws.com"
}