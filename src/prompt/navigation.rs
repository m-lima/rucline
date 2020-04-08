pub(super) fn next_word(pivot: usize, string: &[char]) -> usize {
    let end = string.len();
    if pivot == end {
        pivot
    } else {
        let mut index = pivot;

        while index < end && !string[index].is_whitespace() {
            index += 1;
        }

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

        while index > 0 && string[index].is_whitespace() {
            index -= 1;
        }

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

        while index > 0 && !string[index].is_whitespace() {
            index -= 1;
        }

        while index > 0 && string[index - 1].is_whitespace() {
            index -= 1;
        }

        index
    }
}
