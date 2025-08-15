use anyhow::Result;
use bytes::{Buf, BufMut};

use crate::{AppPacket, IpPacket, PluginInfo};

pub enum ProtocolPacket {
    PluginInfo(PluginInfo),
    Config(String),
    SendIP(IpPacket),
    RecvIP(IpPacket),
    SendUDP(AppPacket),
    RecvUDP(AppPacket),
    SendTCP(AppPacket),
    RecvTCP(AppPacket),
}

impl ProtocolPacket {
    pub fn dump(&self, buf: &mut impl BufMut) {
        // Version
        buf.put_u8(1);

        match self {
            ProtocolPacket::PluginInfo(plugin_info) => {
                // OpCode
                buf.put_u8(1);

                // Type
                buf.put_u8(1);
                plugin_info.dump(buf);
            }
            ProtocolPacket::Config(config) => {
                // OpCode
                buf.put_u8(1);

                // Type
                buf.put_u8(2);
                buf.put_slice(config.as_bytes());
            }
            ProtocolPacket::SendIP(ip_packet) => {
                // OpCode
                buf.put_u8(2);

                // Type
                buf.put_u8(1);
                ip_packet.dump(buf);
            }
            ProtocolPacket::RecvIP(ip_packet) => {
                // OpCode
                buf.put_u8(2);

                // Type
                buf.put_u8(2);
                ip_packet.dump(buf);
            }
            ProtocolPacket::SendUDP(app_packet) => {
                // OpCode
                buf.put_u8(2);

                // Type
                buf.put_u8(3);
                app_packet.dump(buf);
            }
            ProtocolPacket::RecvUDP(app_packet) => {
                // OpCode
                buf.put_u8(2);

                // Type
                buf.put_u8(4);
                app_packet.dump(buf);
            }
            ProtocolPacket::SendTCP(app_packet) => {
                // OpCode
                buf.put_u8(2);

                // Type
                buf.put_u8(5);
                app_packet.dump(buf);
            }
            ProtocolPacket::RecvTCP(app_packet) => {
                // OpCode
                buf.put_u8(2);

                // Type
                buf.put_u8(6);
                app_packet.dump(buf);
            }
        }
    }

    pub fn load(buf: &mut impl Buf) -> Result<Self> {
        let version = buf.get_u8();
        if version != 1 {
            anyhow::bail!("invalid protocol version: {}", version);
        }

        let op_code = buf.get_u8();
        let ty = buf.get_u8();

        match (op_code, ty) {
            (1, 1) => Ok(ProtocolPacket::PluginInfo(PluginInfo::load(buf)?)),
            (1, 2) => {
                let length = buf.get_u16();
                let config = buf.copy_to_bytes(length as usize);
                Ok(ProtocolPacket::Config(String::from_utf8(config.to_vec())?))
            }
            (2, 1) => Ok(ProtocolPacket::SendIP(IpPacket::load(buf)?)),
            (2, 2) => Ok(ProtocolPacket::RecvIP(IpPacket::load(buf)?)),
            (2, 3) => Ok(ProtocolPacket::SendUDP(AppPacket::load(buf)?)),
            (2, 4) => Ok(ProtocolPacket::RecvUDP(AppPacket::load(buf)?)),
            (2, 5) => Ok(ProtocolPacket::SendTCP(AppPacket::load(buf)?)),
            (2, 6) => Ok(ProtocolPacket::RecvTCP(AppPacket::load(buf)?)),
            _ => anyhow::bail!("invalid protocol packet type: {}", ty),
        }
    }
}
