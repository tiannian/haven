use std::{
    ffi::{CString, c_int},
    io,
    mem::zeroed,
    os::{fd::RawFd, raw::c_void},
};

use libc::{
    AF_INET, AF_INET6, IP_HDRINCL, IPPROTO_RAW, IPV6_HDRINCL, SO_BINDTODEVICE, SOCK_RAW, SOL_IP,
    SOL_IPV6, SOL_SOCKET, bind, recv, setsockopt, sockaddr, sockaddr_in, sockaddr_in6, socket,
    socklen_t,
};
use tokio::io::unix::AsyncFd;

pub struct IpSocket {
    fd: AsyncFd<RawFd>,
}

impl IpSocket {
    pub fn _new(
        ifname: &str,
        v: c_int,
        addr: *const sockaddr,
        level: c_int,
        hdrincl: c_int,
    ) -> io::Result<Self> {
        let fd = unsafe { socket(v, SOCK_RAW, IPPROTO_RAW) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let ifname_c = CString::new(ifname).unwrap();
        let rc = unsafe {
            setsockopt(
                fd,
                SOL_SOCKET,
                SO_BINDTODEVICE,
                ifname_c.as_ptr() as *const c_void,
                (ifname.len() + 1) as socklen_t,
            )
        };

        if rc != 0 {
            let e = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(io::Error::new(
                e.kind(),
                format!("SO_BINDTODEVICE({ifname}) failed: {e}"),
            ));
        }

        let rc = unsafe {
            setsockopt(
                fd,
                level,
                hdrincl,
                &1 as *const _ as *const c_void,
                size_of::<c_int>() as socklen_t,
            )
        };

        if rc != 0 {
            unsafe { libc::close(fd) };
            return Err(io::Error::last_os_error());
        }

        let rc = unsafe { bind(fd, addr, size_of::<sockaddr_in>() as u32) };
        if rc < 0 {
            let e = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(e);
        }

        let afd = AsyncFd::new(fd)?;
        Ok(Self { fd: afd })
    }

    pub fn new_v4(ifname: &str) -> io::Result<Self> {
        let mut addr: sockaddr_in = unsafe { zeroed() };
        addr.sin_family = AF_INET as u16;
        addr.sin_addr = libc::in_addr { s_addr: 0 };

        Self::_new(
            ifname,
            AF_INET,
            &addr as *const sockaddr_in as *const sockaddr,
            SOL_IP,
            IP_HDRINCL,
        )
    }

    pub fn new_v6(ifname: &str) -> io::Result<Self> {
        let mut addr: sockaddr_in6 = unsafe { zeroed() };
        addr.sin6_family = AF_INET6 as u16;
        addr.sin6_addr = libc::in6_addr { s6_addr: [0u8; 16] };

        Self::_new(
            ifname,
            AF_INET6,
            &addr as *const sockaddr_in6 as *const sockaddr,
            SOL_IPV6,
            IPV6_HDRINCL,
        )
    }

    pub async fn recv_packet(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            println!("asdsadasda");
            let mut guard = self.fd.readable().await?;
            let n = unsafe {
                let ss = *guard.get_inner();

                recv(ss, buf.as_mut_ptr() as *mut c_void, buf.len(), 0)
            };
            if n >= 0 {
                return Ok(n as usize);
            }
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
