use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use ini::Ini;
use serde_json::json;

use jsonwebtokens as jwt;
use jwt::{Algorithm, AlgorithmID, Verifier};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn token_issue() -> String {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("proxy")).unwrap();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, sec.get("jwt_secret").unwrap()).unwrap();
    let header = json!({ "alg": "HS256" });
    let claims = json!({
        "aud": sec.get("jwt_audience").unwrap(),
        "exp": now() + 3600, // one hour
    });

    jwt::encode(&header, &claims, &alg).unwrap()
}

pub fn token_verify(token: &str) -> bool {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("proxy")).unwrap();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, sec.get("jwt_secret").unwrap()).unwrap();

    let verifier = Verifier::create()
        .audience(sec.get("jwt_audience").unwrap())
        .build()
        .unwrap();

    verifier.verify(token, &alg).is_ok()
}
