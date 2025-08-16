use socket_ext::EthernetSocket;

#[tokio::main]
async fn main() {
    let socket = EthernetSocket::new("dummy0").unwrap();
    let mut buf = [0u8; 1500];
    let n = socket.recv_frame(&mut buf).await.unwrap();
    println!("recv: {:?}", n);
}
