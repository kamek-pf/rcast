use std::str::FromStr;

use uuid::Uuid;

#[derive(Debug)]
pub struct Device {
    pub model: String,
    pub uuid: Uuid,
}

impl Device {
    pub fn new(ip: &str, name: &str, ptr_record: &str) -> Self {
        unimplemented!();
    }

    fn parse_ptr_record(ptr_record: &str) -> Result<(String, Uuid), Error> {
        let mut split: Vec<&str> = ptr_record
            .split('.')
            .take(1)
            .next()
            .unwrap() // can't fail, we always get something back
            .split('-')
            .collect();

        if split.len() < 2 {
            return Err(Error::PtrRecordFormat(ptr_record.to_owned()));
        }

        let raw_uuid = split.pop().unwrap(); // same here
        let uuid = Uuid::from_str(raw_uuid).map_err(|_| Error::InvalidUuid)?;
        let model = &split.join(" ");

        Ok((model.to_owned(), uuid))
    }
}

#[derive(Debug, Fail, PartialEq)]
enum Error {
    #[fail(display = "PTR record doesn't match the expected format")]
    PtrRecordFormat(String),
    #[fail(display = "Uuid found in PTR record is invalid")]
    InvalidUuid,
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
        let ptr_ok_4 = "Random-Fake-Chromecast-Device-ca2e93348ea78c867c5ee00e3e3b588d._googlecast._tcp.local";
        let ptr_ok_5 = "Missing-Mdns-Addr-Device-ca2e93348ea78c867c5ee00e3e3b588d._googlecast";

        assert_eq!(("Chromecast Ultra".to_owned(), uuid1), Device::parse_ptr_record(ptr_ok_1).unwrap());
        assert_eq!(("Google Home Mini".to_owned(), uuid2), Device::parse_ptr_record(ptr_ok_2).unwrap());
        assert_eq!(("GoogleCastThingy".to_owned(), uuid2), Device::parse_ptr_record(ptr_ok_3).unwrap());
        assert_eq!(("Random Fake Chromecast Device".to_owned(), uuid2), Device::parse_ptr_record(ptr_ok_4).unwrap());
        assert_eq!(("Missing Mdns Addr Device".to_owned(), uuid2), Device::parse_ptr_record(ptr_ok_5).unwrap());

        let ptr_ko_1 = "Chromecast-Weird-Exa-bc7866b8d9b0a99263a2020cd1135iii._googlecast._tcp.local";
        let ptr_ko_2 = "._googlecast._tcp.local";
        let ptr_ko_3 = "lol wtf";
        let ptr_ko_4 = "";

        assert_eq!(Err(Error::InvalidUuid), Device::parse_ptr_record(ptr_ko_1));
        assert_eq!(Err(Error::PtrRecordFormat(ptr_ko_2.to_owned())), Device::parse_ptr_record(ptr_ko_2));
        assert_eq!(Err(Error::PtrRecordFormat(ptr_ko_3.to_owned())), Device::parse_ptr_record(ptr_ko_3));
        assert_eq!(Err(Error::PtrRecordFormat(ptr_ko_4.to_owned())), Device::parse_ptr_record(ptr_ko_4));
    }
}
