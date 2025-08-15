use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::Result;
use bytes::{Buf, BufMut};

pub enum IpEndpoint {
    IpAddr(IpAddr),
    Domain(String),
}

impl IpEndpoint {
    pub fn dump(&self, buf: &mut impl BufMut) {
        match self {
            IpEndpoint::IpAddr(IpAddr::V4(addr)) => {
                buf.put_u8(0);
                buf.put_slice(&addr.octets());
            }
            IpEndpoint::Domain(domain) => {
                buf.put_u8(3);
                buf.put_slice(domain.as_bytes());
            }
            IpEndpoint::IpAddr(IpAddr::V6(addr)) => {
                buf.put_u8(4);
                buf.put_slice(&addr.octets());
            }
        }
    }

    pub fn load(buf: &mut impl Buf) -> Result<Self> {
        let ty = buf.get_u8();
        match ty {
            0 => {
                let mut ip = [0u8; 4];
                buf.copy_to_slice(&mut ip);
                Ok(IpEndpoint::IpAddr(IpAddr::V4(Ipv4Addr::from(ip))))
            }
            1 => {
                let domain_len = buf.get_u8();
                let domain = buf.copy_to_bytes(domain_len as usize);
                Ok(IpEndpoint::Domain(String::from_utf8(domain.to_vec())?))
            }
            2 => {
                let mut ip = [0u8; 16];
                buf.copy_to_slice(&mut ip);
                Ok(IpEndpoint::IpAddr(IpAddr::V6(Ipv6Addr::from(ip))))
            }
            _ => anyhow::bail!("invalid ip endpoint type: {}", ty),
        }
    }
}
