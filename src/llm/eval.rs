use crate::llm::TaskSpec;
use crate::parse::FileContents;
use crate::{MODEL, TMP_DIR};
use anyhow::Result;
use serde::Deserialize;
use std::io::Write;
use tracing::{info, instrument};
use v_utils::llm;

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
pub fn evaluate(task_spec: &TaskSpec, eval_metrics: &Vec<&str>, file_contents: &Vec<FileContents>) -> Result<Evaluation> {
	let task = &task_spec.task;
	let position = &task_spec.position;
	let temp_cache_file = TMP_DIR.join("eval.md");
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

	let response = llm::oneshot(&message, MODEL.clone())?;
	let response_str = response.extract_codeblock("json")?;
	let requested_json: RequestedJson = serde_json::from_str(&response_str)?;

	let log_entry = format!(
		"\n\n# -----------------------------------------------------------------------------\n\n## Response\n{}",
		response
	);
	file.write_all(log_entry.as_bytes()).expect("Failed to write response to file");

	let evaluation = Evaluation {
		mean_score: requested_json.mean_score,
		decision: requested_json.is_suitable,
		other: response.to_string(),
	};

	let log_entry = format!(
		"\n\n# -----------------------------------------------------------------------------\n\n## Evaluation\n{}",
		evaluation
	);
	file.write_all(log_entry.as_bytes()).expect("Failed to write evaluation to file");

	info!(mean_score = requested_json.mean_score, is_suitable = requested_json.is_suitable);

	Ok(evaluation)
}
