//! # SD3
//! Code for serializing, handling, and deserializing the SD3 family
//! of TCTC data formats 
mod mifc;
mod cmpd;

pub use crate::mifc::Mifc as Mifc;
pub use crate::mifc::MifcNorm as MifcNorm;
pub use crate::mifc::MifcNormError as MifcNormError;
pub use crate::cmpd::CmpdDit as CmpdDit;
