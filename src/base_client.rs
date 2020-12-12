use std::{
    borrow::Borrow,
};
use bitcoin_hashes::{
    Hash, HashEngine, Hmac,
    HmacEngine,
    sha256,
};
use reqwest::{
    Url,
};
use time::{
    Date, OffsetDateTime, Duration,
};

const SIGNATURE_VALIDITY_DAYS: u8 = 7;
const TIME_VARIATION_SECS: u8 = 5;

pub(crate) trait BaseCatenisClient {
    fn get_api_access_secret_ref(&self) -> &String;
    fn get_sign_date_ref(&self) -> &Option<Date>;
    fn get_sign_date_mut_ref(&mut self) -> &mut Option<Date>;
    fn get_signing_key_mut_ref(&mut self) -> &mut Option<[u8; 32]>;

    fn check_update_sign_date_and_key(&mut self, now: &OffsetDateTime) {
        let sign_date = self.get_sign_date_ref();

        let need_to_update = if let None = sign_date {
            true
        } else {
            let lower_bound_sign_date = (now.clone() + Duration::seconds(TIME_VARIATION_SECS as i64)).date() - Duration::days(SIGNATURE_VALIDITY_DAYS as i64);

            if sign_date.unwrap() < lower_bound_sign_date {
                true
            } else {
                false
            }
        };

        if need_to_update {
            *self.get_sign_date_mut_ref() = Some(now.date());

            // Generate new signing key
            let inner_key = String::from("CTN1") + self.get_api_access_secret_ref().as_str();
            let mut hmac_engine = HmacEngine::<sha256::Hash>::new(inner_key.as_bytes());
            hmac_engine.input(self.get_sign_date_ref().unwrap().format("%Y%m%d").as_bytes());
            let date_key = &Hmac::<sha256::Hash>::from_engine(hmac_engine)[..];

            let mut hmac_engine = HmacEngine::<sha256::Hash>::new(date_key);
            hmac_engine.input(b"ctn1_request");

            *self.get_signing_key_mut_ref() = Some(*Hmac::<sha256::Hash>::from_engine(hmac_engine).as_inner());
        }
    }

    fn merge_url_params<I, K, V>(url_path: &str, params: I) -> String
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>
    {
        let mut merged_url_path = String::from(url_path);

        for pair in params.into_iter() {
            let param = pair.borrow().0.as_ref();
            let value = pair.borrow().1.as_ref();
            merged_url_path = merged_url_path.replace(&(String::from(":") + param), value);
        }

        merged_url_path
    }

    #[inline]
    fn assemble_query_params<I, K, V>(query_params: I) -> Vec<(String, String)>
        where
            I: IntoIterator,
            K: AsRef<str>,
            V: AsRef<str>,
            <I as IntoIterator>::Item: Borrow<(K, V)>
    {
        let mut query_params_list = Vec::new();

        for pair in query_params.into_iter() {
            let param = String::from(pair.borrow().0.as_ref());
            let value = String::from(pair.borrow().1.as_ref());
            query_params_list.push((param, value));
        }

        query_params_list
    }

    fn parse_host_with_port(host: &str) -> (Option<String>, Option<u16>) {
        if let Ok(url) = Url::parse(&(String::from("http://") + host)) {
            let host = if let Some(val) = url.host_str() {
                let host = String::from(val);
                Some(host)
            } else {
                None
            };
            let port = if let Some(val) = url.port() { Some(val) } else { None };

            (host, port)
        } else {
            (None, None)
        }
    }

    fn get_host_with_port(url: &Url) -> Option<String> {
        if let Some(host) = url.host_str() {
            let mut host = String::from(host);

            if let Some(port) = url.port() {
                host = host + ":" + &port.to_string();
            }

            Some(host)
        } else {
            None
        }
    }

