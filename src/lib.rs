pub mod tokenizer;

use tokenizer::{Tokenizer, Token};

pub fn tokenize(source: String) -> Result<Vec<Token>, String> {
	let mut tokenizer = Tokenizer::new(source);
	tokenizer.tokenize()
}