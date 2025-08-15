use anyhow::Result;
use bytes::{Buf, BufMut};

pub enum IpEndPoint {
    Ipv4([u8; 4]),
    Domain(String),
    Ipv6([u8; 16]),
}

impl IpEndPoint {
    pub fn dump(&self, buf: &mut impl BufMut) {
        match self {
            IpEndPoint::Ipv4(ip) => {
                buf.put_u8(0);
                buf.put_slice(ip);
            }
            IpEndPoint::Domain(domain) => {
                buf.put_u8(3);
                buf.put_slice(domain.as_bytes());
            }
            IpEndPoint::Ipv6(ip) => {
                buf.put_u8(4);
                buf.put_slice(ip);
            }
        }
    }

    pub fn load(buf: &mut impl Buf) -> Result<Self> {
        let ty = buf.get_u8();
        match ty {
            0 => {
                let mut ip = [0u8; 4];
                buf.copy_to_slice(&mut ip);
                Ok(IpEndPoint::Ipv4(ip))
            }
            1 => {
                let domain_len = buf.get_u8();
                let domain = buf.copy_to_bytes(domain_len as usize);
                Ok(IpEndPoint::Domain(String::from_utf8(domain.to_vec())?))
            }
            2 => {
                let mut ip = [0u8; 16];
                buf.copy_to_slice(&mut ip);
                Ok(IpEndPoint::Ipv6(ip))
            }
            _ => anyhow::bail!("invalid ip endpoint type: {}", ty),
        }
    }
}
