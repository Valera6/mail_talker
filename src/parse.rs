#![allow(dead_code)]
use crate::TMP_DIR;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::process::Command;

lazy_static! {
	static ref IGNORE_PATTERNS: Vec<&'static str> = vec![
		r"\.mod$",
		r"\.sum$",
		r"\.toml$",
		r"\.json$",
		r"\.yaml$",
		r"\.yml$",
		r"\.lock$",
		"Dockerfile$",
		r"\.dockerignore$",
		r"\.travis\.yml$",
		r"\.gitlab-ci\.yml$",
		"Jenkinsfile$",
		r"\.env($|\.example$)",
		r"(\.vscode/|\.idea/|\.editorconfig$)",
		r"\.(jar|war|dll|exe|so|o|a)$",
		r"(node_modules/|vendor/|bower_components/)",
		r"\.(db|sql|sqlite|sqlite3)$",
		r"\.(png|jpg|jpeg|gif|svg|ico)$",
		r"\.(ttf|otf|woff|woff2)$",
		r"\.(css|scss|sass|less)$",
		r"\.(sh|bat)$",
		r"\.(zip|tar|tar\.gz|rar)$",
		r"\.(txt|pdf|doc|docx|ppt|pptx|xls|xlsx)$",
		r"^[^.]+$", // Ignore files without any extension too
	];
	static ref IGNORE_COMPILED: Vec<Regex> = IGNORE_PATTERNS.iter().map(|pattern| Regex::new(pattern).unwrap()).collect();
}

#[derive(Debug, Clone)]
pub struct FileContents {
	pub filename: String,
	pub contents: String,
}
impl FileContents {
	pub fn new(filename: String, contents: String) -> Self {
		Self { filename, contents }
	}

	pub fn extension(&self) -> String {
		self.filename.split('.').last().unwrap().to_string()
	}
}

/// Extracts contents of important files from a public repository
pub fn extract(repo_url: &str) -> Vec<FileContents> {
	let tmp_dir = TMP_DIR.join("repo_clone");

	std::fs::remove_dir_all(&tmp_dir).ok();
	let clone_output = Command::new("git")
		.arg("clone")
		.arg(repo_url)
		.arg(&tmp_dir)
		.output()
		.expect("Failed to clone repository");

	assert!(clone_output.status.success());

	std::env::set_current_dir(&tmp_dir).expect("Failed to change directory");

	let rg_output = Command::new("rg").arg("--files").output().expect("Failed to execute rg");
	assert!(rg_output.status.success());
	let files_newline_separated = String::from_utf8_lossy(&rg_output.stdout);
	let files: Vec<&str> = files_newline_separated
		.lines()
		.filter(|&file| !IGNORE_COMPILED.iter().any(|pattern| pattern.is_match(file)))
		.collect();

	let important_files: Vec<FileContents> = files
		.into_iter()
		.filter_map(|filename| {
			fs::read_to_string(&tmp_dir.join(filename))
				.ok()
				.map(|contents| FileContents::new(filename.to_string(), contents.strip_suffix('\n').unwrap_or_else(|| &contents).to_string()))
		})
		.collect();

	fs::remove_dir_all(tmp_dir).expect("Failed to remove cloned repository");
	important_files
}
