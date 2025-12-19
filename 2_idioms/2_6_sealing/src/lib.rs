pub mod my_error;
pub mod my_iterator_ext;

pub use self::{my_error::MyError, my_iterator_ext::MyIteratorExt};

// The trait below is sealed and cannot be implemented here.
// struct LocalIterator;
//
// impl Iterator for LocalIterator {
//     type Item = ();
//
//     fn next(&mut self) -> Option<Self::Item> {
//         None
//     }
// }
//
// impl MyIteratorExt for LocalIterator {}

// The hidden method of this trait is sealed and cannot be overridden here.
// use std::fmt;
//
// #[derive(Debug)]
// struct LocalError;
//
// impl fmt::Display for LocalError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "local error")
//     }
// }
//
// impl MyError for LocalError {
//     fn type_id(&self) -> std::any::TypeId
//     where
//         Self: 'static,
//     {
//         std::any::TypeId::of::<()>()
//     }
// }

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::my_error::MyError;
    use super::my_iterator_ext::MyIteratorExt as _;

    #[test]
    fn formats_iterator_items_with_separator() {
        let values = [1, 2, 3];
        let formatted = format!("{}", values.iter().format(", "));

        assert_eq!(formatted, "1, 2, 3");
    }

    #[test]
    fn my_error_defaults_to_no_source() {
        #[derive(Debug)]
        struct SimpleError;

        impl fmt::Display for SimpleError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "simple error")
            }
        }

        impl MyError for SimpleError {}

        let err = SimpleError;
        assert!(err.source().is_none());
        assert_eq!(err.to_string(), "simple error");
    }
}
