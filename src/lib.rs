//! A library for handling debian files and other tasks related to the debian package
//! repository. For extracting and retrieving data from `.deb` files please go to
//! <file::Deb>.
//!
//! Using this library requires a few common linux packages installed. These are
//! `tar`, `mkdir`, and `ar` (part of `binutils`). Please note there are some issues
//! with `ar` on ubuntu older than `20.04`

// Shared libraries
pub mod shared;

// File extraction
pub mod file;
