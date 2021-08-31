use hyper::{Body, Method, Request};
use ini::Ini;
use lazy_static::lazy_static;
use log::debug;
use md5;
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::util;

lazy_static! {
    pub static ref DEVICES: Mutex<HashMap<String, u16>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

pub async fn get_port(sn: &str) -> Option<u16> {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("gw")).unwrap();
    let api_url = format!(
        "http://{}/api/device/detail",
        sec.get("api_server").unwrap()
    );
    let timestamp = util::now();
    let secret = sec.get("api_secret").unwrap();
    let sign_orig = format!("timestamp={}&secret={}", timestamp, secret);
    let sign = format!("{:x}", md5::compute(sign_orig));

    let req = Request::builder()
        .method(Method::POST)
        .uri(&api_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("accept", "*/*")
        .header("timestamp", timestamp)
        .header("sign", &sign)
        .body(Body::from(format!("sn={}", sn)))
        .unwrap();
    let client = hyper::Client::new();

    // POST it...
    match client.request(req).await {
        Ok(resp) => {
            let body_vec = hyper::body::to_bytes(resp.into_body())
                .await
                .unwrap()
                .to_vec();
            let body = String::from_utf8_lossy(&body_vec);
            debug!("API: {}", body);
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(v) => match v["data"]["port"].to_string().parse::<u16>() {
                    Ok(port) => match port {
                        0 => None,
                        _ => Some(port),
                    },
                    _ => None,
                },
                _ => None,
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_get_port() {
        assert_eq!(aw!(get_port("T00A00001")), Option::Some(38001));
        assert_eq!(aw!(get_port("TZZA00004")), Option::Some(38003));
    }

    #[test]
    fn test_get_port_invaild() {
        assert_eq!(aw!(get_port("T00B0001")), Option::None);
    }
}
