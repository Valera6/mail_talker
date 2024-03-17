use serde::Serialize;
use serde_json;
use v_utils::llm;

pub static TMP_DIR: &str = "/tmp/mai_reader_cache";

#[derive(Debug, Serialize)]
pub struct Message {
	pub contents: String,
	pub sender: String,
}

fn main() {
	let interaction = [
		Message {
			contents: "GET ME IN".to_string(),
			sender: "them".to_string(),
		},
		Message {
			contents: "What position do you want in on?".to_string(),
			sender: "us".to_string(),
		},
		Message {
			contents: "I want to be your CEO".to_string(),
			sender: "them".to_string(),
		},
	];

	let interaction_str = serde_json::to_string(&interaction).unwrap();

	let response = llm::oneshot(
		format!("From this email conversation, what does the candidate want: {}", interaction_str),
		llm::Model::Fast,
	)
	.unwrap();
	println!("{}", response);
}
