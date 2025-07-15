use log::{error, info, warn};
use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use crate::connection::ConnectionDetails;
use crate::minecraft::minecraft::Minecraft;
use crate::proxy_config::{Games, MappingError, ServerMap};

pub async fn listener_loop(listener: TcpListener, server_map: ServerMap, game: Games) {
    let map_arc = Arc::new(server_map);

    loop {
        let handle = listener.accept();
        match handle.await {
            Ok((stream, _)) => {
                let map_ref = map_arc.clone();
                let game = game.clone();

                tokio::spawn(async move {
                    let pure_ref = map_ref.as_ref();
                    info!("New connection");

                    match new_connection_handler(stream, pure_ref, game).await {
                        Err(err) => warn!(
                            "Error encountered during connection, dropping connection.\n{:?}\n{:?}",
                            err.kind(),
                            err.to_string()
                        ),
                        _ => {
                            info!("Connection closed");
                        }
                    }
                });
            }
            Err(e) => {
                warn!(
                    "Connection received but failed to establish, ignoring.\n{:?}",
                    e
                );
            }
        }
    }
}

async fn new_connection_handler(
    mut client_stream: TcpStream,
    mapping: &ServerMap,
    game: Games,
) -> io::Result<()> {
    let connection = match game {
        Games::Minecraft => Minecraft::new_connection(&mut client_stream).await?,
    };

    let (target_host, target_port) = match mapping.lookup(&connection.target_address) {
        Ok(result) => Ok(result),
        Err(MappingError::TargetError(e)) => {
            let _ = client_stream.shutdown();
            Err(std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                format! {"TARGET ERROR: {}:{}\n{:?}", connection.target_address, game as usize, e.to_string()},
            ))
        }

        Err(MappingError::GameError(e)) => {
            let _ = client_stream.shutdown();
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format! {"GAME ERROR: {:?}:{:?}\n{:?}", connection.target_address, game as usize, e.to_string()},
            ))
        }
    }?;

    let mut server_stream =
        match TcpStream::connect((target_host.clone(), *target_port as u16)).await {
            Ok(stream) => Ok(stream),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::HostUnreachable,
                format!(
                    "Failed to connect to remote server {:?}:{:?} | {:?}",
                    target_host, target_port, e
                ),
            )),
        }?;

    server_stream.write_all(&connection.first_packet).await?;
    server_stream.flush().await?;

    let (mut server_read, mut server_write) = server_stream.split();
    let (mut client_read, mut client_write) = client_stream.split();

    tokio::select!(
        result = forwarding_loop(&mut client_read, &mut server_write) => {connection_ended(result)},
        result = forwarding_loop(&mut server_read, &mut client_write) => {connection_ended(result)},
    );
    Ok(())
}

async fn forwarding_loop(
    in_stream: &mut tokio::net::tcp::ReadHalf<'_>,
    out_stream: &mut tokio::net::tcp::WriteHalf<'_>,
) -> Result<(), io::Error> {
    loop {
        match in_stream.readable().await {
            Ok(_) => {
                if let Err(e) = forward_one_packet(in_stream, out_stream).await {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        return Err(e);
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
    }
}

async fn forward_one_packet(
    in_stream: &mut tokio::net::tcp::ReadHalf<'_>,
    out_stream: &mut tokio::net::tcp::WriteHalf<'_>,
) -> Result<(), io::Error> {
    let mut buff = [0_u8; 1500];
    match in_stream.try_read(&mut buff[..]) {
        Ok(0) => Err(io::Error::new(
            io::ErrorKind::ConnectionReset,
            "Connection closed by remote",
        )),
        Ok(bytes_read) => {
            out_stream.writable().await?;
            out_stream.try_write(&buff[..bytes_read])?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn connection_ended(result: Result<(), io::Error>) {
    match result {
        Ok(_) => {
            info!("Connection ended without error");
        }
        Err(e) => {
            warn!("Connection ended with error.\n{:?}", e);
        }
    }
}
