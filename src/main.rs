use decide::{Case, Cases};
use std::path::{Path, PathBuf};

mod decide;
pub mod mail;

lazy_static::lazy_static! {
	pub static ref TMP_DIR: PathBuf = Path::new("/tmp/mail_talker_cache/").to_path_buf();
}

// could put the tasks up on hackmd.io
// Would need a tiny little parser then

// to test, let's make tests that take in a conversation and roll through the entirety of it, printing to stderr
// Then feed my 3 threads with them to it

fn main() {
	std::fs::create_dir_all(TMP_DIR.clone()).unwrap();

	let interaction = vec![mail::Message {
		contents: "I want to be your CEO".to_string(),
		sender: "them".to_string(),
	}];

	let cases = Cases(vec![
		Case::new("position", "candidate wants a position", "Tell him he can't have it"),
		Case::new(
			"update",
			"candidate asking about result of interview / take-home submission",
			"tell him to wait",
		),
		Case::new("next_steps", "candidate asking about next steps", "tell him to wait"),
	]);

	let case = decide::determine_case(&interaction, &cases).unwrap();
	dbg!(&case);

	let answer = match case {
		Some(c) => Some(decide::compose(&interaction, &c).unwrap()),
		None => None,
	};
	dbg!(&answer);
}
