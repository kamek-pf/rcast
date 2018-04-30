use dns_parser::Packet;

static SERVICE_NAME: &str = "_googlecast._tcp.local";

/// Figure out if a device is GCast enabled by looking at the DNS response
pub fn is_cast_device(packet: &Packet) -> bool {
    packet
        .answers
        .first()
        .map(|a| a.name.to_string() == SERVICE_NAME)
        .unwrap_or(false)
}
