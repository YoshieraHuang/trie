use thiserror::Error;

/// Token is the smallest unit of inserting subject
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Token<'a> {
    /// normal one represented by str
    Normal(&'a str),
    /// wildcard which will always match a single token
    OneWildcard,
    /// wildcard which will always match one or more tokens
    /// but it can only appear at the end of subject
    MultiWildcard
}

/// A Wrapper for a vector of Tokens
#[derive(Debug, Default, PartialEq, Hash)]
pub struct Tokens<'a>(pub(crate) Vec<Token<'a>>);

impl<'a> From<Vec<Token<'a>>> for Tokens<'a> {
    fn from(v: Vec<Token<'a>>) -> Tokens<'a> {
        Tokens(v)
    }
}

impl<'a> Tokens<'a> {
    /// Whether it contains wildcards 
    pub fn has_no_wildcard(&self) -> bool {
        self.0.iter()
            .try_for_each(|t| {
                match t {
                    // Some(()) means true here
                    Token::Normal(_) => Some(()),
                    // None means false here and will short-circurt
                    _ => None
                }
            })
        .is_some()
    }

    /// Whether tokens is consistent with keys
    pub fn match_keys(&self, keys: impl AsRef<[&'a str]>) -> bool {
        let keys = keys.as_ref();
        // If `tokens` is longer than `keys`, these two is inconsistent
        if self.0.len() > keys.len() { return false; }
        // If `tokens` is shorter than `keys`, these two may be consistent only
        // when last token is multi wildcard, otherwise these two is inconsistent
        if self.0.len() < keys.len() {
            match self.0.last() {
                Some(Token::MultiWildcard) => { },
                _ => { return false; }
            }
        }
        // compare the two sequences one by one
        self.0.iter().zip(keys.iter())
            .try_for_each(|(t, k)| {
                match t {
                    // Some(()) means true here
                    Token::Normal(s) if s == k => Some(()),
                    Token::OneWildcard | Token::MultiWildcard => Some(()),
                    // None means false here and will short-circurt
                    _ => None 
                }
            }).is_some()
    }
}

/// Can parse bytes to token vector
pub trait TokenParser {
    type Error;

    /// Parses str to token sequence
    fn parse_tokens<'a>(&self, source: &'a str) -> Result<Tokens<'a>, Self::Error>;
}

/// Common configurations to parse something to tokens
pub struct CommonTokenParser<'b> {
    /// char to seperate tokens
    seperate_char: char,
    /// chars to represent one-token wildcard
    one_wildcard_chars: &'b str,
    /// chars to represent multi-token wildcard
    multi_wildcard_chars: &'b str,
}

