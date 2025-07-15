use tokio::net::TcpStream;

pub trait ConnectionDetails {
    async fn new_connection(stream: &mut TcpStream) -> Result<NewConnection, std::io::Error>;
}

pub struct NewConnection {
    pub target_address: String,
    pub first_packet: Vec<u8>,
}
