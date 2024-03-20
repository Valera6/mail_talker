use crate::llm::TaskSpec;
use crate::parse::FileContents;
use crate::{MODEL, TMP_DIR};
use anyhow::{anyhow, Result};
use std::io::Write;
use tracing::info;
use v_utils::llm;

#[derive(Debug, Clone, PartialEq)]
pub struct EvalMetric {
	pub key: &'static str,
	pub specification: &'static str,
	pub value: Option<f64>,
}
impl EvalMetric {
	pub fn new(key: &'static str, specification: &'static str, value: Option<f64>) -> Self {
		Self { key, specification, value }
	}
}
impl std::fmt::Display for EvalMetric {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\"{}\": {}", self.key, self.specification)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalMetrics(pub Vec<EvalMetric>);
impl EvalMetrics {
	pub fn update(&mut self, key: &str, new_value: f64) -> Result<()> {
		self.0.iter_mut().find(|m| m.key == key).ok_or_else(|| anyhow!("Key not found"))?.value = Some(new_value);
		Ok(())
	}

	pub fn from_file_contents(file_contents: &Vec<FileContents>) -> EvalMetrics {
		let n_lines: usize = file_contents.iter().map(|f| f.contents.lines().count()).sum();

		let mut metrics = vec![
			EvalMetric::new("functionality", "`5/10` if exactly what was required", None),
			EvalMetric::new("difficulty_of_language", "python is `0/10` -> assembly `10/10`", None),
		];

		let n_lines_metric = ((n_lines as f64 - 100.0) / 45.0).min(10.0); // ballpark estimate of effort. Currently 500 lines of pure code is 10/10, but we could increase that
		metrics.push(EvalMetric::new("n_lines", "AUTO", Some(n_lines_metric)));

		if n_lines > 250 {
			let conditional = vec![
				EvalMetric::new("reliability", "how reliable is the code", None),
				EvalMetric::new("maintainability", "how mantainable is the code", None),
			];
			metrics.extend(conditional);
		}

		Self(metrics)
	}
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
pub fn evaluate(task_spec: &TaskSpec, eval_metrics: &EvalMetrics, file_contents: &Vec<FileContents>) -> Result<Evaluation> {
	let task = &task_spec.task;
	let position = &task_spec.position;
	let temp_cache_file = TMP_DIR.join("eval.md");
	let mut file = std::fs::File::create(&temp_cache_file).expect(&format!("Failed to create or open file at {:?}", temp_cache_file));

	let mut message = format!(
		"Give objective evaluation of the performance of a candidate for a position of a programmer, on the scale of 1-10 for each of the following metrics: [{}]",
		eval_metrics.0.iter().filter(|m| m.value.is_none()).map(|m| m.key).collect::<Vec<&str>>().join(", ") // is_none() check, because some fields are already initialized
	);
	let task: String = task.chars().next().unwrap().to_lowercase().to_string() + &task[1..]; // for natural integration into the sentence
	message.push_str(&format!("\nThe assignment was to {}.", task));

	message.push_str("\nHere are all the files with actual code from submitted repo:");
	for c in file_contents {
		message.push_str(&format!("\n\n{}:\n````{}\n{}\n````", c.filename, c.extension(), c.contents));
	}

	message.push_str(&format!(
		r#"

After evaluating the candidate's performance on the provided metrics, return as json codeblock evaluations of each metric, and then whether the candidate is suitable for the position of {}:
```json
{{
	{}
	"is_suitable": bool
}}
```
Only return the ```json``` codeblock in the very end, _after_ having had evaluated each metric individually
If any of the creterions defines its evaluation scale, IGNORE THE CONVENTION and follow it exactly."#,
		position.to_lowercase(),
		eval_metrics.0.iter().filter(|m| m.value.is_none()).map(|m| m.to_string()).collect::<Vec<String>>().join(",\n") // is_none() check, because some fields are already initialized
	));

	file.write_all(message.as_bytes()).expect("Failed to write message to file");

	let response = llm::oneshot(&message, MODEL.clone())?;
	let response_str = response.extract_codeblock("json")?;
	let requested_json: serde_json::Value = serde_json::from_str(&response_str)?;
	let obj = requested_json
		.as_object()
		.cloned()
		.ok_or_else(|| anyhow!("Returned json is not an object:\n{response_str}"))?;

	let log_entry = format!(
		"\n\n# -----------------------------------------------------------------------------\n\n## Response\n{}",
		response
	);
	file.write_all(log_entry.as_bytes()).expect("Failed to write response to file");

	fn deserialize_rating(rating: &str) -> Result<f64> {
		if let Ok(rating) = rating.parse::<f64>() {
			Ok(rating)
		} else {
			let split = rating.split('/').collect::<Vec<&str>>();
			if split.len() != 2 {
				return Err(anyhow!("Invalid rating format: {}", rating));
			}
			let numerator = split[0].trim().parse::<f64>()?;
			let denominator = split[1].trim().parse::<f64>()?;
			match denominator == 10.0 {
				true => Ok(numerator),
				false => Err(anyhow!("Invalid rating format: {}", rating)),
			}
		}
	}

	let evaluation = {
		let mut is_suitable: Option<bool> = None;
		let mut updated_eval = eval_metrics.clone();
		for (key, value) in obj.iter() {
			match key {
				_ if key == "is_suitable" => {
					is_suitable = Some(
						value
							.as_bool()
							.ok_or_else(|| anyhow!("llm returned a non-boolean value for {key}\nFull JSON: {response_str}"))?,
					);
				}
				_ => {
					if let Some(n) = value.as_f64() {
						updated_eval.update(key, n)?;
					} else {
						let rating_str = value.as_str().ok_or_else(|| anyhow!("Expected a string for {key}, but found: {:?}", value))?;
						let value = deserialize_rating(rating_str)
							.or_else(|e| Err(anyhow!("Error deserializing rating for {key}: {e}\nFull JSON: {response_str}")))?;
						updated_eval.update(key, value)?;
					}
				}
			}
		}

		let decision = is_suitable.ok_or_else(|| anyhow!("llm did not return `is_suitable` field\nFull JSON: {response_str}"))?;

		let eval_values: Vec<f64> = updated_eval
			.0
			.iter()
			.map(|m| {
				m.value.ok_or_else(|| {
					anyhow!(
						"LLM did not return values for all of the requested evaluation metrics.\nMetrics: {:?}\nFull JSON: {}",
						eval_metrics,
						response_str
					)
				})
			})
			.collect::<Result<Vec<_>, _>>()?;
		if eval_values.is_empty() {
			return Err(anyhow!(
				"LLM did not return values for all of the requested evaluation metrics.\nMetrics: {:?}\nFull JSON: {}",
				eval_metrics,
				response_str
			));
		}

		info!(?eval_values);

		let mean_score: f64 = eval_values.iter().sum::<f64>() / eval_values.len() as f64;
		info!(mean_score, decision);

		Evaluation {
			mean_score,
			decision,
			other: response.to_string(),
		}
	};

	let log_entry = format!(
		"\n\n# -----------------------------------------------------------------------------\n\n## Evaluation\n{}",
		evaluation
	);
	file.write_all(log_entry.as_bytes()).expect("Failed to write evaluation to file");

	Ok(evaluation)
}
