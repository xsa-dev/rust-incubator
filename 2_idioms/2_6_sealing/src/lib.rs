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
