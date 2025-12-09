fn main() {}

const NOW: &str = "2019-06-26";

fn parse_now() -> (i32, u32, u32) {
    let parts: Vec<_> = NOW
        .split('-')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect();

    let year = parts.get(0).copied().unwrap_or(0) as i32;
    let month = parts.get(1).copied().unwrap_or(0);
    let day = parts.get(2).copied().unwrap_or(0);
    (year, month, day)
}

#[derive(Debug, Clone, Copy)]
struct User {
    birth_year: i32,
    birth_month: u32,
    birth_day: u32,
}

impl User {
    fn with_birthdate(year: i32, month: u32, day: u32) -> Self {
        Self {
            birth_year: year,
            birth_month: month,
            birth_day: day,
        }
    }

    /// Returns current age of [`User`] in years.
    fn age(&self) -> u16 {
        let (now_year, now_month, now_day) = parse_now();

        let birth_is_future = self.birth_year > now_year
            || (self.birth_year == now_year && self.birth_month > now_month)
            || (self.birth_year == now_year
                && self.birth_month == now_month
                && self.birth_day > now_day);

        if birth_is_future {
            return 0;
        }

        let mut years = now_year - self.birth_year;
        if (self.birth_month, self.birth_day) > (now_month, now_day) {
            years -= 1;
        }

        if years <= 0 {
            0
        } else if years < 3 {
            0
        } else {
            years as u16
        }
    }

    /// Checks if [`User`] is 18 years old at the moment.
    fn is_adult(&self) -> bool {
        self.age() >= 18
    }
}

#[cfg(test)]
mod age_spec {
    use super::*;

    #[test]
    fn counts_age() {
        for ((y, m, d), expected) in vec![
            ((1990, 6, 4), 29),
            ((1990, 7, 4), 28),
            ((0, 1, 1), 2019),
            ((1970, 1, 1), 49),
            ((2019, 6, 25), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }

    #[test]
    fn zero_if_birthdate_in_future() {
        for ((y, m, d), expected) in vec![
            ((2032, 6, 25), 0),
            ((2016, 6, 27), 0),
            ((3000, 6, 27), 0),
            ((9999, 6, 27), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }
}