    fn get_url_path_with_query(url: &Url) -> String {
        let mut path = String::from(url.path());

        if let Some(query) = url.query() {
            path = path + "?" + query;
        }

        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestSt {
        pub api_access_secret: String,
        pub sign_date: Option<Date>,
        pub signing_key: Option<[u8; 32]>,
    }

    impl BaseCatenisClient for TestSt {
        fn get_api_access_secret_ref(&self) -> &String {
            &self.api_access_secret
        }

        fn get_sign_date_ref(&self) -> &Option<Date> {
            &self.sign_date
        }

        fn get_sign_date_mut_ref(&mut self) -> &mut Option<Date> {
            &mut self.sign_date
        }

        fn get_signing_key_mut_ref(&mut self) -> &mut Option<[u8; 32]> {
            &mut self.signing_key
        }
    }

    #[test]
    fn it_access_fields() {
        let mut st = TestSt {
            api_access_secret: String::from("ABCDEFG"),
            sign_date: Some(time::date!(2020-12-11)),
            signing_key: Some([1_u8; 32]),
        };

        assert_eq!(*st.get_api_access_secret_ref(), st.api_access_secret);
        assert_eq!(*st.get_sign_date_ref(), st.sign_date);

        // Update sign date
        *st.get_sign_date_mut_ref() = Some(time::date!(2020-12-25));

        assert_eq!(st.sign_date, Some(time::date!(2020-12-25)));

        // Update signing key
        *st.get_signing_key_mut_ref() = Some([2_u8; 32]);

        assert_eq!(st.signing_key, Some([2_u8; 32]));
    }

    #[test]
    fn it_update_none_sign_data() {
        let mut st = TestSt {
            api_access_secret: String::from("ABCDEFG"),
            sign_date: None,
            signing_key: None,
        };

        let now: OffsetDateTime = time::date!(2020-12-11).midnight().assume_utc();

        st.check_update_sign_date_and_key(&now);

        assert_eq!(st.sign_date, Some(now.date()));
        assert!(st.signing_key.is_some(), "field 'signing_key' was not updated as expected");
    }

    #[test]
    fn it_no_need_update_sign_data() {
        let mut st = TestSt {
            api_access_secret: String::from("ABCDEFG"),
            sign_date: Some(time::date!(2020-12-05)),
            signing_key: Some([1_u8; 32]),
        };
        let orig_st = st.clone();

        let now: OffsetDateTime = time::date!(2020-12-11).with_time(time::time!(23:59:55)).assume_utc();

        st.check_update_sign_date_and_key(&now);

        assert_eq!(st.sign_date, orig_st.sign_date);
        assert_eq!(st.signing_key, orig_st.signing_key);
    }

    #[test]
    fn it_indeed_update_sign_data() {
        let mut st = TestSt {
            api_access_secret: String::from("ABCDEFG"),
            sign_date: Some(time::date!(2020-12-04)),
            signing_key: Some([1_u8; 32]),
        };
        let orig_st = st.clone();

        let now: OffsetDateTime = time::date!(2020-12-11).with_time(time::time!(23:59:55)).assume_utc();

        st.check_update_sign_date_and_key(&now);

        assert_ne!(st.sign_date, orig_st.sign_date);
        assert_ne!(st.signing_key, orig_st.signing_key);
    }

    #[test]
    fn it_merge_url_params() {
        let url_path = "dir1/:par1/dir2/:par2";

        let merged_url_path = TestSt::merge_url_params(url_path, &[
            ("par1", "val1"),
            ("par2", "val2"),
        ]);

        assert_eq!(merged_url_path, "dir1/val1/dir2/val2");
    }

    #[test]
    fn it_assemble_query_params() {
        let query_params = TestSt::assemble_query_params(&[
            ("par1", "val1"),
            ("par2", "val2"),
        ]);

        assert_eq!(query_params, vec![(String::from("par1"), String::from("val1")),(String::from("par2"), String::from("val2"))]);
    }

    #[test]
    fn it_parse_invalid_host_with_port() {
        let parsed_host = TestSt::parse_host_with_port("");

        assert_eq!(parsed_host, (None, None));
    }

    #[test]
    fn it_parse_host_with_no_port() {
        let parsed_host = TestSt::parse_host_with_port("localhost");

        assert_eq!(parsed_host, (Some(String::from("localhost")), None));
    }

    #[test]
    fn it_parse_host_with_port() {
        let parsed_host = TestSt::parse_host_with_port("localhost:3000");

        assert_eq!(parsed_host, (Some(String::from("localhost")), Some(3000_u16)));
    }

    #[test]
    fn it_get_no_host_with_port() {
        let url = Url::parse("unix:/run/foo.socket").unwrap();

        let host_with_port = TestSt::get_host_with_port(&url);

        assert_eq!(host_with_port, None);
    }

    #[test]
    fn it_get_host_with_no_port() {
        let url = Url::parse("http://localhost/").unwrap();

        let host_with_port = TestSt::get_host_with_port(&url);

        assert_eq!(host_with_port, Some(String::from("localhost")));
    }

    #[test]
    fn it_get_host_with_port() {
        let url = Url::parse("http://localhost:3000/").unwrap();

        let host_with_port = TestSt::get_host_with_port(&url);

        assert_eq!(host_with_port, Some(String::from("localhost:3000")));
    }

    #[test]
    fn it_get_url_path_with_no_query() {
        let url = Url::parse("http://localhost:3000/dir1/resource1").unwrap();

        let path_with_query = TestSt::get_url_path_with_query(&url);

        assert_eq!(path_with_query, String::from("/dir1/resource1"));
    }

    #[test]
    fn it_get_url_path_with_query() {
        let url = Url::parse("http://localhost:3000/dir1/resource1?parm1=val1&parm2=val2").unwrap();

        let path_with_query = TestSt::get_url_path_with_query(&url);

        assert_eq!(path_with_query, String::from("/dir1/resource1?parm1=val1&parm2=val2"));
    }
}