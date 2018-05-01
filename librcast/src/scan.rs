use std::str::FromStr;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use dns_parser::Packet;

use device::Device;

static SERVICE_NAME: &str = "_googlecast._tcp.local";
static INTERFACE: &str = "0.0.0.0";
static MULTICAST_ADDR: &str = "224.0.0.251";
static MULTICAST_PORT: u16 = 5353;

type Error = ScanError;

pub fn scan_for(time_millis: u64) -> Vec<Device> {
    // let local_addr = Ipv4Addr::from_str(INTERFACE).unwrap();
    // let multicast_addr = Ipv4Addr::from_str(MULTICAST_ADDR).unwrap();
    // let listener_socket_addr = SocketAddr::new(local_addr.into(), MULTICAST_PORT);
    unimplemented!();
}

/// Figure out if a device is GCast enabled by looking at the DNS response
pub fn is_cast_device(packet: &Packet) -> bool {
    packet
        .answers
        .first()
        .map(|a| a.name.to_string() == SERVICE_NAME)
        .unwrap_or(false)
}

fn get_listener_socket(
    listener_socket_addr: SocketAddr,
    local_addr: &Ipv4Addr,
    multicast_addr: &Ipv4Addr,
) -> Result<UdpSocket, Error> {
    let socket = UdpSocket::bind(&listener_socket_addr)
        .map_err(|_| ScanError::ListnerSocketBind)?;

    socket.join_multicast_v4(multicast_addr, local_addr)
        .map_err(|_| ScanError::CannotJoinMulticast)?;

    Ok(socket)
}

fn get_client_socket(
    client_socket_addr: SocketAddr,
    multicast_socket_addr: SocketAddr,
) -> Result<UdpSocket, Error> {
    let socket =
        UdpSocket::bind(&client_socket_addr)
        .map_err(|_| ScanError::ClientSocketBind)?;

    socket
        .connect(multicast_socket_addr)
        .map_err(|_| ScanError::ClientSocketConnect)?;

    Ok(socket)
}

#[derive(Debug, Fail, PartialEq)]
pub enum ScanError {
    #[fail(display = "Could not bind listener to localhost on port 5353")]
    ListnerSocketBind,
    #[fail(display = "Could not join multicast on listener socket")]
    CannotJoinMulticast,
    #[fail(display = "Could not bind client to localhost on any free port")]
    ClientSocketBind,
    #[fail(display = "Could not connect client to multicast socket address")]
    ClientSocketConnect,
}
