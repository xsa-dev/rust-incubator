fn main() {
    println!("Implement me!");
}

fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
    let mut sign = None;
    let mut width = None;
    let mut precision = None;

    let dot_index = input.find('.');
    let (before_dot, after_dot) = match dot_index {
        Some(idx) => (&input[..idx], &input[idx + 1..]),
        None => (input, ""),
    };

    for ch in before_dot.chars() {
        if ch == '+' {
            sign = Some(Sign::Plus);
            break;
        } else if ch == '-' {
            sign = Some(Sign::Minus);
            break;
        }
    }

    if let Some(digits) = before_dot
        .chars()
        .rev()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>()
        .parse::<usize>()
        .ok()
    {
        width = Some(digits);
    }

    if !after_dot.is_empty() {
        if let Some(rest) = after_dot.strip_prefix('*') {
            precision = Some(Precision::Asterisk);
            let _ = rest;
        } else if let Some((digits, _)) = after_dot.split_once('$') {
            if let Ok(value) = digits.parse::<usize>() {
                precision = Some(Precision::Argument(value));
            }
        } else {
            let digits: String = after_dot
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                if let Ok(value) = digits.parse::<usize>() {
                    precision = Some(Precision::Integer(value));
                }
            }
        }
    }

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
        ] {
            let (_, width, _) = parse(input);
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
        ] {
            let (_, _, precision) = parse(input);
            assert_eq!(precision, expected);
        }
    }
}
