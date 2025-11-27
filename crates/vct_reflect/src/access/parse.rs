use crate::access::Accessor;
use core::{
    fmt::{self, Write},
    num::ParseIntError,
};

#[derive(Debug, PartialEq, Eq)]
struct Ident<'a>(&'a str);

impl<'a> Ident<'a> {
    /// A string that correctly represents a positive integer will be converted to [`Accessor::TupleIndex`].
    /// All other string will be converted to [`Accessor::Field`] (Including incorrect ident, such as "1a2").
    ///
    /// Both start with dot and cannot be directly distinguished.
    fn field(self) -> Accessor<'a> {
        match self.0.parse() {
            Ok(index) => Accessor::TupleIndex(index),
            Err(_) => Accessor::FieldName(self.0.into()),
        }
    }

    fn field_index(self) -> Result<Accessor<'a>, InnerError<'a>> {
        Ok(Accessor::FieldIndex(self.0.parse()?))
    }

    fn list_index(self) -> Result<Accessor<'a>, InnerError<'a>> {
        Ok(Accessor::FieldIndex(self.0.parse()?))
    }
}

// NOTE: We use repr(u8) so that the `match byte` in `Token::symbol_from_byte`
// becomes a "check `byte` is one of SYMBOLS and forward its value" this makes
// the optimizer happy, and shaves off a few cycles.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum Token<'a> {
    Dot = b'.',
    Pound = b'#',
    OpenBracket = b'[',
    CloseBracket = b']',
    Ident(Ident<'a>),
}

impl Token<'_> {
    const SYMBOLS: &'static [u8] = b".#[]";

    #[inline]
    fn symbol_from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'.' => Some(Self::Dot),
            b'#' => Some(Self::Pound),
            b'[' => Some(Self::OpenBracket),
            b']' => Some(Self::CloseBracket),
            _ => None,
        }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Dot => f.write_char('.'),
            Token::Pound => f.write_char('#'),
            Token::OpenBracket => f.write_char('['),
            Token::CloseBracket => f.write_char(']'),
            Token::Ident(ident) => f.write_str(ident.0),
        }
    }
}

// Seal visibility
#[derive(Debug, PartialEq, Eq)]
enum InnerError<'a> {
    NoIdent,
    IsNotIdent(Token<'a>),
    UnexpectedIdent(Ident<'a>),
    InvalidIndex(ParseIntError),
    Unclosed,
    BadClose(Token<'a>),
    CloseBeforeOpen,
}

impl From<ParseIntError> for InnerError<'static> {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        InnerError::InvalidIndex(value)
    }
}

impl fmt::Display for InnerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InnerError::NoIdent => {
                write!(f, "expected an identifier, but reached end of path string")
            }
            InnerError::IsNotIdent(token) => {
                write!(f, "expected an identifier, got '{token}' instead")
            }
            InnerError::UnexpectedIdent(ident) => {
                write!(f, "expected a keyword ('#.[]'), got '{}' instead", ident.0)
            }
            InnerError::InvalidIndex(_) => write!(f, "failed to parse index as integer"),
            InnerError::Unclosed => write!(
                f,
                "a '[' wasn't closed, reached end of path string before finding a ']'"
            ),
            InnerError::BadClose(token) => {
                write!(f, "a '[' wasn't closed properly, got '{token}' instead")
            }
            InnerError::CloseBeforeOpen => write!(f, "a ']' was found before an opening '['"),
        }
    }
}

impl core::error::Error for InnerError<'_> {}

/// An error that occurs when parsing reflect path strings.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError<'a>(InnerError<'a>);

impl fmt::Display for ParseError<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <InnerError as fmt::Debug>::fmt(&self.0, f)
    }
}

impl core::error::Error for ParseError<'_> {}

// A one-time path parser
pub(super) struct PathParser<'a> {
    path: &'a str,
    remaining: &'a [u8],
}

impl<'a> PathParser<'a> {
    #[inline]
    pub(crate) fn new(path: &'a str) -> Self {
        Self {
            path,
            remaining: path.as_bytes(),
        }
    }

    #[inline]
    fn offset(&self) -> usize {
        self.path.len() - self.remaining.len()
    }

    // Get the next token, skip spaces.
    //
    // The obtained ident will remove the trailing space.
    fn next_token(&mut self) -> Option<Token<'a>> {
        let to_parse = self.remaining.trim_ascii_start();

        let (first_byte, remaining) = to_parse.split_first()?;

        if let Some(token) = Token::symbol_from_byte(*first_byte) {
            self.remaining = remaining;
            return Some(token);
        }

        // find indent
        let ident_len = to_parse.iter().position(|t| Token::SYMBOLS.contains(t));
        let (ident, remaining) = to_parse.split_at(ident_len.unwrap_or(to_parse.len()));
        let ident = ident.trim_ascii_end();

        // # Safety
        // - `&str` should be a valid UTF-8 string.
        // - Ensure that the passed bytes are valid UTF-8.
        #[expect(unsafe_code, reason = "Ensure that the passed bytes are valid UTF-8.")]
        let ident = unsafe { core::str::from_utf8_unchecked(ident) };

        self.remaining = remaining;
        Some(Token::Ident(Ident(ident)))
    }

    #[inline]
    fn next_ident(&mut self) -> Result<Ident<'a>, InnerError<'a>> {
        match self.next_token() {
            Some(Token::Ident(ident)) => Ok(ident),
            Some(other) => Err(InnerError::IsNotIdent(other)),
            None => Err(InnerError::NoIdent),
        }
    }

    fn following_accessor(&mut self, token: Token<'a>) -> Result<Accessor<'a>, InnerError<'a>> {
        match token {
            Token::Dot => Ok(self.next_ident()?.field()),
            Token::Pound => self.next_ident()?.field_index(),
            Token::OpenBracket => {
                let index_ident = self.next_ident()?.list_index()?;
                match self.next_token() {
                    Some(Token::CloseBracket) => Ok(index_ident),
                    Some(other) => Err(InnerError::BadClose(other)),
                    None => Err(InnerError::Unclosed),
                }
            }
            Token::CloseBracket => Err(InnerError::CloseBeforeOpen),
            Token::Ident(ident) => Err(InnerError::UnexpectedIdent(ident)),
        }
    }
}

impl<'a> Iterator for PathParser<'a> {
    type Item = (Result<Accessor<'a>, ParseError<'a>>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token()?;
        let offset = self.offset();
        Some((
            self.following_accessor(token)
                .map_err(|error| ParseError(error)),
            offset,
        ))
    }
}
