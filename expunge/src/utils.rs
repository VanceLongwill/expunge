use std::net::IpAddr;

/// Removes the last IP octet that can be used to identify an individual vs a location
///
/// Example:
///
/// 123.89.46.72 -> 123.89.46.0
///
pub fn mask_last_octet(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(ip) => {
            let mut octets = ip.octets();
            octets[3] = 0;
            IpAddr::from(octets)
        }
        IpAddr::V6(ip) => {
            let mut octets = ip.octets();
            octets[15] = 0;
            IpAddr::from(octets)
        }
    }
}
