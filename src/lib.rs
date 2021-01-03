pub mod deb;
pub mod extractor;

pub use deb::Deb;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
