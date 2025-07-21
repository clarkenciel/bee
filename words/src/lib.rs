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

pub mod letters {
    const REFERENCE_ORD: super::Bitmask = 0b1100001; // 'a'

    /// Compute the bitmask of a character.
    ///
    /// # Panics
    ///
    /// Panics if the character is not covered by 
    /// [`std::char::to_digit`](https://doc.rust-lang.org/std/primitive.char.html#method.to_digit)
    pub fn bitmask(letter: &char) -> super::Bitmask {
        1 << letter.to_ascii_lowercase() as super::Bitmask - REFERENCE_ORD
    }
}
