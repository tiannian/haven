use std::{
    ffi::{CString, c_void},
    io,
    mem::zeroed,
    os::fd::RawFd,
};

use libc::{
    AF_PACKET, ETH_P_ALL, ETH_P_IP, ETH_P_IPV6, SOCK_RAW, bind, c_int, close, htons,
    if_nametoindex, recv, send, sockaddr, sockaddr_ll, socket, socklen_t,
};
use tokio::io::unix::AsyncFd;

pub struct EthernetSocket {
    fd: AsyncFd<RawFd>,
}

impl EthernetSocket {
    fn _new(ifname: &str, proto: u16) -> io::Result<Self> {
        let fd = unsafe { socket(AF_PACKET, SOCK_RAW, htons(proto) as c_int) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let c_if = CString::new(ifname).unwrap();
        let ifindex = unsafe { if_nametoindex(c_if.as_ptr()) } as i32;
        if ifindex == 0 {
            unsafe {
                close(fd);
            }

            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("interface '{}' not found", ifname),
            ));
        }

        let mut sll: sockaddr_ll = unsafe { zeroed() };
        sll.sll_family = AF_PACKET as u16;
        sll.sll_ifindex = ifindex;
        sll.sll_protocol = htons(proto);

        let rc = unsafe {
            bind(
                fd,
                &sll as *const sockaddr_ll as *const sockaddr,
                size_of::<sockaddr_ll>() as socklen_t,
            )
        };
        if rc < 0 {
            unsafe { close(fd) };
            return Err(io::Error::last_os_error());
        }

        let afd = AsyncFd::new(fd)?;

        Ok(EthernetSocket { fd: afd })
    }

    pub fn new(ifname: &str) -> io::Result<Self> {
        let proto = ETH_P_ALL as u16;

        Self::_new(ifname, proto)
    }

    pub fn new_ipv4(ifname: &str) -> io::Result<Self> {
        let proto = ETH_P_IP as u16;

        Self::_new(ifname, proto)
    }

    pub fn new_ipv6(ifname: &str) -> io::Result<Self> {
        let proto = ETH_P_IPV6 as u16;

        Self::_new(ifname, proto)
    }

    pub async fn recv_frame(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            println!("asdsadasda");

            let mut guard = self.fd.readable().await?;

            let ss = *guard.get_inner();

            match unsafe { recv(ss, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) } {
                n if n >= 0 => return Ok(n as usize),
                _ => {
                    let err = io::Error::last_os_error();
                    if err.kind() == io::ErrorKind::WouldBlock {
                        guard.clear_ready();
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }

    pub async fn send_frame(&self, frame: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.fd.writable().await?;
            match unsafe {
                let ss = *guard.get_inner();

                send(ss, frame.as_ptr() as *const c_void, frame.len(), 0)
            } {
                n if n >= 0 => return Ok(n as usize),
                _ => {
                    let err = io::Error::last_os_error();
                    if err.kind() == io::ErrorKind::WouldBlock {
                        guard.clear_ready();
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}
