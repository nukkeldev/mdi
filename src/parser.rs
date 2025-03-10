//! An extensible, UTF-8 CommonMark 0.31.2 semi-compliant Markdown parser.
//!
//! The parser has diverged from the specification in the following ways:
//! - `\r` is not a valid line-ending on it's own.

// -- Errors --

use std::marker::PhantomData;

#[derive(Debug)]
pub enum ParseError {
    NotYetImplemented(&'static str),
    EOF,
}

// -- AST --

pub type Snippet<'a> = Vec<Element<'a>>;

/// An uncoupled slice of some source text.
#[derive(Debug)]
pub struct Span<'a> {
    /// The first character of the element.
    start: usize,
    /// The last character of the element.
    end: usize,

    // Adds an implicit lifetime requirement.
    _marker: PhantomData<&'a str>,
}

#[derive(Debug)]
pub enum Element<'a> {
    Heading { level: u8, contents: Snippet<'a> },
    Text(Span<'a>),
    Line(Snippet<'a>),
}

// -- Spans --

impl<'a> Span<'a> {
    // Initialization

    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            _marker: PhantomData,
        }
    }

    pub fn one(start: usize) -> Self {
        Self {
            start,
            end: start,
            _marker: PhantomData,
        }
    }

    // Editing

    pub fn push(&mut self) {
        self.end += 1;
    }

    // Resolution

    pub fn resolve_unchecked(&self, source: &'a str) -> &'a str {
        &source[self.start..=self.end]
    }
}

// -- Parser --

pub struct Parser<'a, Lines: Iterator<Item = &'a str>> {
    /// The current line being processed.
    line: Option<&'a str>,
    /// The line number of the current line, starts from 1.
    line_number: usize,
    /// The index of the first character of this line.
    line_start: usize,
    /// An iterator to consume the next line.
    lines: Lines,
    /// The source.
    source: &'a str,

    /// A collection of parsed elements.
    elements: Snippet<'a>,
}

impl<'a, I: Iterator<Item = &'a str>> Parser<'a, I> {
    // Initialization

    pub fn from_str(input: &'a str) -> Parser<'a, impl Iterator<Item = &'a str>> {
        let lines = input.split_inclusive(&['\n']).filter_map(|line| {
            // Trim carriage returns
            let line = line.trim_end_matches('\r');

            // Filter empty lines
            if line.is_empty() {
                return None;
            }

            return Some(line);
        });

        Parser {
            line: None,
            line_number: 0,
            line_start: 0,
            lines,
            source: input,
            elements: vec![],
        }
    }

    // Parsing

    /// Parses the next line of the input, returning `None` on success.
    pub fn parse_line(&mut self) -> Option<ParseError> {
        // Queue the next line
        self.line = self.lines.next();
        self.line_number += 1;

        // Check if we have a line to parse
        if self.line.is_none() {
            return Some(ParseError::EOF);
        }

        // Expand the line into it's characters
        let mut chars = self.line.unwrap().char_indices();
        let mut elements: Vec<Element<'a>> = vec![];
        let mut current_element: Option<Element<'a>> = None;

        // Parse the elements on the line.
        loop {
            match chars.next() {
                Some((i, _)) => {
                    // All other elements are not valid for this character so it must be a text character.
                    if current_element.is_none() {
                        current_element = Some(Element::Text(Span::<'a>::one(i)))
                    } else if let Some(Element::Text(s)) = &mut current_element {
                        s.push();
                    }
                }
                None => {
                    if let Some(element) = current_element.take() {
                        elements.push(element);
                    }
                }
            }
        }

        // Reorganize the parsed elements

        // Append the collected element

        None
    }
}

// -- Tests --

#[cfg(test)]
mod tests {
    use super::*;

    fn init_logger() {
        let _ = pretty_env_logger::formatted_builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn debug() {
        init_logger();

        const INPUT: &str = concat![
            "LINE 1\n",
            "LINE 2\r\n",
            "\t\t\t \t\n",
            "BLANK LINE ABOVE\n"
        ];

        // let _ = parse(INPUT);
    }
}
