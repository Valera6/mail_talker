#![allow(dead_code)]
use crate::TMP_DIR;
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, anyhow};
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
pub fn extract(repo_url: &str) -> Result<Vec<FileContents>> {
	let repo_name = repo_url.split('/').last().ok_or_else(|| anyhow!("Invalid repository URL"))?;
	let repo_name = repo_name.strip_suffix(".git").unwrap_or(repo_name);
	let tmp_clone_dir = TMP_DIR.join(format!("repo_clone_{}", repo_name));

	std::fs::remove_dir_all(&tmp_clone_dir).ok();
	let clone_output = Command::new("git")
		.arg("clone")
		.arg("--depth=1")
		.arg(repo_url)
		.arg(&tmp_clone_dir)
		.output()?;

	if !clone_output.status.success() {
		return Err(anyhow::anyhow!("Failed to clone repository:\n{:?}", clone_output));
	}

	std::env::set_current_dir(&tmp_clone_dir).expect("Failed to change directory");

	let rg_output = Command::new("rg")
        .arg("--files")
        .output()
        .map_err(|_| anyhow!("Failed to execute rg. Ensure ripgrep is installed."))?;
	if !rg_output.status.success() {
		return Err(anyhow!("Failed to list files in cloned repository with rg:\n{:?}", rg_output));
	}
	let files_newline_separated = String::from_utf8_lossy(&rg_output.stdout);
	let files: Vec<&str> = files_newline_separated
		.lines()
		.filter(|&file| !IGNORE_COMPILED.iter().any(|pattern| pattern.is_match(file)))
		.collect();

	let important_files: Vec<FileContents> = files
		.into_iter()
		.filter_map(|filename| {
			std::fs::read_to_string(&tmp_clone_dir.join(filename))
				.ok()
				.map(|contents| FileContents::new(filename.to_string(), contents.strip_suffix('\n').unwrap_or_else(|| &contents).to_string()))
		})
		.collect();

	std::fs::remove_dir_all(tmp_clone_dir).ok();
	Ok(important_files)
}
