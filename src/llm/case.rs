use crate::mail;
use anyhow::{anyhow, Result};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use v_utils::llm;

#[derive(Debug, Clone)]
pub struct Case {
	pub key: &'static str,
	pub situation: &'static str,
	pub instruction: &'static str,
}
impl Case {
	pub fn new(key: &'static str, situation: &'static str, instruction: &'static str) -> Self {
		Self { key, situation, instruction }
	}
}

pub struct Cases(pub Vec<Case>);

impl Cases {
	fn by_key<T: AsRef<str>>(&self, key: T) -> Option<&Case> {
		self.0.iter().find(|c| c.key == key.as_ref())
	}
}

impl Serialize for Cases {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut seq = serializer.serialize_struct("Cases", self.0.len())?;
		for case in &self.0 {
			seq.serialize_field(case.key, case.situation)?;
		}
		seq.end()
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct RequestedJson {
	pub case: Option<String>,
}

pub fn determine_case(interaction: &Vec<mail::Message>, cases: &Cases) -> Result<Option<Case>> {
	let response = llm::oneshot(
		format!(r#"We just received a new message on our company email. The following is the entirety of the conversation up to this point:  ```{}
```

Your job is to decide whether this is one of the outlined special cases and we can pass it onto our system of auto-answering. Here are all of the special cases: ```json
{}
```

You return a json code-block like ```json
{{
	"case": String
}}
```
Where "case" is the key of the situation or null if none obviously match"#,
			serde_json::to_string(&interaction).unwrap(),
			serde_json::to_string(&cases).unwrap()
		),
		llm::Model::Fast,
	)
	.unwrap();
	let r: RequestedJson = serde_json::from_str(&response.extract_codeblock("json")?)?;
	Ok(match r.case {
		Some(s) => Some(cases.by_key(s).ok_or_else(|| anyhow!("Llm returned a non-existent key"))?.clone()),
		None => None,
	})
}
