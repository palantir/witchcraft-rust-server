// Copyright 2021 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

pub fn get_ip() -> Option<IpAddr> {
    let ifaddrs = IfAddrs::get().ok()?;

    let mut fallback = None;
    for ifaddr in &ifaddrs {
        let addr = match ifaddr.addr() {
            Some(addr) if !addr.is_loopback() => addr,
            _ => continue,
        };

        let is_site_local = match addr {
            IpAddr::V4(addr) => addr.is_private(),
            // https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html#method.is_unicast_site_local
            IpAddr::V6(addr) => (addr.segments()[0] & 0xffc0) == 0xfec0,
        };

        if is_site_local {
            return Some(addr);
        }

        if fallback.is_none() {
            fallback = Some(addr);
        }
    }

    fallback
}

foreign_type! {
    unsafe type IfAddrs {
        type CType = libc::ifaddrs;
        fn drop = libc::freeifaddrs;
    }
}

impl IfAddrs {
    fn get() -> io::Result<IfAddrs> {
        unsafe {
            let mut ifaddrs = ptr::null_mut();
            let r = libc::getifaddrs(&mut ifaddrs);
            if r == 0 {
                Ok(IfAddrs::from_ptr(ifaddrs))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl IfAddrsRef {
    fn next(&self) -> Option<&IfAddrsRef> {
        unsafe {
            let next = (*self.as_ptr()).ifa_next;
            if next.is_null() {
                None
            } else {
                Some(IfAddrsRef::from_ptr(next))
            }
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    fn addr(&self) -> Option<IpAddr> {
        unsafe {
            let addr = (*self.as_ptr()).ifa_addr;
            if addr.is_null() {
                return None;
            }

            match libc::c_int::from((*addr).sa_family) {
                libc::AF_INET => {
                    let addr = addr as *mut libc::sockaddr_in;
                    let addr = Ipv4Addr::from(u32::from_be((*addr).sin_addr.s_addr));
                    Some(IpAddr::V4(addr))
                }
                libc::AF_INET6 => {
                    let addr = addr as *mut libc::sockaddr_in6;
                    let addr = Ipv6Addr::from((*addr).sin6_addr.s6_addr);
                    Some(IpAddr::V6(addr))
                }
                _ => None,
            }
        }
    }

    fn iter(&self) -> Iter<'_> {
        Iter(Some(self))
    }
}

impl<'a> IntoIterator for &'a IfAddrs {
    type Item = &'a IfAddrsRef;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a IfAddrsRef {
    type Item = &'a IfAddrsRef;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

struct Iter<'a>(Option<&'a IfAddrsRef>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a IfAddrsRef;

    fn next(&mut self) -> Option<&'a IfAddrsRef> {
        let cur = match self.0 {
            Some(cur) => cur,
            None => return None,
        };

        self.0 = cur.next();
        Some(cur)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let addrs = IfAddrs::get().unwrap();
        println!("{:?}", addrs.iter().map(|a| a.addr()).collect::<Vec<_>>());
    }
}
