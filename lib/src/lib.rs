mod scheduler;
mod sequencer;
mod storage;
mod types;

pub use storage::StorageError;
pub use storage::Store;

#[cfg(test)]
pub mod test {
    use std::path::PathBuf;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;

    use chrono::DateTime;
    use chrono::Local;
    use chrono::TimeZone;

    // Generate datetime on tests, with less verbosity.
    pub fn d(year: i32, month: u32, day: u32, hour: u32, minute: u32, sec: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, sec)
            .unwrap()
    }

    // Create a fresh temp directory for tests that need the filesystem.
    pub fn dir(name: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir()
            .join(format!("tataki_test_{}", std::process::id()))
            .join(format!(
                "{}_{}",
                name,
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
