use crate::connection::{ConnectionDetails, NewConnection};
use tokio::io::{AsyncReadExt, Interest};
use tokio::net::TcpStream;

enum McLoaders {
    Unknown,
    Vanilla,
    Forge,
}

pub enum Minecraft {}

impl ConnectionDetails for Minecraft {
    async fn new_connection(stream: &mut TcpStream) -> Result<NewConnection, std::io::Error> {
        Self::determine_target_host(stream).await
    }
}

impl Minecraft {
    async fn determine_target_host(
        stream: &mut TcpStream,
    ) -> Result<NewConnection, std::io::Error> {
        let loader: McLoaders;

        let mut packet: [u8; 1024] = [0; 1024];

        loop {
            let ready = stream.ready(Interest::READABLE).await?;
            if ready.is_readable() {
                if stream.read(&mut packet).await? > 0 {
                    loader = Self::determine_loader(&packet);
                    break;
                }
            }
        }

        let address_segment = match loader {
            McLoaders::Unknown => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Could not identify the Minecraft loader type, is the client Minecraft?",
            )),
            McLoaders::Vanilla => {
                let address_segment_len = packet[4] as usize;
                Ok(packet[5..address_segment_len + 5].to_vec())
            }
            McLoaders::Forge => {
                let address_segment_len = packet[4] as usize;
                Ok(packet[5..address_segment_len].to_vec())
            }
        }?;

        let target_host = match String::from_utf8(address_segment) {
            Ok(result) => result,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    "Address does not exist, is the client Minecraft?",
                ));
            }
        };

        Ok(NewConnection {
            target_address: target_host,
            first_packet: packet.to_vec(),
        })
    }

    fn determine_loader(packet: &[u8]) -> McLoaders {
        // Ensure packet has data
        if packet.len() < 1 {
            return McLoaders::Unknown;
        }

        let address_segment_len = packet[4] as usize;

        // If Forge; 0x70 77 76 == FML
        // Forge includes an indicator at the end of the address segment of the packet
        // Always delmited by 0x00
        if packet[address_segment_len..address_segment_len + 3] == [0x70, 0x77, 0x76] {
            return McLoaders::Forge;
        }

        // As of right now Fabric is no different from vanilla on the handshake packet

        McLoaders::Vanilla
    }
}
