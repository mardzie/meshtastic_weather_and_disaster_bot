type InputTimePeriod = u8;
type MaxTimePeriod = u8;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Status Code: {0}")]
    StatusCode(reqwest::StatusCode),
    #[error("Json Error: {}", 0)]
    Json(#[from] serde_json::Error),
    #[error("Time Period too long: {0} is too long: consider {1} at most.")]
    TooManyRequested(InputTimePeriod, MaxTimePeriod),
}
