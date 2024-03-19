use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message {
	pub contents: String,
	pub sender: String,
}

// Here should be the mail api integration, but google apis are dumb.
