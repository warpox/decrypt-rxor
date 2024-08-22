//! Tools for handling data encoded with various methods

/// A string of bytes represented as a hex pair (e.g. 000102FF would be [0x00, 0x01, 0x02, 0xFF] u8s)
#[derive(Clone, Debug, PartialEq)]
pub struct Hex(String);

/// A string of bytes that's been base64 encoded
#[derive(Clone, Debug, PartialEq)]
pub struct Base64(String);

/// A string of ASCII characters
#[derive(Clone, Debug, PartialEq)]
pub struct Ascii(String);

/// Raw bytes interface, primary type for decoded interactions
#[derive(Clone, Debug, PartialEq)]
pub struct Raw(Vec<u8>);

/// trait for explicit decoding (if not using From/Into semantics)
pub trait Decode {
    fn decode(&self) -> Raw;
}

/// trait for explicit encoding (if not using From/Into)
pub trait Encode {
    fn encode(this: &Raw) -> Self;
}

// basic interface type API
impl Raw {
    /// construct (decode) a raw instance that can be moved into a Raw
    pub fn new<T: Into<Raw>>(moved: T) -> Raw {
        moved.into()
    }

    /// read only access to bytes
    pub fn get(&self) -> &Vec<u8> {
        &self.0
    }

    /// determines the bit-level hamming distance between two Raw sequences
    pub fn hamming(&self, other: &Self) -> usize {
        let mut distance = 0;

        for (l, r) in self.0.iter().zip(other.0.iter()) {
            for b in 0..8 {
                let bit = 1 << b;

                if (l & bit) != (r & bit) {
                    distance += 1;
                }
            }
        }

        distance
    }

    /// normalized hamming function (divide through by bits)
    pub fn hamming_normalized(&self, other: &Self) -> f32 {
        self.hamming(other) as f32 / (8.0 * self.len() as f32)
    }

    /// get iter to u8s
    pub fn iter(&self) -> core::slice::Iter<'_, u8> {
        self.0.iter()
    }

    /// len of internal vec
    pub fn len(&self) -> usize {
        self.0.len()
    }

}

impl std::ops::Index<usize> for Raw {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl From<Vec<u8>> for Raw {
    fn from(value: Vec<u8>) -> Self {
        Raw(value)
    }
}

impl From<&[u8]> for Raw {
    fn from(value: &[u8]) -> Self {
        Raw(Vec::from_iter(value.iter().cloned()))
    }
}

impl Ascii {
    /// if an ASCII string is provided, it will become an ASCII type
    pub fn new(s: String) -> Option<Self> {
        if s.is_ascii() {
            Some(Self(s))
        } else {
            None
        }
    }
}

impl Hex {
    /// validate input is a Hex string, then consume into Hex
    pub fn new(s: String) -> Option<Self> {
        if Hex::string_is(&s) {
            Some(Self(s))
        } else {
            None
        }
    }

    /// check if a char is a hex symbol (0-F)
    pub fn is(c: char) -> bool {
        match c {
            'a'..='f' => true,
            'A'..='F' => true,
            '0'..='9' => true,
            _ => false,
        }
    }

    /// validate all the chars in a string to be (0-F)
    pub fn string_is(s: &String) -> bool {
        for c in s.chars() {
            if !Hex::is(c) {
                return false;
            }
        }

        true
    }

    fn to_u8(c: char) -> u8 {
        match c {
            'a'..='f' => c as u8 - 'a' as u8 + 10,
            'A'..='F' => c as u8 - 'A' as u8 + 10,
            '0'..='9' => c as u8 - '0' as u8,
            _ => panic!(), /* unreachable! */
        }
    }

    fn from_u8(u: u8) -> char {
        if u < 10 {
            (u + '0' as u8) as char
        } else {
            (u - 10 + 'a' as u8) as char
        }
    }
}

impl Base64 {
    /// construct a Base64 representation, if the string is valid Base64 encoding
    pub fn new(s: String) -> Option<Self> {
        if Base64::is_string(&s) {
            Some(Self(s))
        } else {
            None
        }
    }

    pub fn is(c: char) -> bool {
        match c {
            '0'..='9' => true,
            'a'..='z' => true,
            'A'..='Z' => true,
            '+' => true,
            '/' => true,
            '=' => true, // pad
            _ => false,
        }
    }

    pub fn is_string(s: &String) -> bool {
        for c in s.chars() {
            if !Base64::is(c) {
                return false;
            }
        }
        true
    }

