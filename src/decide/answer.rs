use crate::decide::Case;
use crate::mail;
use anyhow::Result;
use v_utils::llm;

//TODO: If the last message contains a github link, we call auto_eval
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
	dbg!(&llm_response);
	let mail_text = llm_response.extract_codeblock("")?;
	Ok(mail_text)
}
