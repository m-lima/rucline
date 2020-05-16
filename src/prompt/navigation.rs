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

#[cfg(test)]
mod test {
    use crate::prompt::char_string::CharString;

    #[derive(Copy, Clone)]
    enum Direction {
        Forward,
        Backward,
    }

    impl Direction {
        pub(super) fn start_for(self, scenario: &CharString) -> usize {
            match self {
                Direction::Forward => 0,
                Direction::Backward => scenario.len(),
            }
        }
    }

    struct Tester {
        direction: Direction,
        scenarios: [CharString; 6],
    }

    impl Tester {
        pub(super) fn prepare(direction: Direction) -> Self {
            Self {
                direction,
                scenarios: [
                    CharString::new(),
                    CharString::from("   \t   "),
                    CharString::from("AddZ   \t   "),
                    CharString::from("   \t   AddZ"),
                    CharString::from("   \t   AddZ   \t   "),
                    CharString::from("AddZ AdZ  AZ   O AZ  AdZ   AddZ"),
                ],
            }
        }

        pub(super) fn test<F, V>(&self, uut: F, validator: V)
        where
            F: Fn(usize, &[char]) -> usize,
            V: Fn(usize, &[char]) -> bool,
        {
            for scenario in &self.scenarios {
                test_scenario(
                    &uut,
                    &validator,
                    self.direction,
                    &scenario,
                    self.direction.start_for(&scenario),
                    0,
                )
            }
        }
    }

    fn test_scenario<F, V>(
        uut: F,
        validator: V,
        direction: Direction,
        scenario: &CharString,
        start: usize,
        iteration: usize,
    ) where
        F: Fn(usize, &[char]) -> usize,
        V: Fn(usize, &[char]) -> bool,
    {
        let pivot = uut(start, scenario);

        match direction {
            Direction::Forward => {
                if pivot == scenario.len() {
                    return;
                }
                assert!(pivot > start);
            }
            Direction::Backward => {
                if pivot == 0 {
                    return;
                }
                assert!(pivot < start);
            }
        }

        assert!(
            validator(pivot, scenario),
            "failed on iteration {} at index {} for {}",
            iteration,
            pivot,
            scenario
        );
        test_scenario(uut, validator, direction, scenario, pivot, iteration + 1);
    }

    #[test]
    fn next_word() {
        let tester = Tester::prepare(Direction::Forward);
        tester.test(super::next_word, |pivot, string| {
            string[pivot] == 'A' || string[pivot] == 'O'
        });
    }

    #[test]
    fn previous_word() {
        let tester = Tester::prepare(Direction::Backward);
        tester.test(super::previous_word, |pivot, string| {
            string[pivot] == 'A' || string[pivot] == 'O'
        });
    }

    #[test]
    fn previous_word_end() {
        let tester = Tester::prepare(Direction::Backward);
        tester.test(super::previous_word_end, |pivot, string| {
            string[pivot - 1] == 'Z' || string[pivot - 1] == 'O'
        });
    }
}
