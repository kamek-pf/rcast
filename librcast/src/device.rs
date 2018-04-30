use std::str::FromStr;
use std::net::Ipv4Addr;

use uuid::Uuid;
use dns_parser::{Packet, RRData};

#[derive(Debug)]
pub struct Device {
    pub uuid: Uuid,
    pub name: String,
    pub model: String,
    pub ip: Ipv4Addr,
    pub port: u16,
}

// Check answers => a.name.to_string() == SERVICE_NAME, then
// additional.data where A(ip) => ip, name.to_string() => uuid
// additional.data SRV(record) => port
// additional.data TXT(desc) => model / name
// "id=bc7866b8d9b0a99263a2020cd11355f8cd=488042CE34BB67A545D1F0349D27A42Drm=475BF428D3B7BC4Bve=05md=Chromecast Ultraic=/setup/icon.pngfn=Salonca=4101st=0bs=FA8FCA73F8F8nf=1rs=""
// "id=ca2e93348ea78c867c5ee00e3e3b588dcd=F1D6D14A72EB22FC70176DB9D345FB2Erm=ve=05md=Google Home Miniic=/setup/icon.pngfn=Living Room speakerca=2052st=0bs=FA8FCA7B5D0Cnf=1rs="

impl Device {
    pub fn from_dns(packet: &Packet) -> i32 {
        unimplemented!()
    }

    fn get_uuid(packet: &Packet) -> Result<Uuid, Error> {
        packet
            .additional
            .iter()
            .filter_map(|a| match a.data {
                RRData::SRV { .. } => Some(a),
                _ => None,
            })
            .next()
            .ok_or(Error::MissingPort)
            .map(|a| a.name.to_string().split('.').next().unwrap().to_owned())
            .and_then(|str_uuid| Uuid::from_str(&str_uuid).map_err(|_| Error::InvalidUuid))
    }

    fn get_name(packet: &Packet) -> Result<String, Error> {
        packet
            .additional
            .iter()
            .filter_map(|a| match a.data {
                RRData::TXT(ref info) => Some(info),
                _ => None,
            })
            .next()
            .ok_or(Error::MissingIpAddress)
            .map(|info| info.split('.').next().unwrap().to_owned())
    }

    fn get_ip(packet: &Packet) -> Result<Ipv4Addr, Error> {
        packet
            .additional
            .iter()
            .filter_map(|a| match a.data {
                RRData::A(ip) => Some(ip),
                _ => None,
            })
            .next()
            .ok_or(Error::MissingIpAddress)
    }

    fn get_port(packet: &Packet) -> Result<u16, Error> {
        packet
            .additional
            .iter()
            .filter_map(|a| match a.data {
                RRData::SRV { port, .. } => Some(port),
                _ => None,
            })
            .next()
            .ok_or(Error::MissingPort)
    }

    fn parse_device_name(device_name: &str) -> Result<(String, Uuid), Error> {
        let mut split: Vec<&str> = device_name
            .split('.')
            .take(1)
            .next()
            .unwrap() // can't fail, we always get something back
            .split('-')
            .collect();

        if split.len() < 2 {
            return Err(Error::DeviceNameFormat(device_name.to_owned()));
        }

        let raw_uuid = split.pop().unwrap(); // same here
        let uuid = Uuid::from_str(raw_uuid).map_err(|_| Error::InvalidUuid)?;
        let model = &split.join(" ");

        Ok((model.to_owned(), uuid))
    }
}

#[derive(Debug, Fail, PartialEq)]
enum Error {
    #[fail(display = "Device name doesn't match the expected format")]
    DeviceNameFormat(String),
    #[fail(display = "Uuid found in PTR record is invalid")]
    InvalidUuid,
    #[fail(display = "Could not find device ip address in DNS response")]
    MissingIpAddress,
    #[fail(display = "Could not find device port in DNS response")]
    MissingPort,
    #[fail(display = "Missing TXT record from DNS response, cannot get device name")]
    MissingTxtRecord,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_ptr() {
        let uuid1 = Uuid::parse_str("bc7866b8d9b0a99263a2020cd11355f8").unwrap();
        let uuid2 = Uuid::parse_str("ca2e93348ea78c867c5ee00e3e3b588d").unwrap();

        let ptr_ok_1 = "Chromecast-Ultra-bc7866b8d9b0a99263a2020cd11355f8._googlecast._tcp.local";
        let ptr_ok_2 = "Google-Home-Mini-ca2e93348ea78c867c5ee00e3e3b588d._googlecast._tcp.local";
        let ptr_ok_3 = "GoogleCastThingy-ca2e93348ea78c867c5ee00e3e3b588d._googlecast._tcp.local";
        let ptr_ok_4 =
            "Random-Fake-Chromecast-Device-ca2e93348ea78c867c5ee00e3e3b588d._googlecast._tcp.local";
        let ptr_ok_5 = "Missing-Mdns-Addr-Device-ca2e93348ea78c867c5ee00e3e3b588d._googlecast";

        assert_eq!(
            ("Chromecast Ultra".to_owned(), uuid1),
            Device::parse_device_name(ptr_ok_1).unwrap()
        );
        assert_eq!(
            ("Google Home Mini".to_owned(), uuid2),
            Device::parse_device_name(ptr_ok_2).unwrap()
        );
        assert_eq!(
            ("GoogleCastThingy".to_owned(), uuid2),
            Device::parse_device_name(ptr_ok_3).unwrap()
        );
        assert_eq!(
            ("Random Fake Chromecast Device".to_owned(), uuid2),
            Device::parse_device_name(ptr_ok_4).unwrap()
        );
        assert_eq!(
            ("Missing Mdns Addr Device".to_owned(), uuid2),
            Device::parse_device_name(ptr_ok_5).unwrap()
        );

        let ptr_ko_1 =
            "Chromecast-Weird-Exa-bc7866b8d9b0a99263a2020cd1135iii._googlecast._tcp.local";
        let ptr_ko_2 = "._googlecast._tcp.local";
        let ptr_ko_3 = "lol wtf";
        let ptr_ko_4 = "";

        assert_eq!(Err(Error::InvalidUuid), Device::parse_device_name(ptr_ko_1));
        assert_eq!(
            Err(Error::DeviceNameFormat(ptr_ko_2.to_owned())),
            Device::parse_device_name(ptr_ko_2)
        );
        assert_eq!(
            Err(Error::DeviceNameFormat(ptr_ko_3.to_owned())),
            Device::parse_device_name(ptr_ko_3)
        );
        assert_eq!(
            Err(Error::DeviceNameFormat(ptr_ko_4.to_owned())),
            Device::parse_device_name(ptr_ko_4)
        );
    }
}
