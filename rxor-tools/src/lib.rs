//! Repeating XOR Cipher Toolset

pub mod encodings;
pub use encodings::*;

/// xor a sequence against a fixed length cyclical key
pub fn xor(seq: &Raw, key: &Raw) -> Raw {
    let mut ki = 0;
    let mut r = Vec::new();

    for s in seq.iter() {
        r.push(s ^ key[ki]);
        ki = (ki + 1) % key.len();
    }

    Raw::new(r)
}

/// attempt to guess the key length of a given encrypted sequence
pub fn find_key_len(raw : &Raw, llim: usize, hlim: usize) -> Vec<usize> {
    assert!(llim <= hlim);
    let mut distances = Vec::new();
    for _ in llim..=hlim {
        distances.push(0.0f32);
    }

    let mut guesses = Vec::new();

    // compute hamming distances for each key len
    for n in llim..=hlim {
        let blocks = raw.len() / n;

        // assuming key is smaller than 1/2 the input size
        if blocks < 2 {
            distances[n - llim] = f32::INFINITY;
        } else {
            // single block algorithm (todo! could be expanded to multiple block average)
            let s0: Raw = raw.get()[0..n].into();
            let s1: Raw = raw.get()[n..(2*n)].into();

            distances[n - llim] = s0.hamming_normalized(&s1);
        }
    }

    // get top 5% or set
    let top_n = ((hlim - llim) as f32 * 0.05).max(1.0) as usize;

    for _ in 0..top_n {
        let mut best = (f32::INFINITY, 0usize);
        for (i, d) in distances.iter().enumerate() {
            if *d < best.0 && !guesses.contains(&i) {
                best = (*d, i + llim);
            }
        }

        guesses.push(best.1);
    }

    guesses
}

mod test {
    use super::*;

    #[test]
    fn test_find_key_len() {
        let seq = "Secret Message!".to_string();
        let key = "ABC".to_string();

        let raw_seq: Raw = Ascii::new(seq).unwrap().into();
        let raw_key: Raw = Ascii::new(key).unwrap().into();

        let encrypted_seq = xor(&raw_seq, &raw_key);

        let key_len = find_key_len(&encrypted_seq, 1, 5);
        assert_eq!(key_len[0], 3);
    }
}
