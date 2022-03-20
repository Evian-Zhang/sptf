use std::time::Duration;

pub const PROTOCOL_VERSION: u32 = 1;
pub const FILEWATCHER_DEBOUNCE_DURATION: Duration = Duration::from_secs(1);
pub const MAX_FILE_UPLOAD_SIZE: usize = 64 * 1024 * 1024;
pub const COOKIE_AUTH_TOKEN_NAME: &str = "SPTF_AUTH";
/// Redis cache expires in 30 mins
pub const REDIS_CACHE_EXPIRATION_IN_SECONDS: usize = 30 * 60;
