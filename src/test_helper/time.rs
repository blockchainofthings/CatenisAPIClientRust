use std::{
    sync::Mutex,
    thread::{
        self, ThreadId,
    },
    collections::HashMap,
};
use once_cell::sync::Lazy;
use time::OffsetDateTime;

static CUSTOM_TIME: Lazy<Mutex<HashMap<ThreadId, i64>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn now() -> OffsetDateTime {
    let custom_time = CUSTOM_TIME.lock().unwrap();
    let thread_id = thread::current().id();

    if custom_time.contains_key(&thread_id) {
        OffsetDateTime::from_unix_timestamp(*custom_time.get(&thread_id).unwrap())
    } else {
        OffsetDateTime::now_utc()
    }
}

pub struct CustomTime();

impl CustomTime {
    pub fn set(time: &OffsetDateTime) -> Self {
        let mut custom_time = CUSTOM_TIME.lock().unwrap();

        custom_time.insert(thread::current().id(), time.unix_timestamp());

        CustomTime()
    }
}

impl Drop for CustomTime {
    fn drop(&mut self) {
        let mut custom_time = CUSTOM_TIME.lock().unwrap();

        custom_time.remove(&thread::current().id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn it_get_system_time() {
        let sys_now = OffsetDateTime::now_utc();
        let test_now = now();

        assert!(test_now >= sys_now, "Current test time less than current system time");
        assert!((test_now - sys_now) < std::time::Duration::from_millis(10), "Current test time not within 10 ms from current system time");
    }

    #[test]
    fn it_set_custom_time() {
        let ref_time = time::date!(2020-12-01).with_time(time::time!(06:00:00)).assume_utc();

        let _custom_time = CustomTime::set(&ref_time);

        assert_eq!(now(), ref_time);
    }

    #[test]
    fn it_set_custom_time_thread() {
        let child_thread = thread::spawn(|| {
            let ref_time = time::date!(2020-12-05).with_time(time::time!(08:10:05)).assume_utc();

            let _custom_time = CustomTime::set(&ref_time);

            assert_eq!(now(), ref_time);
        });

        let sys_now = OffsetDateTime::now_utc();
        let test_now = now();

        assert!(test_now >= sys_now, "Current test time less than current system time");
        assert!((test_now - sys_now) < std::time::Duration::from_millis(10), "Current test time not within 10 ms from current system time");

        child_thread.join().unwrap();
    }

    #[test]
    fn it_set_custom_time_two_threads() {
        let child_thread_1 = thread::spawn(|| {
            let ref_time = time::date!(2020-12-10).with_time(time::time!(10:20:10)).assume_utc();

            let _custom_time = CustomTime::set(&ref_time);

            assert_eq!(now(), ref_time);
        });

        let child_thread_2 = thread::spawn(|| {
            let ref_time = time::date!(2020-12-15).with_time(time::time!(12:40:15)).assume_utc();

            let _custom_time = CustomTime::set(&ref_time);

            assert_eq!(now(), ref_time);
        });

        let sys_now = OffsetDateTime::now_utc();
        let test_now = now();

        assert!(test_now >= sys_now, "Current test time less than current system time");
        assert!((test_now - sys_now) < std::time::Duration::from_millis(10), "Current test time not within 10 ms from current system time");

        child_thread_1.join().unwrap();
        child_thread_2.join().unwrap();
    }
}