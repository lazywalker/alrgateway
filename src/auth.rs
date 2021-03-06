use ini::Ini;
use serde_json::{json, Value};

use jsonwebtokens as jwt;
use jwt::{error::Error, Algorithm, AlgorithmID, Verifier};

use crate::util;

pub fn token_issue() -> String {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("proxy")).unwrap();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, sec.get("jwt_secret").unwrap()).unwrap();
    let header = json!({ "alg": "HS256" });
    let claims = json!({
        "aud": sec.get("jwt_audience").unwrap(),
        "exp": util::now() + 3600, // one hour
    });

    jwt::encode(&header, &claims, &alg).unwrap()
}

pub fn token_verify(token: &str) -> Result<Value, Error> {
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("proxy")).unwrap();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, sec.get("jwt_secret").unwrap()).unwrap();

    let verifier = Verifier::create()
        .audience(sec.get("jwt_audience").unwrap())
        .build()?;

    verifier.verify(token, &alg)
}
