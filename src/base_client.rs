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

    fn get_host_with_port(url: &reqwest::Url) -> Option<String> {
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

    fn get_url_path_with_query(url: &reqwest::Url) -> String {
        let mut path = String::from(url.path());

        if let Some(query) = url.query() {
            path = path + "?" + query;
        }

        path
    }
}