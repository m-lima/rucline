// TODO: Add more characters to word separation

pub(super) fn next_word(pivot: usize, string: &[char]) -> usize {
    let end = string.len();
    if pivot == end {
        pivot
    } else {
        let mut index = pivot;

        // Go through the characters of the word
        while index < end && !string[index].is_whitespace() {
            index += 1;
        }

        // Go through the trailing whitespace, if any
        while index < end && string[index].is_whitespace() {
            index += 1;
        }

        index
    }
}

pub(super) fn previous_word(pivot: usize, string: &[char]) -> usize {
    if pivot == 0 {
        pivot
    } else {
        let mut index = pivot - 1;

        // Go through the trailing whitespace, if any
        while index > 0 && string[index].is_whitespace() {
            index -= 1;
        }

        // Go through the characters of the word
        while index > 0 && !string[index - 1].is_whitespace() {
            index -= 1;
        }

        index
    }
}

pub(super) fn previous_word_end(pivot: usize, string: &[char]) -> usize {
    if pivot == 0 {
        pivot
    } else {
        let mut index = pivot - 1;

        // At the end of the string, go through all trailing whitespace
        if pivot == string.len() {
            while index > 0 && string[index].is_whitespace() {
                index -= 1;
            }
        }

        // Go through the leading characters of the current word
        while index > 0 && !string[index].is_whitespace() {
            index -= 1;
        }

        // Go through the leading spaces of the current word
        while index > 0 && string[index - 1].is_whitespace() {
            index -= 1;
        }

        index
    }
}

// // TODO
// #[cfg(test)]
// mod test {
//     use crate::prompt::char_string::CharString;
//
//     const EMPTY: CharString = CharString::new();
//     const WHITE_SPACE: CharString = CharString::from("   \t   ");
//     const PRE_WHITE_SPACE: CharString = CharString::from("AddZ   \t   ");
//     const POST_WHITE_SPACE: CharString = CharString::from("   \t   AddZ");
//     const INTRA_WHITE_SPACE: CharString = CharString::from("   \t   AddZ   \t   ");
//     const MULTIPLE_WORDS_A: CharString = CharString::from("AddZ AdZ  AZ   A AZ  AdZ   AddZ");
//     const MULTIPLE_WORDS_Z: CharString = CharString::from("AddZ AdZ  AZ   Z AZ  AdZ   AddZ");
//
//     enum Allowance {
//         Start,
//         End,
//     }
//
//     fn test_scenario<F: Fn(usize, &[char]) -> usize>(target: F, allowance: Allowance) {
//         {
//             let index = 0;
//             while let index
//         }
//     }
//
//     // fn test_scenario(allowance: Allowance) {
//     //     allowed_index = mat
//     // }
//
//     #[test]
//     fn next_word() {
//         use super::next_word as uut;
//     }
//
//     #[test]
//     fn previous_word() {}
//
//     #[test]
//     fn previous_word_end() {}
// }
