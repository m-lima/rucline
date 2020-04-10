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
