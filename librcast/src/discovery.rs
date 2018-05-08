use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::str::FromStr;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use dns_parser::{Builder, Packet, QueryClass, QueryType};

use device::Device;

static SERVICE_NAME: &str = "_googlecast._tcp.local";
static INTERFACE: &str = "0.0.0.0";
static MULTICAST_ADDR: &str = "224.0.0.251";
static MULTICAST_PORT: u16 = 5353;

type Error = ScanError;

pub fn scan() -> Result<Vec<Device>, Error> {
    scan_for(Duration::from_millis(350))
}

pub fn scan_for(duration: Duration) -> Result<Vec<Device>, Error> {
    let (listener_socket, client_socket) = get_sockets(duration)?;
    let (sender, receiver) = mpsc::channel();
    let mut devices: Vec<Device> = vec![];

    broadcast(client_socket)?;

    // Listen DNS responses and push valid devices through the channel
    thread::spawn(move || loop {
        let mut buf = [0; 1500];
        let res = listener_socket
            .recv(&mut buf)
            .map_err(|_| ScanError::Timeout)
            .and_then(|_| Packet::parse(&buf).map_err(|_| ScanError::MalformedResponse))
            .and_then(|p| Device::from_dns_packet(&p).map_err(|_| ScanError::NotChromecast))
            .and_then(|d| sender.send(d).map_err(|_| ScanError::Timeout));

        if let Err(ScanError::Timeout) = res {
            break;
        }
    });

    // Read from the channel and build the final vector
    loop {
        match receiver.recv_timeout(duration) {
            Ok(d) => devices.push(d),
            Err(_) => break,
        }
    }

    Ok(devices)
}

/// Figure out if a device is GCast enabled by looking at the DNS response
pub fn is_cast_device(packet: &Packet) -> bool {
    packet
        .answers
        .first()
        .map(|a| a.name.to_string() == SERVICE_NAME)
        .unwrap_or(false)
}

fn get_sockets(timeout: Duration) -> Result<(UdpSocket, UdpSocket), Error> {
    let local_addr = Ipv4Addr::from_str(INTERFACE).unwrap();
    let multicast_addr = Ipv4Addr::from_str(MULTICAST_ADDR).unwrap();

    let listener_socket_addr = SocketAddr::new(local_addr.into(), MULTICAST_PORT);
    let multicast_socket_addr = SocketAddr::new(multicast_addr.into(), MULTICAST_PORT);
    let client_socket_addr = SocketAddr::new(local_addr.into(), 0);

    let listener_socket =
        get_listener_socket(listener_socket_addr, &local_addr, &multicast_addr, timeout)?;
    let client_socket = get_client_socket(client_socket_addr, multicast_socket_addr)?;

    Ok((listener_socket, client_socket))
}

fn get_listener_socket(
    listener_socket_addr: SocketAddr,
    local_addr: &Ipv4Addr,
    multicast_addr: &Ipv4Addr,
    timeout: Duration,
) -> Result<UdpSocket, Error> {
    let socket = UdpSocket::bind(listener_socket_addr).map_err(|_| ScanError::ListnerSocketBind)?;

    socket
        .join_multicast_v4(multicast_addr, local_addr)
        .map_err(|_| ScanError::CannotJoinMulticast)?;

    socket
        .set_read_timeout(Some(timeout))
        .map_err(|_| ScanError::SetTimeout)?;

    Ok(socket)
}

fn get_client_socket(
    client_socket_addr: SocketAddr,
    multicast_socket_addr: SocketAddr,
) -> Result<UdpSocket, Error> {
    let socket = UdpSocket::bind(client_socket_addr).map_err(|_| ScanError::ClientSocketBind)?;

    socket
        .connect(multicast_socket_addr)
        .map_err(|_| ScanError::ClientSocketConnect)?;

    Ok(socket)
}

fn broadcast(client_socket: UdpSocket) -> Result<(), Error> {
    let mut builder = Builder::new_query(0, false);
    builder.add_question(SERVICE_NAME, QueryType::PTR, QueryClass::IN);
    let packet_data = builder.build().unwrap();

    client_socket
        .send(&packet_data)
        .map_err(|_| ScanError::Broadcast)?;

    Ok(())
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
    #[fail(display = "Could not send broadcast DNS query")]
    Broadcast,
    #[fail(display = "Could not set timout on UDP socket")]
    SetTimeout,
    #[fail(display = "UDP socket timeout")]
    Timeout,
    #[fail(display = "Malformed DNS response")]
    MalformedResponse,
    #[fail(display = "The device found doesnt appear to be a Chromecast")]
    NotChromecast,
}

// impl From<DeviceError> for ScanError {
//     fn from(err: DeviceError) -> ScanError {
//         ScanError::NotChromecast
//     }
// }
