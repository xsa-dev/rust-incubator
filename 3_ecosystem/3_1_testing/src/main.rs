use std::{
    cmp::Ordering,
    env,
    io::{self, BufRead},
};

fn main() {
    println!("Guess the number!");

    let secret_number = get_secret_number();

    loop {
        println!("Please input your guess.");

        let guess = match get_guess_number() {
            Some(n) => n,
            _ => continue,
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}

fn get_secret_number() -> u32 {
    match parse_secret_number(env::args()) {
        Ok(number) => number,
        Err(SecretNumberError::Missing) => panic!("No secret number is specified"),
        Err(SecretNumberError::NotANumber) => panic!("Secret number is not a number"),
    }
}

fn get_guess_number() -> Option<u32> {
    read_guess(&mut io::stdin().lock()).expect("Failed to read line")
}

#[derive(Debug, PartialEq, Eq)]
enum SecretNumberError {
    Missing,
    NotANumber,
}

fn parse_secret_number<I>(mut args: I) -> Result<u32, SecretNumberError>
where
    I: Iterator<Item = String>,
{
    args.next();

    let secret = args.next().ok_or(SecretNumberError::Missing)?;
    secret
        .trim()
        .parse::<u32>()
        .map_err(|_| SecretNumberError::NotANumber)
}

fn read_guess<R>(reader: &mut R) -> io::Result<Option<u32>>
where
    R: BufRead,
{
    let mut guess = String::new();
    reader.read_line(&mut guess)?;
    Ok(guess.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn parses_secret_number_from_args() {
        let args = vec!["step".into(), "123".into()];
        assert_eq!(parse_secret_number(args.into_iter()), Ok(123));
    }

    #[test]
    fn reports_missing_secret_number() {
        let args = vec!["step".into()];
        assert_eq!(
            parse_secret_number(args.into_iter()),
            Err(SecretNumberError::Missing)
        );
    }

    #[test]
    fn reports_invalid_secret_number() {
        let args = vec!["step".into(), "not-a-number".into()];
        assert_eq!(
            parse_secret_number(args.into_iter()),
            Err(SecretNumberError::NotANumber)
        );
    }

    #[test]
    fn reads_guess_from_buffer() {
        let mut input = Cursor::new(b"56\n".as_slice());
        assert_eq!(read_guess(&mut input).unwrap(), Some(56));
    }

    #[test]
    fn returns_none_for_invalid_guess() {
        let mut input = Cursor::new(b"abc\n".as_slice());
        assert_eq!(read_guess(&mut input).unwrap(), None);
    }

    #[test]
    fn propagates_io_errors_from_reader() {
        struct FailingReader;

        impl io::Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "failure"))
            }
        }

        impl BufRead for FailingReader {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Err(io::Error::new(io::ErrorKind::Other, "failure"))
            }

            fn consume(&mut self, _: usize) {}
        }

        let mut reader = FailingReader;
        let err = read_guess(&mut reader).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
    }
}
