extern crate dns_parser as dns;
extern crate librcast;

// use std::iter::Iterator;
use std::str::FromStr;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use librcast::device::Device;

// use std::thread;
// use std::net;

static SERVICE_NAME: &str = "_googlecast._tcp.local";

static INTERFACE: &str = "0.0.0.0";
static MULTICAST_ADDR: &str = "224.0.0.251";
static MULTICAST_PORT: u16 = 5353;

fn main() {
    let local_addr = Ipv4Addr::from_str(INTERFACE).unwrap();
    let multicast_addr = Ipv4Addr::from_str(MULTICAST_ADDR).unwrap();

    let listener_socket_addr = SocketAddr::new(local_addr.into(), MULTICAST_PORT);
    let target_socket_addr = SocketAddr::new(multicast_addr.into(), MULTICAST_PORT);
    let loopback_socket_addr = SocketAddr::new(local_addr.into(), 0);

    let listener_socket =
        UdpSocket::bind(&listener_socket_addr).expect("server couldn't bind to address");
    listener_socket
        .join_multicast_v4(&multicast_addr, &local_addr)
        .unwrap();

    let client_socket =
        UdpSocket::bind(&loopback_socket_addr).expect("client couldn't bind to address");
    client_socket
        .connect(&target_socket_addr)
        .expect("connect function failed");

    let mut builder = dns::Builder::new_query(0, false);
    builder.add_question(SERVICE_NAME, dns::QueryType::PTR, dns::QueryClass::IN);
    let packet_data = builder.build().unwrap();

    client_socket
        .send(&packet_data)
        .expect("couldn't send message");

    // let ten_millis = time::Duration::from_millis(1000);
    // thread::sleep(ten_millis);

    let mut devices: Vec<Device> = vec![];
    for i in 0..10 {
        let mut buf = [0; 400];
        match listener_socket.recv(&mut buf) {
            Ok(_received) => {
                let packet = dns::Packet::parse(&buf).unwrap();
                if let Ok(d) = Device::from_dns_packet(&packet) {
                    devices.push(d);
                }

                if i > 1 {
                    break;
                }
            }
            Err(e) => println!("recv function failed: {:?}", e),
        };
    }

    println!("Packets : {:?}", devices);
}
