use {
    serde::{Deserialize, Serialize},
    std::fmt::{self, Write},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(try_from = "&str", into = "String")]
pub struct Color(pub(crate) [u8; 3]);

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn to_hex(v: u8) -> u8 {
            match v {
                0..=9 => b'0' + v,
                10..=15 => b'A' + v - 10,
                _ => unreachable!(),
            }
        }

        for byte in self.0 {
            let a = byte >> 4;
            let b = byte & 0b1111;
            f.write_char(to_hex(a) as char)?;
            f.write_char(to_hex(b) as char)?;
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a str> for Color {
    type Error = ParseError<'a>;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        fn from_hex(v: u8) -> Option<u8> {
            match v {
                b'0'..=b'9' => Some(v - b'0'),
                b'a'..=b'f' => Some(v - b'a' + 10),
                b'A'..=b'F' => Some(v - b'A' + 10),
                _ => None,
            }
        }

        let bytes = s.as_bytes();
        if bytes.len() != 6 {
            return Err(ParseError(s));
        }

        let mut col = [0; 3];
        for (pair, colbyte) in bytes.chunks(2).zip(&mut col) {
            let [a, b]: [u8; 2] = pair.try_into().expect("bytes pair");
            *colbyte =
                (from_hex(a).ok_or(ParseError(s))? << 4) | from_hex(b).ok_or(ParseError(s))?;
        }

        Ok(Self(col))
    }
}

impl From<Color> for String {
    fn from(col: Color) -> Self {
        col.to_string()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ParseError<'a>(&'a str);

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse {:?} to rgb color", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let str = "FF0023";
        let col = Color::try_from(str).expect("parse color");
        assert_eq!(col, Color([0xFF, 0x00, 0x23]));
    }

    #[test]
    fn to_str() {
        let col = Color([0xFF, 0x00, 0x23]);
        let str = String::from(col);
        assert_eq!(str, "FF0023");
    }
}
