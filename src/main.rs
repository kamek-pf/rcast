extern crate librcast;

use librcast::discovery;

fn main() {
    let devices = discovery::scan();
    println!("{:?}", devices);
}
