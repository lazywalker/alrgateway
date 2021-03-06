mod auth;
mod device;
mod proxy;
mod util;

#[tokio::main]
async fn main() {
    log4rs::init_file("./conf/log.yaml", Default::default()).unwrap();
    
    proxy::serv().await;
}
