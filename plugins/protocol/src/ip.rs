use anyhow::Result;
use bytes::{Buf, BufMut};

use crate::IpEndpoint;

pub struct IpPacket {
    pub endpoint: IpEndpoint,
    pub data: Vec<u8>,
}

impl IpPacket {
    pub fn dump(&self, buf: &mut impl BufMut) {
        self.endpoint.dump(buf);
        buf.put_u16(self.data.len() as u16);
        buf.put_slice(&self.data);
    }

    pub fn load(buf: &mut impl Buf) -> Result<Self> {
        let endpoint = IpEndpoint::load(buf)?;
        let length = buf.get_u16();
        let data = buf.copy_to_bytes(length as usize).to_vec();
        Ok(Self { endpoint, data })
    }
}
