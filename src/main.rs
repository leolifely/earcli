use bluer::rfcomm::{Stream, SocketAddr};
use tokio::io::AsyncWriteExt;



#[tokio::main]
async fn main() {
    let address: bluer::Address = "2C:BE:EB:D7:71:56".parse().unwrap();
    let target = SocketAddr::new(address, 15);
    let data = hex::decode("5560010ff00300ed010100cd47").unwrap();

    let session = bluer::Session::new().await.unwrap();

    let adapter = session.default_adapter().await.unwrap();
    adapter.set_powered(true).await.unwrap();

    let mut socket = Stream::connect(target).await.unwrap();

    socket.write_all(&data).await.unwrap();
}