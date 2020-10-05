pub(super) fn next_word(pivot: usize, string: &str) -> usize {
    let end = string.len();
    if pivot == end {
        pivot
    } else {
        unicode_segmentation::UnicodeSegmentation::split_word_bound_indices(string)
            .find(|pair| pair.0 > pivot)
            .map_or(string.len(), |pair| pair.0)
    }
}

pub(super) fn previous_word(pivot: usize, string: &str) -> usize {
    if pivot == 0 {
        pivot
    } else {
        unicode_segmentation::UnicodeSegmentation::split_word_bound_indices(string)
            .rfind(|pair| pair.0 < pivot)
            .map_or(0, |pair| pair.0)
    }
}

pub(super) fn previous_word_end(pivot: usize, string: &str) -> usize {
    if pivot == 0 {
        pivot
    } else {
        unicode_segmentation::UnicodeSegmentation::split_word_bound_indices(string)
            .rfind(|pair| pair.0 < pivot)
            .map_or(0, |pair| pair.0 + pair.1.len())
    }
}

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
        scenarios: [&'static str; 6],
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
                    "AddZ AdZ  AZ   O AZ  AdZ   AddZ",
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
            let c = string[pivot..].chars().next().unwrap();
            c == 'A' || c == 'O'
        });
    }

    #[test]
    fn previous_word() {
        let tester = Tester::prepare(Direction::Backward);
        tester.test(super::previous_word, |pivot, string| {
            let c = string[pivot..].chars().next().unwrap();
            c == 'A' || c == 'O'
        });
    }

    #[test]
    fn previous_word_end() {
        let tester = Tester::prepare(Direction::Backward);
        tester.test(super::previous_word_end, |pivot, string| {
            let c = string[pivot - 1..].chars().next().unwrap();
            c == 'Z' || c == 'O'
        });
    }
}
