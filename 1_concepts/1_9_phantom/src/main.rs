use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

/// Marker-based provider of facts about type `T`.
///
/// The only owned value is [`PhantomData`], so the semantics are encoded at the
/// type level while the value is fully optimized out.
pub struct Fact<T> {
    _marker: PhantomData<T>,
}

impl<T> Fact<T> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

/// Types that can provide a list of facts about themselves.
pub trait TypeFacts {
    /// Returns a static list of facts for the type.
    fn facts() -> &'static [&'static str];
}

impl<T> TypeFacts for Vec<T> {
    fn facts() -> &'static [&'static str] {
        &[
            "Vec is heap-allocated.",
            "Vec may re-allocate on growing.",
            "Vec provides amortized O(1) push to the end.",
        ]
    }
}

impl TypeFacts for String {
    fn facts() -> &'static [&'static str] {
        &[
            "String is a UTF-8 owned buffer.",
            "String is growable and heap-allocated.",
            "String dereferences to &str for convenience.",
        ]
    }
}

impl TypeFacts for i32 {
    fn facts() -> &'static [&'static str] {
        &[
            "i32 is a 32-bit signed integer.",
            "i32 implements Copy and is stored on the stack.",
        ]
    }
}

impl<T> Fact<T>
where
    T: TypeFacts,
{
    /// Returns a pseudo-random fact for `T` using only the type-level marker.
    pub fn fact(&self) -> &'static str {
        let facts = T::facts();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let idx = (now as usize) % facts.len();
        facts[idx]
    }
}

fn main() {
    let vec_fact: Fact<Vec<u8>> = Fact::new();
    println!("Fact about Vec: {}", vec_fact.fact());

    let string_fact: Fact<String> = Fact::new();
    println!("Fact about String: {}", string_fact.fact());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provides_fact_for_vec() {
        let fact: Fact<Vec<i32>> = Fact::new();
        let result = fact.fact();
        assert!(
            Vec::<i32>::facts().contains(&result),
            "returned fact should belong to Vec facts list"
        );
    }

    #[test]
    fn provides_fact_for_string() {
        let fact: Fact<String> = Fact::new();
        let result = fact.fact();
        assert!(
            String::facts().contains(&result),
            "returned fact should belong to String facts list"
        );
    }

    #[test]
    fn provides_fact_for_i32() {
        let fact: Fact<i32> = Fact::new();
        let result = fact.fact();
        assert!(
            i32::facts().contains(&result),
            "returned fact should belong to i32 facts list"
        );
    }
}
