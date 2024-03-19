use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message {
	pub content: String,
	pub sender: String,
}

// Here should be the mail api integration, but google apis are dumb.
