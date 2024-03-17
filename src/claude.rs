use crate::Message;
use crate::TMP_DIR;
use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use std::io::Write;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Model {
	Fast,
	Medium,
	Slow,
	Haiku,
	Sonnet,
	Opus,
}
pub struct Cost {
	pub million_input_tokens: f32,
	pub million_output_tokens: f32,
}

impl Model {
	fn to_str(&self) -> &str {
		match self {
			Model::Fast => "claude-3-haiku-20240307",
			Model::Medium => "claude-3-sonnet-20240229",
			Model::Slow => "claude-3-opus-20240229",
			Model::Haiku => Model::Fast.to_str(),
			Model::Sonnet => Model::Medium.to_str(),
			Model::Opus => Model::Slow.to_str(),
		}
	}

	fn from_str(s: &str) -> Model {
		match s {
			_ if s.contains("haiku") => Model::Fast,
			_ if s.contains("sonnet") => Model::Medium,
			_ if s.contains("opus") => Model::Slow,
			_ => panic!("Unknown model: {}", s),
		}
	}

	pub fn cost(&self) -> Cost {
		match self {
			Model::Fast | Model::Haiku => Cost {
				million_input_tokens: 0.25,
				million_output_tokens: 1.25,
			},
			Model::Medium | Model::Sonnet => Cost {
				million_input_tokens: 3.0,
				million_output_tokens: 15.0,
			},
			Model::Slow | Model::Opus => Cost {
				million_input_tokens: 15.0,
				million_output_tokens: 75.0,
			},
		}
	}
}

///docs: https://docs.anthropic.com/claude/reference/messages_post
pub fn ask_claude(message: &str, model: Model) -> Response {
	let api_key = std::env::var("CLAUDE_TOKEN").expect("CLAUDE_TOKEN environment variable not set");
	let url = "https://api.anthropic.com/v1/messages";

	let mut headers = HeaderMap::new();
	headers.insert("x-api-key", HeaderValue::from_str(&api_key).unwrap());
	headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

	let payload = json!({
		"model": model.to_str(),
		"max_tokens": 1024,
		"messages": [
			{
				"role": "user",
				"content": message
			}
		]
	});

	let client = Client::new();
	let response = client.post(url).headers(headers).json(&payload).send().expect("Failed to send request");

	let response_raw = response.text().expect("Failed to read response body");
	let response: Response = serde_json::from_str(&response_raw).expect("Failed to parse response body");
	response
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Response {
	id: String,
	#[serde(rename = "type")]
	response_type: String,
	role: String,
	content: Vec<Content>,
	model: String,
	stop_reason: String,
	stop_sequence: Option<String>,
	usage: Usage,
}
impl Response {
	pub fn text(&self) -> String {
		self.content[0].text.clone()
	}

	pub fn cost_cents(&self) -> f32 {
		let model = Model::from_str(&self.model);
		let cost = model.cost();
		(self.usage.input_tokens as f32 * cost.million_input_tokens + self.usage.output_tokens as f32 * cost.million_output_tokens) / 10_000.0
	}

	pub fn extract_codeblock(&self, extension: &str) -> Option<String> {
		let text = self.text();
		let codeblock = text.split("```").find(|s| s.starts_with(extension))?;
		Some(codeblock.strip_prefix(extension).unwrap().to_string())
	}
}
impl std::fmt::Display for Response {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Response: {:#?}\nCost (cents): {}", self.text(), self.cost_cents())
	}
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Content {
	#[serde(rename = "type")]
	content_type: String,
	text: String,
}
#[derive(Deserialize, Debug)]
struct Usage {
	input_tokens: u32,
	output_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct RequestedJson {
	mean_score: f64,
	is_suitable: bool,
}

#[derive(Debug)]
pub struct Evaluation {
	pub mean_score: f64,
	pub decision: bool,
	pub other: String,
}
impl std::fmt::Display for Evaluation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\nMean Score: {}\nDecision: {}\nOther:\n{}", self.mean_score, self.decision, self.other)
	}
}
pub fn read_mail(interaction: Vec<Message>, model: Model) -> Result<Evaluation> {
	let temp_cache_file = std::path::Path::new(TMP_DIR).join("message_cache.md");
	let mut file = std::fs::File::create(&temp_cache_file).expect(&format!("Failed to create or open file at {:?}", temp_cache_file));

	let mut message = format!(
		"Give objective evaluation of the performance of a candidate for a position of a programmer, on the scale of 1-10 for each of the following metrics: [{}]",
		eval_metrics.join(", ")
	);
	let task: String = task.chars().next().unwrap().to_lowercase().to_string() + &task[1..]; // cast .lowercase() on the first letter of the task for natural integration into the sentence
	message.push_str(&format!("\nThe assignment was to {}.", task));

	message.push_str("\nHere are all the files with actual code from submitted repo:");
	for c in file_contents {
		message.push_str(&format!("\n\n{}:\n````{}\n{}\n````", c.filename, c.extension(), c.contents));
	}

	message.push_str(&format!(
		r#"

After evaluating the candidate's performance on the provided metrics, is the candidate suitable for the position of a {}? Return as json:
```json
{{
	"mean_score": float,
	"is_suitable": bool
}}
```
Only return the ```json``` codeblock in the very end, _after_ having had evaluated each metric individually"#,
		position.to_lowercase()
	));
	message.push_str(&format!(
		"\nIf any of the creterions defines its evaluation scale, IGNORE THE CONVENTION and follow it exactly: [{}]",
		eval_metrics.join(", ")
	));

	file.write_all(message.as_bytes()).expect("Failed to write message to file");

	let response = ask_claude(&message, model);
	let response_str = response
		.extract_codeblock("json")
		.expect(&format!("Response did not contain a ```json block:\nResponse: {}", response));
	let requested_json: RequestedJson = serde_json::from_str(&response_str)?;

	Ok(Evaluation {
		mean_score: requested_json.mean_score,
		decision: requested_json.is_suitable,
		other: response.to_string(),
	})
}
