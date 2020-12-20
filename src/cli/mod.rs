// The main function lives in the lib, so it can use private items, but needs
// to be pub as `src/main.rs` can only see items any other consumers of the
// library can.
//
// Point is, this isn't part of the API, so dont depend on it.
#[path = "main.rs"]
#[doc(hidden)]
pub mod __main;

mod args;
mod cmd;
