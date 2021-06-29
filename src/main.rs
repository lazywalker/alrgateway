mod device;
mod proxy;

#[tokio::main]
async fn main() {
    log4rs::init_file("./conf/log.yaml", Default::default()).unwrap();

    device::db_init();
    proxy::serv().await;
}
