//! # SD3
//! Code for serializing, handling, and deserializing the SD3 family
//! of TCTC data formats 
mod mifc;

pub use crate::mifc::Mifc as Mifc;
pub use crate::mifc::MifcNorm as MifcNorm;
pub use crate::mifc::MifcNormError as MifcNormError;
