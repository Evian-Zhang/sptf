use std::time::Duration;

pub const PROTOCOL_VERSION: u32 = 1;
pub const FILEWATCHER_DEBOUNCE_DURATION: Duration = Duration::from_secs(1);
pub const MAX_FILE_UPLOAD_SIZE: usize = 64 * 1024 * 1024;
