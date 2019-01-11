//! # SD3
//! Code for serializing, handling, and deserializing SD3 data 
mod mifc;

pub use crate::mifc::Mifc as Mifc;
pub use crate::mifc::MifcNorm as MifcNorm;
pub use crate::mifc::MifcNormError as MifcNormError;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
