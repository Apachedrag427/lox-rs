#[derive(Debug, PartialEq)]
pub enum Token {
	LeftParen, RightParen, LeftBrace, RightBrace,
	Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

	Bang, BangEqual,
	Equal, EqualEqual,
	Greater, GreatEqual,
	Less, LessEqual,

	Identifier(String),
	String(String),
	Number(f64),

	And, Class, Else, False, Fun, For, If, Nil, Or,
	Print, Return, Super, This, True, Var, While,

	Eof
}

pub struct Tokenizer {
	source: String,
	offset: usize,
	tokens: Vec<Token>
}

static OPERATORS: &str = "!=><";

impl Tokenizer {
	pub fn new(source: impl Into<String>) -> Tokenizer {
		Tokenizer {
			source: source.into(),
			offset: 0,
			tokens: vec![]
		}
	}

	fn get_2d_location(&self, offset: usize) -> (usize, usize) {
		let bytes = self.source.as_bytes();
		let mut line: usize = 1;
		let mut column: usize = 1;
		let mut current_offset: usize = 0;

		while current_offset < offset {
			let c = bytes[current_offset] as char;

			if c == '\n' {
				line += 1;
				column = 1;
			}
			current_offset += 1;
			column += 1;
		}

		(line, column)
	}

	fn generate_report(&self, message: impl Into<String>, offset: usize) -> String {
		let location = self.get_2d_location(offset);
		format!("[{}:{}] Error: {}", location.0, location.1, message.into())
	}

	pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
		
		// Ensure the final token is properly processed
		// (Otherwise, if an identifier or number is the final token, it'll never be pushed to the result)
		self.source.push(' ');

		let bytes = self.source.as_bytes();

		let mut errors = vec![];

		let mut reading_string = false;
		let mut reading_number = false;
		let mut reading_identifier = false;
		let mut read_start_offset: usize = 0;
		let mut escape_next = false;
		let mut string_buf: Vec<char> = vec![];

		while self.offset < self.source.len() {
			let c = bytes[self.offset] as char;
			let current_offset = self.offset;
			self.offset += 1;

			if reading_string {
				if escape_next {
					string_buf.push(c);
					escape_next = false;
					continue;
				}

				if c == '"' {
					reading_string = false;
					self.tokens.push(Token::String(
						std::mem::take(&mut string_buf)
						.into_iter()
						.collect()
					));
					continue;
				}

				if c == '\\' {
					escape_next = true;
					continue;
				}

				string_buf.push(c);
				continue;
			}

			if reading_number {
				if c.is_numeric() || c == '.' {
					string_buf.push(c);
					continue;
				}

				reading_number = false;

				let num_string: String = std::mem::take(&mut string_buf)
					.into_iter()
					.collect();

				if let Ok(num) = num_string.parse::<f64>() {
					self.tokens.push(Token::Number(num));
				} else {
					errors.push(self.generate_report(format!("Invalid number '{}'", num_string), read_start_offset));
				}
			}

			if reading_identifier {
				if c.is_alphanumeric() {
					string_buf.push(c);
					continue;
				}

				reading_identifier = false;

				let iden: String = std::mem::take(&mut string_buf)
					.into_iter()
					.collect();

				self.tokens.push(match &iden[..] {
					"and" => Token::And,
					"class" => Token::Class,
					"else" => Token::Else,
					"false" => Token::False,
					"fun" => Token::Fun,
					"for" => Token::For,
					"if" => Token::If,
					"nil" => Token::Nil,
					"or" => Token::Or,
					"print" => Token::Print,
					"return" => Token::Return,
					"super" => Token::Super,
					"this" => Token::This,
					"true" => Token::True,
					"var" => Token::Var,
					"while" => Token::While,
					_ => Token::Identifier(iden)
				});
			}

			if c.is_whitespace() {
				continue;
			}

			match c {
				'(' => self.tokens.push(Token::LeftParen),
				')' => self.tokens.push(Token::RightParen),
				'{' => self.tokens.push(Token::LeftBrace),
				'}' => self.tokens.push(Token::RightBrace),
				',' => self.tokens.push(Token::Comma),
				'.' => self.tokens.push(Token::Dot),
				'-' => self.tokens.push(Token::Minus),
				'+' => self.tokens.push(Token::Plus),
				';' => self.tokens.push(Token::Semicolon),
				'*' => self.tokens.push(Token::Star),
				_ => {
					if c == '"' {
						read_start_offset = current_offset;

						reading_string = true;
						continue;
					}
					if c.is_numeric() {
						read_start_offset = current_offset;

						reading_number = true;
						string_buf.push(c);
						continue;
					}
					if c.is_alphabetic() {
						read_start_offset = current_offset;

						reading_identifier = true;
						string_buf.push(c);
						continue;
					}
					if OPERATORS.contains(c) {
						if self.offset<bytes.len() && bytes[self.offset] as char == '=' {
							self.offset += 1;
							match c {
								'!' => self.tokens.push(Token::BangEqual),
								'=' => self.tokens.push(Token::EqualEqual),
								'<' => self.tokens.push(Token::LessEqual),
								'>' => self.tokens.push(Token::GreatEqual),
								_ => unreachable!()
							}
						} else {
							match c {
								'!' => self.tokens.push(Token::Bang),
								'=' => self.tokens.push(Token::Equal),
								'<' => self.tokens.push(Token::Less),
								'>' => self.tokens.push(Token::Greater),
								_ => unreachable!()
							}
						}
						continue;
					}
					if c == '/' {
						if self.offset<bytes.len() && bytes[self.offset] as char == '/' {
							while self.offset < bytes.len() && bytes[self.offset] as char != '\n' {
								self.offset += 1;
							}
						} else {
							self.tokens.push(Token::Slash);
						}
						continue;
					}
					errors.push(self.generate_report(format!("Invalid token '{}'", c), current_offset))
				}
			}
		}

