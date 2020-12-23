#[cfg(test)]
macro_rules! now {
    () => { test_helper::time::now() }
}
#[cfg(not(test))]
macro_rules! now {
    () => { time::OffsetDateTime::now_utc() }
}