use std::collections::VecDeque;
use std::fmt::Display;
use UnescapeError::*;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum UnescapeError {
    OnlyOneSlashError,
    IllegalEscape,
    EmptyUnicode,
    UnclosedUnicode,
    IllegalUnicode,
    TooLongUnicode,
    ValueOutOfUnicode,
    IllegalSurrogate,
    InvalidCharInUnicode,
    TooShortEscape,
    InvalidCharInHex,
    ValueOutOfHex,
}

impl Display for UnescapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OnlyOneSlashError => "Only one slash found, expected an escape sequence".to_string(),
            IllegalEscape => "Illegal escape sequence".to_string(),
            EmptyUnicode => "Empty unicode escape sequence".to_string(),
            UnclosedUnicode => "Unclosed unicode escape sequence".to_string(),
            IllegalUnicode => "Illegal unicode escape sequence".to_string(),
            TooLongUnicode => "Too long unicode escape sequence".to_string(),
            ValueOutOfUnicode => "Value out of unicode range".to_string(),
            IllegalSurrogate => "Illegal surrogate pairs in unicode escape sequence".to_string(),
            InvalidCharInUnicode => "Invalid character in unicode escape sequence".to_string(),
            TooShortEscape => "Too short escape sequence".to_string(),
            InvalidCharInHex => "Invalid character in hexadecimal escape sequence".to_string(),
            ValueOutOfHex => "Value out of hexadecimal range".to_string(),
        };
        write!(f, "{}", str)
    }
}

pub fn unescape(input: &str) -> Result<String, UnescapeError> {
    let mut que = input.chars().collect::<VecDeque<char>>();
    let mut res: String = String::new();

    if input.is_empty() {
        return Ok(res);
    }

    while let Some(c) = que.pop_front() {
        if c != '\\' {
            res.push(c);
            continue;
        }

        let esc = que.pop_front().ok_or(OnlyOneSlashError)?;
        match esc {
            'b' => res.push('\u{0008}'),
            'r' => res.push('\r'),
            'n' => res.push('\n'),
            't' => res.push('\t'),
            '\'' => res.push('\''),
            '\\' => res.push('\\'),
            'u' => {
                if que.is_empty() || !que.iter().any(|&c| c == '}'){
                    return Err(UnclosedUnicode);
                }

                if que.pop_front().unwrap() != '{' {
                    return Err(IllegalUnicode);
                }

                let mut digits: u32 = 0;
                let mut value: u32 = 0;

                while let Some(x) = que.pop_front() {
                    if digits > 6 {
                        return Err(TooLongUnicode);
                    }
                    
                    if x == '}' {
                        if digits == 0 {
                            return Err(EmptyUnicode);
                        }
                        
                        if value > 0x10FFFF {
                            return Err(ValueOutOfUnicode);
                        }
                        
                        let ch = char::from_u32(value).ok_or(IllegalSurrogate)?;
                        res.push(ch);
                        break;
                    }
                    
                    if !x.is_ascii_hexdigit() {
                        return Err(InvalidCharInUnicode);
                    }
                    
                    digits += 1;
                    value = (value << 4) | x.to_digit(16).unwrap();
                }
                
            }

            'x' => {
                let high = que.pop_front().ok_or(TooShortEscape)?;
                let high = high.to_digit(16).ok_or(InvalidCharInHex)?;

                let low = que.pop_front().ok_or(TooShortEscape)?;
                let low = low.to_digit(16).ok_or(InvalidCharInHex)?;

                let val = high * 16 + low;

                if val > 0x7f {
                    return Err(ValueOutOfHex);
                }

                res.push(val as u8 as char);
            }

            _ => return Err(IllegalEscape),
        }
    }

    Ok(res)
}