use lazy_static::lazy_static;
use std::time::Duration;

pub mod dbl;
pub mod godbolt;
pub mod insights;
pub mod quick_link;
pub mod wandbox;

lazy_static! {
    /// Shared client for all outbound API traffic
    pub static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Unable to build shared http client");
}
