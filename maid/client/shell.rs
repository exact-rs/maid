use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParseError {
    UnterminatedQuote,
    DanglingBackslash,
    InvalidEscape(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ParseError::UnterminatedQuote => f.write_str("unterminated quote"),
            ParseError::DanglingBackslash => f.write_str("dangling backslash at end of input"),
            ParseError::InvalidEscape(c) => write!(f, "invalid escape sequence '\\{}' in double-quoted string", c),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
enum State {
    Delimiter,
    Backslash,
    Unquoted,
    UnquotedBackslash,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedBackslash,
}

pub(crate) trait IntoArgs {
    fn try_into_args(&self) -> Result<Vec<String>, ParseError>;
}

impl<S: std::ops::Deref<Target = str>> IntoArgs for S {
    fn try_into_args(&self) -> Result<Vec<String>, ParseError> {
        let mut parser = ArgumentParser::new();
        parser.parse(self)
    }
}

struct ArgumentParser {
    words: Vec<String>,
    current_word: String,
    state: State,
}

impl ArgumentParser {
    fn new() -> Self {
        Self {
            words: Vec::new(),
            current_word: String::new(),
            state: State::Delimiter,
        }
    }

    fn parse(&mut self, input: &str) -> Result<Vec<String>, ParseError> {
        use State::*;

        for c in input.chars() {
            self.state = match self.state {
                Delimiter => self.handle_delimiter(c)?,
                Backslash => self.handle_backslash(c)?,
                Unquoted => self.handle_unquoted(c)?,
                UnquotedBackslash => self.handle_unquoted_backslash(c)?,
                SingleQuoted => self.handle_single_quoted(c)?,
                DoubleQuoted => self.handle_double_quoted(c)?,
                DoubleQuotedBackslash => self.handle_double_quoted_backslash(c)?,
            };
        }

        self.handle_end_of_input()?;
        Ok(mem::take(&mut self.words))
    }

    fn push_word(&mut self) {
        if !self.current_word.is_empty() {
            self.words.push(mem::take(&mut self.current_word));
        }
    }

    fn handle_delimiter(&mut self, c: char) -> Result<State, ParseError> {
        Ok(match c {
            '\'' => State::SingleQuoted,
            '"' => State::DoubleQuoted,
            '\\' => State::Backslash,
            '\t' | ' ' | '\n' => State::Delimiter,
            c => {
                self.current_word.push(c);
                State::Unquoted
            }
        })
    }

    fn handle_backslash(&mut self, c: char) -> Result<State, ParseError> {
        Ok(match c {
            '\n' => State::Delimiter,
            c => {
                self.current_word.push(c);
                State::Unquoted
            }
        })
    }

    fn handle_unquoted(&mut self, c: char) -> Result<State, ParseError> {
        Ok(match c {
            '\'' => State::SingleQuoted,
            '"' => State::DoubleQuoted,
            '\\' => State::UnquotedBackslash,
            '\t' | ' ' | '\n' => {
                self.push_word();
                State::Delimiter
            }
            c => {
                self.current_word.push(c);
                State::Unquoted
            }
        })
    }

    fn handle_unquoted_backslash(&mut self, c: char) -> Result<State, ParseError> {
        Ok(match c {
            '\n' => State::Unquoted,
            c => {
                self.current_word.push(c);
                State::Unquoted
            }
        })
    }

    fn handle_single_quoted(&mut self, c: char) -> Result<State, ParseError> {
        Ok(match c {
            '\'' => State::Unquoted,
            c => {
                self.current_word.push(c);
                State::SingleQuoted
            }
        })
    }

    fn handle_double_quoted(&mut self, c: char) -> Result<State, ParseError> {
        Ok(match c {
            '"' => State::Unquoted,
            '\\' => State::DoubleQuotedBackslash,
            c => {
                self.current_word.push(c);
                State::DoubleQuoted
            }
        })
    }

    fn handle_double_quoted_backslash(&mut self, c: char) -> Result<State, ParseError> {
        match c {
            '\n' => Ok(State::DoubleQuoted),
            '$' | '`' | '"' | '\\' => {
                self.current_word.push(c);
                Ok(State::DoubleQuoted)
            }
            c => Err(ParseError::InvalidEscape(c)),
        }
    }

    fn handle_end_of_input(&mut self) -> Result<(), ParseError> {
        match self.state {
            State::SingleQuoted | State::DoubleQuoted => Err(ParseError::UnterminatedQuote),
            State::DoubleQuotedBackslash => Err(ParseError::DanglingBackslash),
            State::Backslash | State::UnquotedBackslash => {
                self.current_word.push('\\');
                self.push_word();
                Ok(())
            }
            _ => {
                self.push_word();
                Ok(())
            }
        }
    }
}