		if reading_string {
			errors.push(
				self.generate_report(
					format!("Unterminated string {}",
						std::mem::take(&mut string_buf)
						.into_iter()
						.collect::<String>()
					),
					read_start_offset
				)
			)
		}

		self.tokens.push(Token::Eof);

		if errors.len() > 0 {
			return Err(errors.join("\n"));
		}

		Ok(std::mem::take(&mut self.tokens))
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	fn tokenize(source: &str) -> Result<Vec<Token>, String> {
		let mut tokenizer = Tokenizer::new(source);
		tokenizer.tokenize()
	}

	#[test]
	fn test_hello_world() {
		let source = r#"
		print "Hello, World!";
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Print,
			Token::String(
				String::from("Hello, World!")
			),
			Token::Semicolon,
			Token::Eof
		])
	}

	#[test]
	fn test_comments() {
		let source = r#"
		// !@#$%^&*()_+abcdefghijklmnop.,1234567890
		print "Hello, World!"; // !@#$%^&*()_+abcdefghijklmnop.,1234567890
		// !@#$%^&*()_+abcdefghijklmnop.,1234567890
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Print,
			Token::String(
				String::from("Hello, World!")
			),
			Token::Semicolon,
			Token::Eof
		])
	}

	#[test]
	fn test_numbers() {
		let source = r#"
		print 1.2;
		print 1.0;
		print 0.1;
		print 1;
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Print,
			Token::Number(1.2),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(0.1),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::Semicolon,

			Token::Eof
		])
	}

	#[test]
	fn test_strings() {
		let source = r#"
		print "Hi";
		print "\"Escapes\"";
		print "Self escapes \\";
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Print,
			Token::String(
				String::from("Hi")
			),
			Token::Semicolon,

			Token::Print,
			Token::String(
				String::from("\"Escapes\"")
			),
			Token::Semicolon,

			Token::Print,
			Token::String(
				String::from("Self escapes \\")
			),
			Token::Semicolon,

			Token::Eof
		])
	}

	#[test]
	fn test_operators() {
		let source = r#"
		print 1 == 2;
		print 1 != 2;
		print 1 > 2;
		print 1 <= 2;
		print 1 < 1;
		print !true;
		print !!true;
		print !!!true;
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Print,
			Token::Number(1.0),
			Token::EqualEqual,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::BangEqual,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::Greater,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::LessEqual,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::Less,
			Token::Number(1.0),
			Token::Semicolon,

			Token::Print,
			Token::Bang,
			Token::True,
			Token::Semicolon,

			Token::Print,
			Token::Bang,
			Token::Bang,
			Token::True,
			Token::Semicolon,

			Token::Print,
			Token::Bang,
			Token::Bang,
			Token::Bang,
			Token::True,
			Token::Semicolon,

			Token::Eof
		])
	}

	#[test]
	fn test_division() {
		let source = r#"
		print 1 / 2;
		print 1 / 2 / 2;
		print 1 / 2 // 2
		;
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Print,
			Token::Number(1.0),
			Token::Slash,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::Slash,
			Token::Number(2.0),
			Token::Slash,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Number(1.0),
			Token::Slash,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Eof
		])
	}

	#[test]
	fn test_variables() {
		let source = r#"
		var a123 = false;
		var x = 1;
		var y = 2;
		print a123;
		print x + y;
		"#;
		assert_eq!(tokenize(source).unwrap(), vec![
			Token::Var,
			Token::Identifier(
				String::from("a123")
			),
			Token::Equal,
			Token::False,
			Token::Semicolon,

			Token::Var,
			Token::Identifier(
				String::from("x")
			),
			Token::Equal,
			Token::Number(1.0),
			Token::Semicolon,

			Token::Var,
			Token::Identifier(
				String::from("y")
			),
			Token::Equal,
			Token::Number(2.0),
			Token::Semicolon,

			Token::Print,
			Token::Identifier(
				String::from("a123")
			),
			Token::Semicolon,

			Token::Print,
			Token::Identifier(
				String::from("x")
			),
			Token::Plus,
			Token::Identifier(
				String::from("y")
			),
			Token::Semicolon,

			Token::Eof
		])
	}
}