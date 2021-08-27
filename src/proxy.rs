use basic_cookies::Cookie;
use hyper::header::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use ini::Ini;
use log::{debug, error, info, trace};
use std::collections::HashMap;

use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};
use url::Url;

use crate::{auth, device};

fn debug_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_str = format!("{:?}", req);
    Ok(Response::new(Body::from(body_str)))
}

fn error_request(text: &'static str) -> Result<Response<Body>, Infallible> {
    error!("{:}", text);
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(text))
        .unwrap())
}

async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let host = req.headers().get("host").unwrap().to_str().unwrap();
    let vhost = host.split(".").collect::<Vec<&str>>()[0];
    let mut token = Option::None;

    if req.uri().path().starts_with("/debug") {
        return debug_request(req);
    }

    // token validating
    let abs_url = format!("http://{}{}", host, req.uri().to_string());
    let hash_query: HashMap<_, _> = Url::parse(&abs_url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();

    token = match hash_query.get("token") {
        Some(it) => Some(it.clone()),
        None => match req.headers().get("cookie") {
            Some(it) => {
                let parsed_cookies = Cookie::parse(it.to_str().unwrap()).unwrap();
                let mut t = None;
                for i in &parsed_cookies {
                    if i.get_name() == "token" {
                        t = Some(i.get_value().to_string());
                        break;
                    }
                }
                if t.is_some() {
                    t
                } else {
                    return error_request("token is missing!");
                }
            }
            None => {
                return error_request("token is missing!");
            }
        },
    };

    let sn = vhost.to_uppercase();
    match auth::token_verify(token.clone().unwrap().as_str()) {
        Ok(it) => {
            trace!("{} TOKEN: {:}", sn, it);
        }
        Err(e) => {
            error!("{} URI:{} ERR:{:?}", sn, req.uri(), e);
            return error_request("you are not login.");
        }
    }

    // token is fine, move on
    if req.uri().path() == "/" {
        // -> index page
        match device::get_port(&sn).await {
            Some(it) => {
                let mut map = device::DEVICES.lock().await;
                map.insert(sn, it);
                it
            }
            None => return error_request("invaild port!"),
        };
    }
    let sn = vhost.to_uppercase();
    let map = device::DEVICES.lock().await;
    let remote_port = match map.get(&sn) {
        Some(it) => it,
        None => return error_request("invaild port!"),
    };

    // there it's, we qre going to proxy all request to this url
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("proxy")).unwrap();
    let remote_url = format!("http://{}:{}", sec.get("remote_ip").unwrap(), remote_port);
    info!("{} {} {}", sn, remote_port, req.uri());

    match hyper_reverse_proxy::call(client_ip, remote_url.as_str(), req).await {
        Ok(mut response) => {
            if token.is_some() {
                let cookie = format!("token={}", token.clone().unwrap());
                response
                    .headers_mut()
                    .insert("Set-Cookie", HeaderValue::from_str(&cookie).unwrap());
            };
            Ok(response)
        }
        Err(_error) => {
            error!("{:?}", _error);
            error_request("server error")
        }
    }
}

pub async fn serv() {
    debug!("{}", auth::token_issue());
    let conf = Ini::load_from_file("./conf/config.ini").unwrap();
    let sec = conf.section(Some("proxy")).unwrap();
    let bind_addr = format!(
        "{}:{}",
        sec.get("local_ip").unwrap(),
        sec.get("local_port").unwrap()
    );
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    info!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}
