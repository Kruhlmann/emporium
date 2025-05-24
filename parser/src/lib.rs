use once_cell::sync::Lazy;
use tokio::sync::Semaphore;

pub static HTTP_REQUEST_THROTTLE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(8));

pub mod v2_0_0;
