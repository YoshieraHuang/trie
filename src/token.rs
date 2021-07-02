use thiserror::Error;

/// Token is the smallest unit of inserting subject
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    /// normal one represented by str
    Normal(&'static str),
    /// wildcard which will always match a single token
    OneWildcard,
    /// wildcard which will always match one or more tokens
    /// but it can only appear at the end of subject
    MultiWildcard
}

/// Can parse bytes to token vector
pub trait TokenParser {
    type Error;
    // Parses str to token sequence
    fn parse_tokens(&self, source: &'static str) -> Result<Vec<Token>, Self::Error>;
}

/// Common configurations to parse something to tokens
pub struct CommonTokenParser {
    /// char to seperate tokens
    seperate_char: char,
    /// chars to represent one-token wildcard
    one_wildcard_chars: &'static str,
    /// chars to represent multi-token wildcard
    multi_wildcard_chars: &'static str,
}

impl CommonTokenParser {
    /// Returns a CommonTokenParser instance
    pub fn new(sc: char, owc: &'static str, mwc: &'static str) -> Self {
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

impl TokenParser for CommonTokenParser {
    type Error = CommonTokenError;
    
    fn parse_tokens(&self, source: &'static str) -> Result<Vec<Token>, Self::Error> {
        let tokens = source
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
            )?.0;
        
        return Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_vec_eq {
        ($a: expr, $b: expr) => {
            assert_eq!(&$a[..], &$b[..])
        };
    }

    #[test]
    fn test_common_token_parser() -> Result<(), CommonTokenError> {
        let parser = CommonTokenParser::new('.', "*", ">");
        assert_vec_eq!(
            parser.parse_tokens("a.b")?,
            vec![Token::Normal("a"), Token::Normal("b")]);
        assert_vec_eq!(
            parser.parse_tokens("a.b.c")?,
            vec![Token::Normal("a"), Token::Normal("b"), Token::Normal("c")]
        );
        assert_vec_eq!(
            parser.parse_tokens("a.*.c")?,
            vec![Token::Normal("a"), Token::OneWildcard, Token::Normal("c")]
        );
        assert_vec_eq!(
            parser.parse_tokens("a.b.*")?,
            vec![Token::Normal("a"), Token::Normal("b"), Token::OneWildcard]
        );
        assert_vec_eq!(parser.parse_tokens("*")?, vec![Token::OneWildcard]);
        assert_vec_eq!(parser.parse_tokens("")?, vec![Token::Normal("")]);
        assert_vec_eq!(parser.parse_tokens("..")?,
            vec![Token::Normal(""), Token::Normal(""), Token::Normal("")]);
        assert_vec_eq!(
            parser.parse_tokens("a.b.>")?,
            vec![Token::Normal("a"), Token::Normal("b"), Token::MultiWildcard]
        );
        assert_vec_eq!(
            parser.parse_tokens("a.>")?,
            vec![Token::Normal("a"), Token::MultiWildcard]
        );
        assert_vec_eq!(
            parser.parse_tokens(">")?,
            vec![Token::MultiWildcard]
        );
        assert_eq!(parser.parse_tokens(">.a").unwrap_err(), CommonTokenError::MultiWildcardNotAtEnd);
        Ok(())
    }
}