    fn from_u8(u: u8) -> char {
        let mut map: Vec<char> = Vec::new();

        for c in 'A'..='Z' {
            map.push(c);
        }

        for c in 'a'..='z' {
            map.push(c);
        }

        for c in '0'..='9' {
            map.push(c);
        }

        map.push('+');
        map.push('/');

        map[u as usize]
    }
    fn to_u8(c: char) -> u8 {
        match c {
            'A'..='Z' => c as u8 - 'A' as u8 + 0,
            'a'..='z' => c as u8 - 'a' as u8 + 26,
            '0'..='9' => c as u8 - '0' as u8 + 52,
            '+' => 62,
            '/' => 63,
            '=' => 0,
            _ => panic!(), // unreachable
        }
    }
}

impl From<String> for Raw {
    fn from(value: String) -> Self {
        Raw(value.into_bytes())
    }
}

// ASCII : RAW conversions

impl Decode for Ascii {
    fn decode(&self) -> Raw {
        // we already know that all the chars in c are ASCII types from new()
        Raw(self.0.clone().into_bytes())
    }
}

impl Encode for Ascii {
    fn encode(this: &Raw) -> Self {
        let mut s = String::new();

        for b in this.0.clone() {
            s.push(b as char);
        }

        Self(s)
    }
}

impl From<Ascii> for Raw {
    fn from(value: Ascii) -> Self {
        value.decode()
    }
}

impl From<Raw> for Ascii {
    fn from(value: Raw) -> Self {
        Ascii::encode(&value)
    }
}

// HEX : RAW conversions
impl Decode for Hex {
    fn decode(&self) -> Raw {
        let mut v = Vec::new();

        let mut word = 0;

        for (i, c) in self.0.chars().enumerate() {
            let bits = Hex::to_u8(c);

            if i % 2 == 0 {
                word = bits << 4;
            } else {
                v.push(word | bits);
            }
        }

        if self.0.len() % 2 == 1 {
            v.push(word);
        }

        Raw(v)
    }
}

impl Encode for Hex {
    fn encode(this: &Raw) -> Self {
        let mut s = String::new();
        for b in &this.0 {
            let upper = Hex::from_u8((b & 0xF0) >> 4);
            let lower = Hex::from_u8(b & 0xF);
            s.push(upper);
            s.push(lower);
        }

        Hex(s)
    }
}

impl From<Hex> for Raw {
    fn from(value: Hex) -> Self {
        value.decode()
    }
}

impl From<Raw> for Hex {
    fn from(value: Raw) -> Self {
        Hex::encode(&value)
    }
}

// Base64 : RAW conversions
impl Decode for Base64 {
    fn decode(&self) -> Raw {
        let mut v = Vec::new();

        // need to combine groups of 6 into groups of 8
        //let padding_bits = 2 * self.0.chars().filter(|c| *c == '=').count();

        // take it in 6 bits at a time
        let mut bit_index = 0;
        let mut working_byte = 0;

        for b64 in self.0.chars() {
            // we've reached the end
            if b64 == '=' {
                break;
            }

            let six_bits = Base64::to_u8(b64);

            // bit index = where we "left off" in the previous
            // byte.
            // Cases:
            // base case: 0
            //   [_ _ _ _ _ _ _ _] [_ _ _ _ _ _ _ _]
            //   0 1 2 3 4 5 6 7  *8
            // 0> x x x x x x _ _
            // next case: we've got upper six bits sitting here. bit index == 6
            //   [_ _ _ _ _ _ _ _] [_ _ _ _ _ _ _ _]
            //   0 1 2 3 4 5 6 7  *8
            // 6> - - - - - - x x   x x x x _ _ _ _
            // next case: we've got 4 bits in the working buffer. Bit index == 4
            //   [_ _ _ _ _ _ _ _] [_ _ _ _ _ _ _ _]
            //   0 1 2 3 4 5 6 7  *8
            // 4> - - - - x x x x   x x _ _ _ _ _ _
            // final case, we've got 2 bits loaded, bit index == 2
            //   [_ _ _ _ _ _ _ _] [_ _ _ _ _ _ _ _]
            //   0 1 2 3 4 5 6 7  *8
            // 2> - - x x x x x x   _ _ _ _ _ _ _ _
            // at this point, bit index should wrap back around to 0
            match bit_index {
                // base case, on a word boundary
                0 => working_byte = six_bits << 2,
                6 => {
                    working_byte |= (six_bits & 0b11_00_00) >> 4;
                    v.push(working_byte);
                    working_byte = (six_bits & 0b00_11_11) << 4;
                }
                4 => {
                    working_byte |= (six_bits & 0b11_11_00) >> 2;
                    v.push(working_byte);
                    working_byte = (six_bits & 0b00_00_11) << 6;
                }
                2 => {
                    working_byte |= (six_bits & 0b11_11_11) << 0;
                    v.push(working_byte);
                }

                _ => panic!(), // unreachable
            }

            bit_index = (bit_index + 6) % 8;
        }

        //working_byte <<= padding_bits;
        // if padding_bits {
        //     v.push(working_byte >> padding_bits);
        // }

        Raw(v)
    }
}

impl Encode for Base64 {
    fn encode(this: &Raw) -> Self {
        // take 8 bit words and generate 6 bit phrases
        let mut s = String::new();
        let mut bit_index: u128 = 0;
        let mut buffer = 0;

        for byte in &this.0 {
            // Cases:
            //    | _ _ _ _ _ _ | _ _ _ _ _ _ | ...
            // 0>   x x x x x x   x x _ _ _ _
            //    | _ _ _ _ _ _ | _ _ _ _ _ _ | ...
            // 2>   - - x x x x   x x x x _ _
            //    | _ _ _ _ _ _ | _ _ _ _ _ _ | ...
            // 4>   - - - - x x   x x x x x x
            // <and wraps>
            match bit_index % 6 {
                0 => {
                    s.push(Base64::from_u8(byte >> 2));
                    buffer = (byte & 0b11) << 4;
                }
                2 => {
                    s.push(Base64::from_u8(buffer | (byte >> 4)));
                    buffer = (byte & 0b1111) << 2;
                }
                4 => {
                    s.push(Base64::from_u8(buffer | (byte >> 6)));
                    s.push(Base64::from_u8(byte & 0b111111));
                }
                _ => panic!(), // unreachable
            }

            bit_index += 8;
        }

        let starting_bits = this.0.len() * 8;
        let total_b64 = (starting_bits + 5) / 6;
        let padding_bits = total_b64 * 6 - starting_bits;

        if padding_bits != 0 {
            s.push(Base64::from_u8(buffer));

            for _ in 0..(padding_bits / 2) {
                s.push('=');
            }
        }

        Self(s)
    }
}

impl From<Raw> for Base64 {
    fn from(value: Raw) -> Self {
        Base64::encode(&value)
    }
}

impl From<Base64> for Raw {
    fn from(value: Base64) -> Self {
        value.decode()
    }
}

mod test {
    use super::*;

