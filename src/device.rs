use ini::Ini;
use lazy_static::lazy_static;
use log::{debug, info};
use md5;
use postgres::{Client, NoTls};
use std::{collections::HashMap, time::SystemTime};
use tokio::sync::Mutex;

lazy_static! {
    pub static ref DEVICES: Mutex<HashMap<String, u16>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

fn db_connect() -> Client {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("db")).unwrap();
    let conn_str = format!(
        "host={} user={} password={} dbname={}",
        sec.get("host").unwrap(),
        sec.get("user").unwrap(),
        sec.get("password").unwrap(),
        sec.get("dbname").unwrap()
    );
    Client::connect(&conn_str, NoTls).unwrap()
}

pub fn db_init() {
    let mut client = db_connect();
    let mut transaction = client.transaction().unwrap();

    transaction
        .execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id      SERIAL PRIMARY KEY,
                token   VARCHAR(16) NOT NULL,
                sn      VARCHAR(32) NOT NULL,
                created TIMESTAMP NOT NULL default now(),
                UNIQUE  (token, sn)
            )",
            &[],
        )
        .unwrap();

    let token = "tzWidn138x=H";
    let sn = ["TZZA00004", "T00A00001"];
    transaction
        .execute(
            "INSERT INTO sessions (token, sn) VALUES ($1, $2) ON CONFLICT (token, sn) DO NOTHING",
            &[&token, &sn[0]],
        )
        .unwrap();

    transaction
        .execute(
            "INSERT INTO sessions (token, sn) VALUES ($1, $2) ON CONFLICT (token, sn) DO NOTHING",
            &[&token, &sn[1]],
        )
        .unwrap();

    transaction.commit().unwrap();
    client.close().unwrap();
}

pub fn is_login(token: &str) -> bool {
    info!("{}", "token validating...");

    let mut client = db_connect();

    let login = match client.query_one(
        "SELECT id, token FROM sessions WHERE token = $1 ORDER BY created DESC LIMIT 1",
        &[&token],
    ) {
        Ok(row) => {
            let id: i32 = row.get("id");
            debug!("sessions.id = {:}", id);
            true
        }
        Err(_) => false,
    };

    client.close().unwrap();

    login
}

pub async fn get_port(sn: &str) -> Option<u16> {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("gw")).unwrap();
    let api_url = format!(
        "http://{}/api/device/detail",
        sec.get("api_server").unwrap()
    );
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let secret = sec.get("api_secret").unwrap();
    let sign_orig = format!("timestamp={}&secret={}", timestamp, secret);
    let sign = format!("{:x}", md5::compute(sign_orig));
    let params = [("sn", sn)];

    let client = reqwest::Client::new();
    let res = client
        .post(&api_url)
        .header("timestamp", timestamp)
        .header("sign", &sign)
        .form(&params)
        .send()
        .await
        .unwrap();
    // println!("{:}", &sign);
    // println!("{:#?}", res.json::<serde_json::Value>().await.unwrap());
    match res.json::<serde_json::Value>().await {
        Ok(it) => match it["data"]["port"].to_string().parse::<u16>() {
            Ok(port) => match port {
                0 => None,
                _ => Some(port),
            },
            _ => None,
        },
        _ => None,
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
    fn test_login() {
        db_init();
        let token = "tzWidn138x=H";
        assert!(is_login(token));
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
