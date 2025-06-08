use serde::Serialize;
use serde_json::json;
use validator::{Validate};
use chrono::{Utc, TimeZone, Datelike};
use std::collections::HashMap;

// Helper: get days since Jan 1, 2007
fn get_elapsed_days() -> i64 {
	let start_date = Utc.ymd(2007, 1, 1);
	let now = Utc::now();
	(now - start_date.and_hms(0,0,0)).num_days()
}

// Helper: normalize size as in Go code
fn normalize_size(size: u64) -> u64 {
	if size == 0 { 1 } else { size }
}

#[derive(Debug, Clone, Serialize)]
struct URL {
	url: String,
}

#[derive(Debug, Clone, Serialize, Validate)]
struct Out {
	#[validate(length(min = 1))]
	sha256: String,
}

#[derive(Debug, Clone, Serialize, Validate)]
struct In {
	#[validate(length(min = 1))]
	sha256: String,
}

#[derive(Debug, Clone, Serialize, Validate)]
struct Operation {
	#[validate(length(min = 1))]
	#[serde(rename = "type")]
	typ: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	out: Option<Out>,
	#[serde(skip_serializing_if = "Option::is_none")]
	r#in: Option<In>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	urls: Vec<URL>,
	#[serde(skip_serializing_if = "Option::is_none")]
	previous: Option<In>,
	#[serde(skip_serializing_if = "Option::is_none")]
	size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct Pipeline {
	pipeline_id: String,
	operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateCheck {
	status: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	nextversion: String,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone, Serialize)]
struct DayStart {
	elapsed_days: i64,
}

#[derive(Debug, Clone, Serialize)]
struct App {
	appid: String,
	status: String,
	updatecheck: UpdateCheck,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseWrapper {
	protocol: String,
	daystart: DayStart,
	apps: Vec<App>,
}

#[derive(Debug, Clone, Serialize)]
struct JSONResponse {
	response: ResponseWrapper,
}

// Placeholder for extension::Extension PatchInfo and actual Extension
#[derive(Debug, Clone)]
pub struct PatchInfo {
	pub hashdiff: String,
	pub sizediff: u64,
}

// Please adapt this struct according to your real Extension type
#[derive(Debug, Clone)]
pub struct Extension {
	pub id: String,
	pub sha256: String,
	pub version: String,
	pub fp: String,
	pub size: u64,
	pub status: String,
	pub patch_list: Option<HashMap<String, PatchInfo>>,
}

pub struct UpdateResponse(pub Vec<Extension>);

// Determines update status
fn get_update_status(extension: &Extension) -> &str {
	if extension.status == "noupdate" {
		"noupdate"
	} else {
		"ok"
	}
}

// Helper to generate S3 Host (placeholder: adapt this to your logic)
fn get_s3_extension_bucket_host(app_id: &str) -> String {
	format!("s3.example.com/{}", app_id)
}

impl UpdateResponse {
	pub fn to_json(&self) -> Result<Vec<u8>, String> {
		let elapsed_days = get_elapsed_days();
		let mut response = ResponseWrapper {
			protocol: "4.0".to_string(),
			daystart: DayStart { elapsed_days },
			apps: Vec::new(),
		};

		for ext in &self.0 {
			if ext.sha256.is_empty() {
				return Err(format!("extension {} has empty SHA256", ext.id));
			}

			let update_status = get_update_status(ext).to_string();

			let mut update_check = UpdateCheck {
				status: update_status.clone(),
				nextversion: String::new(),
				pipelines: Vec::new(),
			};

			if update_status == "ok" {
				update_check.nextversion = ext.version.clone();

				// Direct URL generation (adapt host logic as needed)
				let extension_name = format!("extension_{}.crx", ext.version.replace('.', "_"));
				let url = format!(
					"https://{}/release/{}/{}",
					get_s3_extension_bucket_host(&ext.id),
					&ext.id,
					extension_name,
				);

				// If patch available, add diff pipeline
				if !ext.fp.is_empty() {
					if let Some(ref patch_list) = ext.patch_list {
						if let Some(patch) = patch_list.get(&ext.fp) {
							if patch.hashdiff.is_empty() {
								return Err(format!("extension {} has empty Hashdiff", ext.id));
							}
							let fp_prefix = &ext.fp[..ext.fp.len().min(8)];
							let diff_pipeline_id = format!("puff_diff_{}", fp_prefix);
							let patch_url = format!(
								"https://{}/release/{}/patches/{}/{}.puff",
								get_s3_extension_bucket_host(&ext.id),
								&ext.id,
								ext.sha256,
								ext.fp
							);

							// Construct diff pipeline
							let diff_download_op = Operation {
								typ: "download".to_string(),
								out: Some(Out { sha256: patch.hashdiff.clone() }),
								r#in: None,
								urls: vec![URL { url: patch_url }],
								previous: None,
								size: Some(normalize_size(patch.sizediff)),
							};
							let puff_op = Operation {
								typ: "puff".to_string(),
								out: None,
								r#in: None,
								urls: vec![],
								previous: Some(In { sha256: ext.fp.clone() }),
								size: None,
							};
							let crx3_op = Operation {
								typ: "crx3".to_string(),
								out: None,
								r#in: Some(In { sha256: ext.sha256.clone() }),
								urls: vec![],
								previous: None,
								size: None,
							};

							// Validate before pushing
							for op in [&diff_download_op, &puff_op, &crx3_op] {
								if let Err(e) = op.validate() {
									return Err(format!("{} operation validation failed for extension {}: {:?}", op.typ, ext.id, e));
								}
							}

							let diff_pipeline = Pipeline {
								pipeline_id: diff_pipeline_id,
								operations: vec![
									diff_download_op,
									puff_op,
									crx3_op,
								],
							};
							update_check.pipelines.push(diff_pipeline);
						}
					}
				}

				// Add the direct full pipeline always last
				let out = Out { sha256: ext.sha256.clone() };
				let urls = vec![URL { url }];
				let main_download_op = Operation {
					typ: "download".to_string(),
					out: Some(out),
					r#in: None,
					urls,
					previous: None,
					size: Some(normalize_size(ext.size)),
				};
				let main_crx3_op = Operation {
					typ: "crx3".to_string(),
					out: None,
					r#in: Some(In { sha256: ext.sha256.clone() }),
					urls: vec![],
					previous: None,
					size: None,
				};

				for op in [&main_download_op, &main_crx3_op] {
					if let Err(e) = op.validate() {
						return Err(format!("{} operation validation failed for extension {}: {:?}", op.typ, ext.id, e));
					}
				}

				let main_pipeline = Pipeline {
					pipeline_id: "direct_full".to_string(),
					operations: vec![
						main_download_op,
						main_crx3_op,
					],
				};
				update_check.pipelines.push(main_pipeline);
			}

			let app = App {
				appid: ext.id.clone(),
				status: "ok".to_string(),
				updatecheck: update_check,
			};
			response.apps.push(app);
		}

		let json_response = JSONResponse { response };
		serde_json::to_vec(&json_response)
			.map_err(|e| format!("serialization error: {}", e))
	}
}