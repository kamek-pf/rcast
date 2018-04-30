use std::str::FromStr;
use std::net::Ipv4Addr;

use uuid::Uuid;
use dns_parser::{Packet, RRData};

use discover::is_cast_device;

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
    pub fn from_dns(packet: &Packet) -> Result<Self, Error> {
        if is_cast_device(packet) {
            let dev = Device {
                uuid: Self::get_uuid(packet)?,
                name: Self::get_name(packet)?,
                model: Self::get_model(packet)?,
                ip: Self::get_ip(packet)?,
                port: Self::get_port(packet)?,
            };

            Ok(dev)
        } else {
            Err(Error::InvalidServiceName)
        }
    }

    fn get_uuid(packet: &Packet) -> Result<Uuid, Error> {
        packet
            .additional
            .iter()
            .filter_map(|a| match a.data {
                RRData::A(_) => Some(a),
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
            .ok_or(Error::MissingTxtRecord)
            .and_then(|info| Self::get_name_from_txt(&info))
    }

    fn get_model(packet: &Packet) -> Result<String, Error> {
        packet
            .additional
            .iter()
            .filter_map(|a| match a.data {
                RRData::TXT(ref info) => Some(info),
                _ => None,
            })
            .next()
            .ok_or(Error::MissingTxtRecord)
            .and_then(|info| Self::get_model_from_txt(&info))
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

    fn get_name_from_txt(txt_record: &str) -> Result<String, Error> {
        let parsed = txt_record
            .split("fn=")
            .skip(1)
            .next()
            .ok_or(Error::MalformedTxtRecord)?
            .split("ca=")
            .next()
            .unwrap();

        Ok(parsed.to_owned())
    }

    fn get_model_from_txt(txt_record: &str) -> Result<String, Error> {
        let parsed = txt_record
            .split("md=")
            .skip(1)
            .next()
            .ok_or(Error::MalformedTxtRecord)?
            .split("ic=")
            .next()
            .unwrap();

        Ok(parsed.to_owned())
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    #[fail(display = "The device doesn't appear to be Google Cast enabled")]
    InvalidServiceName,
    #[fail(display = "Uuid found in PTR record is invalid")]
    InvalidUuid,
    #[fail(display = "Could not find device ip address in DNS response")]
    MissingIpAddress,
    #[fail(display = "Could not find device port in DNS response")]
    MissingPort,
    #[fail(display = "Missing TXT record from DNS response, cannot get device information")]
    MissingTxtRecord,
    #[fail(display = "Malformed TXT record, cannot get device information")]
    MalformedTxtRecord,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn txt_parser() {
        let txt_ok_1 = "id=bc7866b8d9b0a99263a2020cd11355f8cd=488042CE34BB67A545D1F0349D27A42Drm=475BF428D3B7BC4Bve=05md=Chromecast Ultraic=/setup/icon.pngfn=Salonca=4101st=0bs=FA8FCA73F8F8nf=1rs=";
        let txt_ok_2 = "id=ca2e93348ea78c867c5ee00e3e3b588dcd=F1D6D14A72EB22FC70176DB9D345FB2Erm=ve=05md=Google Home Miniic=/setup/icon.pngfn=Living Room speakerca=2052st=0bs=FA8FCA7B5D0Cnf=1rs=";

        assert_eq!(
            "Chromecast Ultra",
            Device::get_model_from_txt(txt_ok_1).unwrap()
        );
        assert_eq!(
            "Google Home Mini",
            Device::get_model_from_txt(txt_ok_2).unwrap()
        );

        assert_eq!("Salon", Device::get_name_from_txt(txt_ok_1).unwrap());
        assert_eq!(
            "Living Room speaker",
            Device::get_name_from_txt(txt_ok_2).unwrap()
        );
    }
}
