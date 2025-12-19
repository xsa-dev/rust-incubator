fn main() {}

const NOW: &str = "2019-06-26";

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
        let (now_year, now_month, now_day) = now_components();

        if self.birth_year > now_year
            || (self.birth_year == now_year
                && (self.birth_month > now_month
                    || (self.birth_month == now_month && self.birth_day > now_day)))
        {
            return 0;
        }

        let mut age = now_year - self.birth_year;

        if self.birth_month > now_month
            || (self.birth_month == now_month && self.birth_day > now_day)
        {
            age -= 1;
        }

        age as u16
    }

    /// Checks if [`User`] is 18 years old at the moment.
    fn is_adult(&self) -> bool {
        self.age() >= 18
    }
}

fn now_components() -> (i32, u32, u32) {
    let mut parts = NOW.split('-');
    let year = parts
        .next()
        .and_then(|part| part.parse::<i32>().ok())
        .expect("failed to parse year from NOW");
    let month = parts
        .next()
        .and_then(|part| part.parse::<u32>().ok())
        .expect("failed to parse month from NOW");
    let day = parts
        .next()
        .and_then(|part| part.parse::<u32>().ok())
        .expect("failed to parse day from NOW");
    (year, month, day)
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
            ((2020, 6, 27), 0),
            ((3000, 6, 27), 0),
            ((9999, 6, 27), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }

    #[test]
    fn checks_adulthood() {
        for ((y, m, d), expected) in vec![
            ((2000, 6, 26), true),   // exactly 19 years old
            ((2001, 6, 26), true),   // turned 18 today
            ((2001, 6, 27), false),  // birthday tomorrow
            ((2010, 1, 1), false),   // clearly underage
            ((2030, 1, 1), false),   // future date should be treated as not adult
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.is_adult(), expected);
        }
    }
}