impl<'b> CommonTokenParser<'b> {
    /// Returns a CommonTokenParser instance
    pub fn new(sc: char, owc: &'b str, mwc: &'b str) -> Self {
        Self {
            seperate_char: sc,
            one_wildcard_chars: owc,
            multi_wildcard_chars: mwc
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CommonTokenError {
    #[error("multi wildcard not at end")]
    MultiWildcardNotAtEnd,
}

impl<'b> TokenParser for CommonTokenParser<'b> {
    type Error = CommonTokenError;
    
    fn parse_tokens<'a>(&self, source: &'a str) -> Result<Tokens<'a>, Self::Error> {
        Ok(source
            .split(self.seperate_char)
            .try_fold((vec![], false), |(mut vec, has_mwc), s|
                if has_mwc {
                    // token after mwc
                    Err(CommonTokenError::MultiWildcardNotAtEnd)
                } else if s == self.one_wildcard_chars {
                    vec.push(Token::OneWildcard);
                    Ok((vec, false))
                } else if s == self.multi_wildcard_chars {
                    vec.push(Token::MultiWildcard);
                    Ok((vec, true))
                } else {
                    vec.push(Token::Normal(s));
                    Ok((vec, false))
                }
            )?.0.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // macro to generate token conveniently
    macro_rules! token {
        (o) => {
            Token::OneWildcard
        };
        (m) => {
            Token::MultiWildcard
        };
        ($a:literal) => {
            Token::Normal($a)
        }
    }

    #[test]
    fn test_common_token_parser() -> Result<(), CommonTokenError> {
        let parser = CommonTokenParser::new('.', "*", ">");
        assert_eq!(
            parser.parse_tokens("a.b")?,
            Tokens(vec![token!("a"), token!("b")]));
        assert_eq!(
            parser.parse_tokens("a.b.c")?,
            Tokens(vec![token!("a"), token!("b"), token!("c")])
        );
        assert_eq!(
            parser.parse_tokens("a.*.c")?,
            Tokens(vec![token!("a"), token!(o), token!("c")])
        );
        assert_eq!(
            parser.parse_tokens("a.b.*")?,
            Tokens(vec![token!("a"), token!("b"), token!(o)])
        );
        assert_eq!(parser.parse_tokens("*")?, Tokens(vec![token!(o)]));
        assert_eq!(parser.parse_tokens("")?, Tokens(vec![token!("")]));
        assert_eq!(parser.parse_tokens("..")?,
            Tokens(vec![token!(""), token!(""), token!("")]));
        assert_eq!(
            parser.parse_tokens("a.b.>")?,
            Tokens(vec![token!("a"), token!("b"), token!(m)])
        );
        assert_eq!(
            parser.parse_tokens("a.>")?,
            Tokens(vec![token!("a"), token!(m)])
        );
        assert_eq!(
            parser.parse_tokens(">")?,
            Tokens(vec![token!(m)])
        );
        assert_eq!(parser.parse_tokens(">.a").unwrap_err(), CommonTokenError::MultiWildcardNotAtEnd);
        Ok(())
    }

    #[test]
    fn test_matcher() {
        assert_eq!(Tokens(vec![token!("a"), token!("b"), token!("c")]).has_no_wildcard(), true);
        assert_eq!(Tokens(vec![token!("a"), token!(o), token!("c")]).has_no_wildcard(), false);
        assert_eq!(Tokens(vec![token!("a"), token!(o), token!(o)]).has_no_wildcard(), false);        
        assert_eq!(Tokens(vec![token!("a"), token!(o), token!(m)]).has_no_wildcard(), false);
        let tokens = Tokens(vec![token!("a"), token!("b"), token!("c")]);
        assert_eq!(tokens.match_keys(vec!["a", "b", "c"]), true);
        assert_eq!(tokens.match_keys(vec!["a", "b"]), false);
        assert_eq!(tokens.match_keys(vec!["b", "a", "c"]), false);
        assert_eq!(tokens.match_keys(vec!["a", "b", "c", "d"]), false);
        let tokens = Tokens(vec![token!("a"), token!(o)]);
        assert_eq!(tokens.match_keys(vec!["a", "b"]), true);
        assert_eq!(tokens.match_keys(vec!["a", "c"]), true);
        assert_eq!(tokens.match_keys(vec!["b", "c"]), false);
        assert_eq!(tokens.match_keys(vec!["a", "b", "c"]), false);
        let tokens = Tokens(vec![token!("a"), token!(m)]);
        assert_eq!(tokens.match_keys(vec!["a", "b"]), true);
        assert_eq!(tokens.match_keys(vec!["a", "c"]), true);
        assert_eq!(tokens.match_keys(vec!["b", "c"]), false);
        assert_eq!(tokens.match_keys(vec!["a", "b", "c"]), true);
        let tokens = Tokens(vec![token!("a"), token!(o), token!(m)]);
        assert_eq!(tokens.match_keys(vec!["a", "b"]), false);
        assert_eq!(tokens.match_keys(vec!["a", "c"]), false);
        assert_eq!(tokens.match_keys(vec!["b", "c"]), false);
        assert_eq!(tokens.match_keys(vec!["a", "b", "c"]), true);
    }
}