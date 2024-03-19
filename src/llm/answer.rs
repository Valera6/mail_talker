use serde::Deserialize;
use crate::llm::Case;
use crate::mail;
use anyhow::Result;
use v_utils::llm;

pub fn compose(interaction: &Vec<mail::Message>, case: &Case) -> Result<String> {
	let mut chars = case.instruction.chars();
	let instruction = chars.next().unwrap().to_uppercase().chain(chars).collect::<String>();
	let llm_response = llm::oneshot(
		format!(
			r#"Write a response to the the email our company just received. For reference, the entire conversation up to this point:  ```{}
```

{}.
Return response inside triple bacticks, like ```{{text}}```. Sign with:
`best,
Valera`
Do not add a header.
"#,
			serde_json::to_string(&interaction).unwrap(),
			instruction
		),
		llm::Model::Fast,
	)?;
	let mail_text = llm_response.extract_codeblock("")?;
	Ok(mail_text)
}

#[derive(Debug, Deserialize)]
pub struct TaskSpec {
	pub task: String,
	pub position: String,
}

/// Extracts (task, position) from the conversation, to pass to the evaluator
pub fn extract_task_spec(interaction: &Vec<mail::Message>) -> Result<TaskSpec> {
	let q = format!(r#"Parse the following email conversation, and extract exact (task, position) that were given to the candidate. ```{}
```

Return extracted task and position as a json codeblock. Like this: ```json
{{
	"task": String,
	"position": String
}}
```"#, serde_json::to_string(&interaction).unwrap());
	let llm_response = llm::oneshot(&q, llm::Model::Fast)?;
	let task_spec = llm_response.extract_codeblock("json")?;
	let task_spec: TaskSpec = serde_json::from_str(&task_spec)?;
	Ok(task_spec)
}
