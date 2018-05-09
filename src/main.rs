extern crate librcast;

use librcast::discovery;

fn main() {
    let devices = discovery::scan();
    if let Ok(devices) = devices {
        for device in devices {
            println!("{:?}", device);
        }
    }
}
