# mail_talker
An llm pipeline to respond to known email patterns, and request/evaluate assignments of candidates if they apply for a position.

In spirit of the codebase, this README.md is mostly LLM-generated, so don't bother reading.

## Design decisions
As it's a proof of concept, this has absolute minimum of cases considered. Stability of the performance can be further improved by about 1.7x, via introduction of few-shot. Very very easy to do; but once implemented, improvements by other means are capped /* open an issue if this needs explaining */.

For functionality could add a step for shaping eval based on the open positions, but codebase is getting a bit large for a proof of concept.

I could also do the resume parsing here, but better to first ensure the existing implementation is stable enough, as we would be stacking llm errors otherwise.


## Features

- **Email Parsing and Response Generation:** Automatically parses incoming emails to understand the context and generates appropriate responses based on predefined scenarios.

- **Repository Content Extraction:** Fetches and evaluates the content of public repositories submitted by candidates, using specific criteria to assess the suitability of the candidate for a position.

- **Case-Based Decision Making:** Decides on actions to take based on a set of predefined cases, each associated with specific situations and instructions.

- **Automated Repository Cloning and Content Analysis:** Clones public repositories, filters irrelevant files using regex patterns, and analyzes the remaining files to assist in the evaluation process.

- **Language Model Integration:** Uses a language model to evaluate candidates' submissions and generate email responses, ensuring contextually appropriate and coherent communication.

- **Flexible and Extensible:** Easily extendable to include more cases or adapt to different evaluation criteria, making it suitable for various domains beyond candidate evaluation.

## Requirements

- Rust programming environment
- `git` for repository cloning
- `rg` (ripgrep) for efficient file searching within cloned repositories
- Google API credentials for Gmail integration (refer to `listen.py` for Gmail API integration)
- Language model API access and credentials (see `v_utils::llm::Model` for model configurations)

## Installation

1. Ensure you have Rust and Cargo installed on your system.
2. Clone the mail_talker repository to your local machine.
3. Navigate to the cloned repository's root directory.
4. Build the project using Cargo: `cargo build --release`
5. Set up your Google API credentials as per the instructions in `src/listen.py`.
6. Configure language model API credentials and update the relevant sections in `src/llm/`.

## Usage

- Run the utility using `cargo run` from the project root. The utility will start listening for incoming emails and respond based on the configured cases and evaluation criteria.

- To add new cases or modify existing ones, update the `Cases` struct in `src/llm/case.rs`.

- To adjust the file filtering criteria for repository analysis, modify the `IGNORE_PATTERNS` in `src/parse.rs`.


## License
This project is available under the [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) licenses, at your choice.

## Contributing
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
