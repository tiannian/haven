use socket_ext::IpSocket;

#[tokio::main]
async fn main() {
    let socket = IpSocket::new_v4("dummy0").unwrap();
    let mut buf = [0u8; 1500];
    let n = socket.recv_packet(&mut buf).await.unwrap();
    println!("recv: {:?}", n);
}
