use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    println!("Use `parse` or `parse_with_regex` from tests");
}

fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
    parse_manual(input)
}

fn parse_manual(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    if chars
        .get(index + 1)
        .is_some_and(|c| matches!(c, '<' | '^' | '>'))
    {
        index += 2;
    } else if chars
        .get(index)
        .is_some_and(|c| matches!(c, '<' | '^' | '>'))
    {
        index += 1;
    }

    let sign = chars.get(index).and_then(|c| match c {
        '+' => {
            index += 1;
            Some(Sign::Plus)
        }
        '-' => {
            index += 1;
            Some(Sign::Minus)
        }
        _ => None,
    });

    if chars.get(index) == Some(&'#') {
        index += 1;
    }

    if chars.get(index) == Some(&'0') {
        index += 1;
    }

    let width = {
        let start = index;
        while chars.get(index).is_some_and(|c| c.is_ascii_digit()) {
            index += 1;
        }

        if start == index {
            None
        } else {
            let value = chars[start..index].iter().collect::<String>().parse().ok();

            if chars.get(index) == Some(&'$') {
                index += 1;
            }

            value
        }
    };

    let precision = if chars.get(index) == Some(&'.') {
        index += 1;
        match chars.get(index) {
            Some('*') => {
                index += 1;
                Some(Precision::Asterisk)
            }
            Some(c) if c.is_ascii_digit() => {
                let start = index;
                while chars.get(index).is_some_and(|c| c.is_ascii_digit()) {
                    index += 1;
                }

                let digits: Option<usize> =
                    chars[start..index].iter().collect::<String>().parse().ok();

                let result = if chars.get(index) == Some(&'$') {
                    index += 1;
                    digits.map(Precision::Argument)
                } else {
                    digits.map(Precision::Integer)
                };

                result
            }
            _ => None,
        }
    } else {
        None
    };

    (sign, width, precision)
}

fn parse_with_regex(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
    static FORMAT_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?:.?[<^>])?(?P<sign>[+-])?#?0?(?P<width>\d+(?:\$)?)?(?P<precision>\.(?:\d+(?:\$)?|\*))?")
            .expect("valid regex")
    });

    let captures = FORMAT_RE.captures(input);

    let sign = captures
        .as_ref()
        .and_then(|caps| caps.name("sign"))
        .and_then(|m| match m.as_str() {
            "+" => Some(Sign::Plus),
            "-" => Some(Sign::Minus),
            _ => None,
        });

    let width = captures
        .as_ref()
        .and_then(|caps| caps.name("width"))
        .and_then(|m| m.as_str().trim_end_matches('$').parse().ok());

    let precision = captures
        .as_ref()
        .and_then(|caps| caps.name("precision"))
        .and_then(|m| match &m.as_str()[1..] {
            "*" => Some(Precision::Asterisk),
            value => {
                let trimmed = value.trim_end_matches('$');
                trimmed.parse().ok().map(|number| {
                    if value.ends_with('$') {
                        Precision::Argument(number)
                    } else {
                        Precision::Integer(number)
                    }
                })
            }
        });

    (sign, width, precision)
}

#[derive(Debug, PartialEq)]
enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
enum Precision {
    Integer(usize),
    Argument(usize),
    Asterisk,
}

#[cfg(test)]
mod spec {
    use super::*;

    #[test]
    fn parses_sign() {
        for (input, expected) in vec![
            ("", None),
            (">8.*", None),
            (">+8.*", Some(Sign::Plus)),
            ("-.1$x", Some(Sign::Minus)),
            ("a^#043.8?", None),
        ] {
            let (sign, ..) = parse(input);
            assert_eq!(sign, expected);
            let (sign, ..) = parse_with_regex(input);
            assert_eq!(sign, expected);
        }
    }

    #[test]
    fn parses_width() {
        for (input, expected) in vec![
            ("", None),
            (">8.*", Some(8)),
            (">+8.*", Some(8)),
            ("-.1$x", None),
            ("a^#043.8?", Some(43)),
            ("+1$?", Some(1)),
        ] {
            let (_, width, _) = parse(input);
            assert_eq!(width, expected);
            let (_, width, _) = parse_with_regex(input);
            assert_eq!(width, expected);
        }
    }

    #[test]
    fn parses_precision() {
        for (input, expected) in vec![
            ("", None),
            (">8.*", Some(Precision::Asterisk)),
            (">+8.*", Some(Precision::Asterisk)),
            ("-.1$x", Some(Precision::Argument(1))),
            ("a^#043.8?", Some(Precision::Integer(8))),
            ("+1$.2$", Some(Precision::Argument(2))),
        ] {
            let (_, _, precision) = parse(input);
            assert_eq!(precision, expected);
            let (_, _, precision) = parse_with_regex(input);
            assert_eq!(precision, expected);
        }
    }
}
