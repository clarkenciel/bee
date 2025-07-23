pub type Bitmask = i32;

/// Compute the bitmask of a word.
///
/// # Panics
///
/// Panics if the word contains any characters that are not
/// covered by [`std::char::to_digit`](https://doc.rust-lang.org/std/primitive.char.html#method.to_digit)
pub fn bitmask(word: &str) -> Bitmask {
    word.chars().fold(0, |bm, c| {
        bm | letters::bitmask(&c)
    })
}

pub fn vec_from_bitmask(bm: &Bitmask) -> Vec<char> {
    (0..26).into_iter().filter_map(|offset| {
        let mask = bm & (1 << offset);
        if mask > 0 {
            Some(crate::letters::from_bitmask(&mask))
        } else {
            None
        }
    })
    .collect()
}

#[test]
fn test_vec_roundtrip() {
    assert_eq!(
        vec!['a', 'b', 'c', 'h', 's', 'u'],
        vec_from_bitmask(&bitmask("bacchus")),
    )
}

/// Utilities to bitmask individual characters
///
/// ## Round tripping
///
/// if you attempt to round trip a character through bitmask -> from_bitmask
/// and that charcter is not a lowercase ascii/latin character you 
pub mod letters {
    const REFERENCE_ORD: i32 = 'a'.to_ascii_lowercase() as u8 as i32; // 0b1100001; // 'a'

    /// Compute the bitmask of a character.
    ///
    /// This bitmask will be an i32 with the bit that corresponds to `letter`'s
    /// position (0-indexed) in the lowercase latin alphabet set to 1.
    pub fn bitmask(letter: &char) -> super::Bitmask {
        (1 << *letter as u8 as i32 - REFERENCE_ORD) as super::Bitmask
    }

    /// Reverse the process of `bitmask`.
    ///
    /// This assumes that `bm` is a bitmask with only one bit set to `1`.
    /// It also assumes the bitmask represents a lowercase character.
    ///
    /// ## Panics
    ///
    /// If more than one position is set it may panic.
    pub fn from_bitmask(bm: &super::Bitmask) -> char {
        LETTERS[bm.ilog2() as usize]
    }

    const LETTERS: [char; 26] = [
        'a',
        'b',
        'c',
        'd',
        'e',
        'f',
        'g',
        'h',
        'i',
        'j',
        'k',
        'l',
        'm',
        'n',
        'o',
        'p',
        'q',
        'r',
        's',
        't',
        'u',
        'v',
        'w',
        'x',
        'y',
        'z',
    ];

    #[test]
    fn test_roundtrip() {
        assert_eq!(
            'c',
            from_bitmask(&bitmask(&'c')),
        )
    }
}

