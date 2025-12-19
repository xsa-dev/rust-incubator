fn main() {
    let mut s = Solver {
        expected: Trinity { a: 1, b: 2, c: 3 },
        unsolved: vec![
            Trinity { a: 1, b: 2, c: 3 },
            Trinity { a: 2, b: 1, c: 3 },
            Trinity { a: 2, b: 3, c: 1 },
            Trinity { a: 3, b: 1, c: 2 },
        ],
    };
    s.resolve();
    println!("{:?}", s)
}

#[derive(Clone, Debug, PartialEq)]
struct Trinity<T> {
    a: T,
    b: T,
    c: T,
}

impl<T> Trinity<T> {
    fn rotate(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
        std::mem::swap(&mut self.b, &mut self.c);
    }
}

#[derive(Debug)]
struct Solver<T> {
    expected: Trinity<T>,
    unsolved: Vec<Trinity<T>>,
}

impl<T: PartialEq> Solver<T> {
    fn resolve(&mut self) {
        let expected = &self.expected;
        let mut unsolved = std::mem::take(&mut self.unsolved);
        unsolved.retain_mut(|t| {
            for _ in 0..3 {
                if t == expected {
                    return false;
                }
                t.rotate();
            }
            true
        });
        self.unsolved = unsolved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_rotated_matches_from_unsolved() {
        let mut solver = Solver {
            expected: Trinity {
                a: 1,
                b: 2,
                c: 3,
            },
            unsolved: vec![
                Trinity { a: 1, b: 2, c: 3 },
                Trinity { a: 2, b: 3, c: 1 },
                Trinity { a: 3, b: 1, c: 2 },
                Trinity { a: 2, b: 1, c: 3 },
            ],
        };

        solver.resolve();

        assert_eq!(solver.unsolved.len(), 1);
        assert_eq!(
            solver.unsolved[0],
            Trinity {
                a: 2,
                b: 1,
                c: 3
            }
        );
    }

    #[test]
    fn keeps_non_matching_values() {
        let mut solver = Solver {
            expected: Trinity {
                a: 9,
                b: 8,
                c: 7,
            },
            unsolved: vec![
                Trinity { a: 1, b: 2, c: 3 },
                Trinity { a: 2, b: 3, c: 1 },
                Trinity { a: 3, b: 2, c: 1 },
            ],
        };

        solver.resolve();

        assert_eq!(solver.unsolved.len(), 3);
        assert!(solver.unsolved.contains(&Trinity { a: 1, b: 2, c: 3 }));
        assert!(solver.unsolved.contains(&Trinity { a: 2, b: 3, c: 1 }));
        assert!(solver.unsolved.contains(&Trinity { a: 3, b: 2, c: 1 }));
    }
}
