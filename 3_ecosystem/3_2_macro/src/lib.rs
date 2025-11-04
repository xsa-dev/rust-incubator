//! Helper macros for building [`BTreeMap`](std::collections::BTreeMap) values.
//!
//! The crate exposes two variants of the `btreemap!` macro:
//! - [`btreemap`] – implemented using `macro_rules!`.
//! - [`proc_btreemap`] – implemented as a procedural macro located in the
//!   companion [`btreemap_proc_macro`] crate.
//!
//! Both macros accept the same syntax and return a populated
//! [`BTreeMap`](std::collections::BTreeMap) instance.

/// Declarative implementation of the [`btreemap!`] macro.
#[macro_export]
macro_rules! btreemap {
    () => {
        ::std::collections::BTreeMap::new()
    };
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        let mut map = ::std::collections::BTreeMap::new();
        $(
            map.insert($key, $value);
        )+
        map
    }};
}

pub use btreemap_proc_macro::btreemap as proc_btreemap;

#[cfg(test)]
mod tests {
    use super::proc_btreemap;
    use std::collections::BTreeMap;

    #[test]
    fn declarative_macro_builds_map() {
        let map = btreemap! {
            "a" => 1,
            "b" => 2,
        };

        let mut expected = BTreeMap::new();
        expected.insert("a", 1);
        expected.insert("b", 2);

        assert_eq!(map, expected);
    }

    #[test]
    fn procedural_macro_builds_map() {
        let map = proc_btreemap! {
            "a" => 1,
            "b" => 2,
        };

        let mut expected = BTreeMap::new();
        expected.insert("a", 1);
        expected.insert("b", 2);

        assert_eq!(map, expected);
    }
}
