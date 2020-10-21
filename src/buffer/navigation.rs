pub(super) fn next_scalar_value(index: usize, string: &str) -> usize {
    if let Some((offset, _)) = string[index..].char_indices().nth(1) {
        index + offset
    } else {
        string.len()
    }
}

pub(super) fn previous_scalar_value(index: usize, string: &str) -> usize {
    if let Some((new_index, _)) = string[..index].char_indices().next_back() {
        new_index
    } else {
        0
    }
}

pub(super) fn next_word(pivot: usize, string: &str) -> usize {
    let end = string.len();
    if pivot == end {
        pivot
    } else {
        unicode_segmentation::UnicodeSegmentation::split_word_bound_indices(string)
            .find(|pair| {
                pair.0 > pivot && pair.1.chars().next().map_or(true, |c| !c.is_whitespace())
            })
            .map_or(string.len(), |pair| pair.0)
    }
}

pub(super) fn previous_word(pivot: usize, string: &str) -> usize {
    if pivot == 0 {
        pivot
    } else {
        unicode_segmentation::UnicodeSegmentation::split_word_bound_indices(string)
            .rfind(|pair| {
                pair.0 < pivot && pair.1.chars().next().map_or(true, |c| !c.is_whitespace())
            })
            .map_or(0, |pair| pair.0)
    }
}

pub(super) fn previous_word_end(pivot: usize, string: &str) -> usize {
    if pivot == 0 {
        pivot
    } else {
        unicode_segmentation::UnicodeSegmentation::split_word_bound_indices(string)
            .rfind(|pair| {
                pair.0 + pair.1.len() < pivot
                    && pair.1.chars().next().map_or(true, |c| !c.is_whitespace())
            })
            .map_or(0, |pair| pair.0 + pair.1.len())
    }
}

// Allowed because it makes test clearer
#[allow(clippy::non_ascii_literal)]
#[cfg(test)]
mod test {
    #[derive(Copy, Clone)]
    enum Direction {
        Forward,
        Backward,
    }

    impl Direction {
        pub(super) fn start_for(self, scenario: &str) -> usize {
            match self {
                Direction::Forward => 0,
                Direction::Backward => scenario.len(),
            }
        }
    }

    struct Tester {
        direction: Direction,
        scenarios: [&'static str; 10],
    }

    impl Tester {
        pub(super) fn prepare(direction: Direction) -> Self {
            Self {
                direction,
                scenarios: [
                    "",
                    "   \t   ",
                    "AddZ   \t   ",
                    "   \t   AddZ",
                    "   \t   AddZ   \t   ",
                    "AddZ AdZ  AZ \t  O AZ  AdZ   AddZ",
                    "AddZ AdZ  AZ \t 😀 AZ  AdZ   AddZ",
                    "AddZ AdZ  AZ😀AZ \t  O AZ  AdZ   AddZ",
                    "AddZ AdZ  AZ \t 🇧🇷 AZ  AdZ   AddZ",
                    "AddZ AdZ  AZ🇧🇷AZ \t  O AZ  AdZ   AddZ",
                ],
            }
        }

        pub(super) fn test<F, V>(&self, uut: F, validator: V)
        where
            F: Fn(usize, &str) -> usize,
            V: Fn(usize, &str) -> bool,
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
        scenario: &str,
        start: usize,
        iteration: usize,
    ) where
        F: Fn(usize, &str) -> usize,
        V: Fn(usize, &str) -> bool,
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
            "failed on iteration {} at index {} for \"{}\"",
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
            let c = string[pivot..].chars().next().unwrap();
            c == 'A' || c == 'O' || c == '😀' || c == '🇧'
        });
    }

    #[test]
    fn previous_word() {
        let tester = Tester::prepare(Direction::Backward);
        tester.test(super::previous_word, |pivot, string| {
            let c = string[pivot..].chars().next().unwrap();
            c == 'A' || c == 'O' || c == '😀' || c == '🇧'
        });
    }

    #[test]
    fn previous_word_end() {
        let tester = Tester::prepare(Direction::Backward);
        tester.test(super::previous_word_end, |pivot, string| {
            let c = string[..pivot].chars().next_back().unwrap();
            c == 'Z' || c == 'O' || c == '😀' || c == '🇷'
        });
    }

    #[test]
    fn within_multiple_unicode_scalar_values() {
        let string = "ab 🇧🇷 cd";
        let pivot = "ab 🇧🇷".len() - 4;

        assert_eq!(super::next_word(pivot, string), "ab 🇧🇷 ".len());
        assert_eq!(super::previous_word(pivot, string), "ab ".len());
        assert_eq!(super::previous_word_end(pivot, string), "ab".len());
    }

    #[test]
    fn next_scalar_value() {
        use super::next_scalar_value;

        let string = "a😀øé🇧🇷";
        let mut index = 0;
        index = next_scalar_value(index, string);
        assert_eq!(string[index..].chars().next().unwrap(), '😀');
        index = next_scalar_value(index, string);
        assert_eq!(string[index..].chars().next().unwrap(), 'ø');
        index = next_scalar_value(index, string);
        assert_eq!(string[index..].chars().next().unwrap(), 'é');
        index = next_scalar_value(index, string);
        assert_eq!(
            string[index..].chars().next().unwrap(),
            "🇧🇷".chars().next().unwrap()
        );
        index = next_scalar_value(index, string);
        assert_eq!(
            string[index..].chars().next().unwrap(),
            "🇧🇷".chars().nth(1).unwrap()
        );
        index = next_scalar_value(index, string);
        assert_eq!(index, string.len());
        index = next_scalar_value(index, string);
        assert_eq!(index, string.len());
    }

    #[test]
    fn previous_scalar_value() {
        use super::previous_scalar_value;

        let string = "a😀øé🇧🇷";
        let mut index = string.len();
        index = previous_scalar_value(index, string);
        assert_eq!(
            string[index..].chars().next().unwrap(),
            "🇧🇷".chars().nth(1).unwrap()
        );
        index = previous_scalar_value(index, string);
        assert_eq!(
            string[index..].chars().next().unwrap(),
            "🇧🇷".chars().next().unwrap()
        );
        index = previous_scalar_value(index, string);
        assert_eq!(string[index..].chars().next().unwrap(), 'é');
        index = previous_scalar_value(index, string);
        assert_eq!(string[index..].chars().next().unwrap(), 'ø');
        index = previous_scalar_value(index, string);
        assert_eq!(string[index..].chars().next().unwrap(), '😀');
        index = previous_scalar_value(index, string);
        assert_eq!(index, 0);
        assert_eq!(string[index..].chars().next().unwrap(), 'a');
        index = previous_scalar_value(index, string);
        assert_eq!(index, 0);
    }
}
