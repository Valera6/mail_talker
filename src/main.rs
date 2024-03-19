use llm::{Case, Cases};
use std::path::{Path, PathBuf};
use v_utils::llm::Model;

mod llm;
pub mod mail;
pub mod parse;

lazy_static::lazy_static! {
	pub static ref TMP_DIR: PathBuf = Path::new("/tmp/mail_talker_cache/").to_path_buf();
	pub static ref MODEL: Model = Model::Medium;
}

// could put the tasks up on hackmd.io
// Would need a tiny little parser then

// to test, let's make tests that take in a conversation and roll through the entirety of it, printing to stderr
// Then feed my 3 threads with them to it

fn main() {
	std::fs::create_dir_all(TMP_DIR.clone()).unwrap();
	tracing_subscriber::fmt::init();

	let interaction = vec![
		mail::Message {
			content: "I am a trader, with rust and python knowledge. Would love to work with you in any way shape or form.".to_string(),
			sender: "them".to_string(),
		},
		mail::Message {
			content: "I'm assuming you want a quant position. First show us what you got: create a parser of exchange documentations, to detect unannounced changes. Target time: 4 hours".to_string(),
			sender: "us".to_string(),
		},
		mail::Message {
			content: "Get me in!\n\nhttps://github.com/Valera6/doc_scraper.git".to_string(),
			sender: "them".to_string(),
		},
	];

	let cases = Cases(vec![
		Case::new("position", "candidate wants a position", "Tell him he can't have it"),
		Case::new(
			"update",
			"candidate asking about result of interview / take-home submission",
			"tell him to wait",
		),
		Case::new("next_steps", "candidate asking about next steps", "tell him to wait"),
	]);

	let case = match interaction.last().unwrap().content.contains("https://github.com/") {
		true => Some({
			let link = interaction
				.last()
				.unwrap()
				.content
				.split_whitespace()
				.find(|s| s.contains("https://github.com/"))
				.unwrap();
			let contents = parse::extract(link);
			let task_spec = llm::extract_task_spec(&interaction).unwrap();

			let eval_metrics = vec![
				"reliability (ignore completely if small number of lines)",
				"maintainability (ignore completely if small number of lines)",
				"interface",
				"effort (100 lines is `0/10` ->  1000 lines `10/10`)",
				"language used (python is `0/10` -> assembly `10/10`)",
			];

			let evalutaiton: llm::Evaluation = llm::evaluate(&task_spec, &eval_metrics, &contents).unwrap(); //task is probably in the last message we sent ourselves. I guess could just ask llm.
			let action = match evalutaiton.decision {
				true => "invite them for an interview; link is: https://calendly.com/valera6/interview",
				false => "tell them they are sadly not suitable for the position",
			};
			Case::new("auto_eval", "AUTO-DETECTED", action)
		}),
		false => llm::determine_case(&interaction, &cases).unwrap(),
	};

	let answer = match case {
		Some(c) => Some(llm::compose(&interaction, &c).unwrap()),
		None => None,
	};
	dbg!(&answer);
}

//TODO: alongside the LLM evaluation, would be great to staticly count the following:
// - number of symbols against the average on the task

//fn main() {
//	std::fs::create_dir_all(TMP_DIR).unwrap();
//
//	//let contents = parse::extract("https://github.com/Valera6/doc_scraper.git");
//	//let task = "create a parser of exchange documentations, to detect unannounced changes. Target time: 4 hours";
//	//let position = "junior";
//
//	//let contents = parse::extract("https://github.com/Valera6/auto_eval.git");
//	//let task = "creata a way to auto-evaluate the performance of a candidate for a position of a programmer";
//	//let position = "senior software engineer";
//
//	//let contents = parse::extract("https://github.com/Valera6/monopoly.git");
//	//let task = "create a monopoly game";
//	//let position = "senior software engineer";
//
//	let contents = parse::extract("https://github.com/lunarcon/pyfractals");
//	let task = "create a fractal generator";
//	let position = "senior software engineer";
