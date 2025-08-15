use anyhow::Result;
use bytes::{Buf, BufMut};

pub struct PluginInfo {
    pub name: String,
    pub version: String,
}

impl PluginInfo {
    pub fn dump(&self, buf: &mut impl BufMut) {
        buf.put_u8(self.name.len() as u8);
        buf.put_slice(self.name.as_bytes());
        buf.put_u8(self.version.len() as u8 + 1);
        buf.put_slice(self.version.as_bytes());
        buf.put_u8(0);
    }

    pub fn load(buf: &mut impl Buf) -> Result<Self> {
        let name_len = buf.get_u8();
        let name = buf.copy_to_bytes(name_len as usize);
        let version_len = buf.get_u8();
        let version = buf.copy_to_bytes(version_len as usize);

        Ok(Self {
            name: String::from_utf8(name.to_vec())?,
            version: String::from_utf8(version.to_vec())?,
        })
    }
}
