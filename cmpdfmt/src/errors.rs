use failure::Error;
use log::{error, warn};

// TODO: make this its own crate
// TODO: add these to sd3norm main.rs

pub fn error_chain(e: &Error) {
    error!("{}", &e);
    for cause in e.iter_causes() {
        error!("caused by: {}", cause);
    }
    match ::std::env::var("RUST_BACKTRACE").as_ref().map(|s| s.as_str()) {
        Ok("1") => error!("Backtrace:\n{}", e.backtrace()),
        _ => (),
    }
}

pub fn warn_chain(e: &Error){
    warn!("{}", &e);
    for cause in e.iter_causes() {
        warn!("caused by: {}", cause);
    }
    match ::std::env::var("RUST_BACKTRACE").as_ref().map(|s| s.as_str()) {
        Ok("1") => warn!("Backtrace:\n{}", e.backtrace()),
        _ => (),
    }
}
