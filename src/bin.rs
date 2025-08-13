#![feature(duration_millis_float)]

use lox_rs::tokenize;

fn main() {
	// let source = if let Ok(data) = std::fs::read_to_string("src/test.lox") {
	// 	data
	// } else if let Ok(data) = std::fs::read_to_string("test.lox") {
	// 	data
	// } else {
	// 	panic!("No test.lox file found")
	// };

	let mut source = String::new();
	for _ in 1..=1_000_000 {
		source.push_str("print \"Hello, World!\";\n");
	}

	let start = std::time::SystemTime::now();

	let tokens = match tokenize(source) {
		Ok(res) => res,
		Err(e) => {
			println!("{e}");
			panic!();
		}
	};
	println!("Tokens: {}\nTime taken: {}ms", tokens.len(), std::time::SystemTime::now().duration_since(start).unwrap().as_millis_f64());
}
