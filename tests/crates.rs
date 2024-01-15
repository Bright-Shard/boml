/// Tests boml by parsing `Cargo.toml` files from well-known crates.

const SYN_URL: &str = "https://raw.githubusercontent.com/dtolnay/syn/98a90d70105f9b43f08eba091d6df1ec490a56e9/Cargo.toml";

use boml::{Error, Toml};

// "well-known crates" hehehe
#[test]
fn boml() {
	let source = include_str!("../Cargo.toml");
	let toml = parse(source);

	let package = toml.get_table("package").unwrap();
	assert_eq!(package.get_string("name").unwrap(), "boml");
	assert_eq!(package.get_string("edition").unwrap(), "2021");

	let dev_deps = toml.get_table("dev-dependencies").unwrap();
	let reqwest = dev_deps.get_table("reqwest").unwrap();
	assert_eq!(reqwest.get_string("version").unwrap(), "0.11");
	let features = reqwest.get_array("features").unwrap();
	assert_eq!(features.len(), 1);
	assert_eq!(features.first().unwrap().string().unwrap(), "blocking");
}

#[test]
fn syn() {
	let source = reqwest::blocking::get(SYN_URL).unwrap().text().unwrap();
	let toml = parse(&source);

	let package = toml.get_table("package").unwrap();
	assert_eq!(package.get_string("name").unwrap(), "syn");
	assert_eq!(package.get_string("version").unwrap(), "2.0.48");

	let features = toml.get_table("features").unwrap();
	let default_features = features.get_array("default").unwrap();
	let default_features: Vec<&str> = default_features
		.iter()
		.map(|feature| feature.string().unwrap())
		.collect();
	assert_eq!(
		default_features,
		vec!["derive", "parsing", "printing", "clone-impls", "proc-macro"]
	);

	let dev_deps = toml.get_table("dev-dependencies").unwrap();
	let reqwest = dev_deps.get_table("reqwest").unwrap();
	assert_eq!(reqwest.get_string("version").unwrap(), "0.11");
	let features = reqwest.get_array("features").unwrap();
	assert_eq!(features.len(), 1);
	assert_eq!(features.first().unwrap().string().unwrap(), "blocking");

	let benches = toml.get_array("bench").unwrap();
	assert_eq!(benches.len(), 2);
}

/// Wraps around `Toml::parse` and provides prettier error messages.
fn parse(source: &str) -> Toml<'_> {
	match Toml::parse(source) {
		Ok(toml) => toml,
		Err(error) => {
			let Error { start, end, kind } = error;

			let more_ctx_start = if start > 15 { start - 15 } else { 0 };
			let more_ctx_end = if source.len() - 16 > end {
				end + 15
			} else {
				source.len() - 1
			};

			panic!(
				"Error at characters {start}..{end} (`{}`): {kind:?}\nMore context: `{}`",
				&source[start..=end],
				&source[more_ctx_start..=more_ctx_end]
			);
		}
	}
}
