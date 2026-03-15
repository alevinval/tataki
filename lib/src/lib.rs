mod sequencer;
mod types;

#[cfg(test)]
pub mod test {
    use chrono::DateTime;
    use chrono::Local;
    use chrono::TimeZone;

    // Generate datetime on tests, with less verbosity.
    pub fn d(year: i32, month: u32, day: u32, hour: u32, minute: u32, sec: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, sec)
            .unwrap()
    }
}