    #[test]
    fn test_all() {
        // test to raw and back
        let (hex_exp, b64_exp) =
            ("49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d", "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t");

        let hex = Hex::new(hex_exp.to_string()).unwrap();
        let b64: Base64 = Base64::new(b64_exp.to_string()).unwrap();

        let raw_hex = hex.decode();
        let back_to_hex: Hex = raw_hex.into();
        assert_eq!(back_to_hex, hex);

        let raw_b64 = b64.decode();
        let back_to_b64: Base64 = raw_b64.into();
        assert_eq!(back_to_b64, b64);

        let ascii_exp = "Hello my baby, hello my darlin', hello my rag-time gal...\n";
        let ascii = Ascii::new(ascii_exp.to_string()).unwrap();
        let raw_ascii = ascii.decode();
        let back_to_ascii: Ascii = raw_ascii.into();
        assert_eq!(back_to_ascii, ascii);

        // convert between a few (should be covered by above, technically)
        let b64_from_hex: Base64 = hex.decode().into();
        assert_eq!(b64_from_hex, b64);

        let hex_from_b64: Hex = b64.decode().into();
        assert_eq!(hex_from_b64, hex);

        let hex_from_ascii: Hex = ascii.decode().into();
        let ascii_from_hex: Ascii = hex_from_ascii.decode().into();
        assert_eq!(ascii_from_hex, ascii);

        let b64_from_ascii: Base64 = ascii.decode().into();
        let ascii_from_b64: Ascii = b64_from_ascii.decode().into();
        assert_eq!(ascii_from_b64, ascii);

        // test a few more ascii sequences
        let ascii0 = Ascii::new("".to_string()).unwrap();
        let ascii1 = Ascii::new("0".to_string()).unwrap();
        let ascii2 = Ascii::new("01".to_string()).unwrap();
        let ascii3 = Ascii::new("012".to_string()).unwrap();
        let ascii4 = Ascii::new("0123".to_string()).unwrap();
        let ascii5 = Ascii::new("01234".to_string()).unwrap();
        let test_ascii_b64 = |a: Ascii| {
            let b64 : Base64 = a.decode().into();
            let ra : Ascii = b64.decode().into();

            assert_eq!(ra, a);
        };

        test_ascii_b64(ascii0);
        test_ascii_b64(ascii1);
        test_ascii_b64(ascii2);
        test_ascii_b64(ascii3);
        test_ascii_b64(ascii4);
        test_ascii_b64(ascii5);
    }

    #[test]
    fn test_hamming() {
        let lhs = Ascii::new("this is a test".to_string()).unwrap();
        let rhs = Ascii::new("wokka wokka!!!".to_string()).unwrap();

        assert_eq!(lhs.decode().hamming(&rhs.decode()), 37);

    }
}
