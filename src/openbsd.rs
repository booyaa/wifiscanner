use nix::libc::{freeifaddrs, getifaddrs, ifaddrs};
use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag, SockProtocol};
use std::ffi::{CStr, CString};
use std::os::unix::io::RawFd;
use std::mem;
use std::ptr;
use super::{Wifi, Error};

pub struct Socket {
    pub ifname: String,
    fd: RawFd,
    cifname: CString,
}

fn find_interface() -> Result<Socket, Error> {
    fn is_wireless(sock: RawFd, iname: &CStr) -> bool {
        let mut mediareq = raw::ifmediareq::new(iname);

        // if we don't support this ioctl, then we're definitely not wireless
        if let Err(_) = unsafe { raw::get_ifmedia(sock, &mut mediareq) } {
            return false;
        };

        (mediareq.current & raw::IFM_MASK) == raw::IFM_IEEE80211
    }

    let sock = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        SockProtocol::Udp,
    ).or_else(|e| Err(Error::NixError(e)))?;

    let mut wireless: Vec<String> = Vec::new();

    unsafe {
        let mut interfaces: *mut ifaddrs = ptr::null_mut();
        let head = interfaces;

        // get list of interfaces
        getifaddrs(&mut interfaces);

        while (*interfaces).ifa_next != 0 as *mut ifaddrs {
            // extract the name
            let iname = CStr::from_ptr((*interfaces).ifa_name);

            // determine if we have a wireless interface
            if is_wireless(sock, &iname) {
                let name = String::from(iname.to_str().unwrap());
                if !wireless.contains(&name) {
                    wireless.push(name);
                }
            }

            interfaces = (*interfaces).ifa_next;
        }

        freeifaddrs(head);
    }

    if wireless.len() == 1 {
        let ifname = wireless.pop().unwrap();
        let cifname = CString::new(ifname.as_str()).unwrap();

        Ok(Socket {
            ifname,
            fd: sock,
            cifname,
        })
    } else if wireless.len() == 0 {
        Err(Error::DiscoveryError("no wireless devices found"))
    } else {
        Err(Error::DiscoveryError("too many wireless devices found"))
    }
}

pub fn scan() -> Result<Vec<Wifi>, Error> {
    let sock = find_interface()?;

    let req = raw::ifreq::new(sock.cifname.as_c_str());
    // put the interface into scan mode
    unsafe { raw::set_80211scan(sock.fd, &req)? };

    let raw_results: [raw::ieee80211_nodereq; 512] = [raw::ieee80211_nodereq::default(); 512];
    let mut raw_container = raw::ieee80211_nodereq_all::new(
        sock.cifname.as_c_str(),
        &raw_results as *const raw::ieee80211_nodereq,
        mem::size_of_val(&raw_results),
    );

    // collect the results
    unsafe { raw::get_allnodes(sock.fd, &mut raw_container)? };

    let mut results: Vec<Wifi> = Vec::new();

    for i in 0..raw_container.nodes as usize {
        let bssid = raw_results[i].bssid;
        results.push(Wifi {
            mac: format!("{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5]),
            ssid: raw::nwid_to_str(&raw_results[i].nwid)?,
            channel: format!("{}", raw_results[i].channel),
            signal_level: format!("{}", raw_results[i].rssi),
            security: format!("{}", raw_results[i].capinfo),
        });
    }

    Ok(results)
}

#[allow(non_camel_case_types)]
mod raw {
    use nix::libc::{c_int, c_short, c_uint, c_void, size_t, sockaddr, uint64_t};
    use std::ffi;
    use std::mem;
    use std::ptr;
    use std::result;
    use std::string::FromUtf8Error;

    const IFNAMSIZ: usize = 16;
    const IEEE80211_ADDR_LEN: usize = 6; // net80211/ieee80211.h
    const IEEE80211_NWID_LEN: usize = 32; // net80211/ieee80211.h
    const IEEE80211_RATE_MAXSIZE: usize = 15; // net80211/ieee80211.h

    pub const IFM_IEEE80211: u64 = 0x400;
    pub const IFM_MASK: u64 = 0xff00;

    fn cstr_to_ifname(name: &ffi::CStr) -> [i8; 16] {
        let mut name16: [i8; 16] = [0; 16];
        let name_arr = name.to_bytes_with_nul();

        for i in 0..14 {
            if i < name_arr.len() {
                name16[i] = name_arr[i] as i8;
            }
        }

        name16
    }

    pub fn nwid_to_str(nwid: &[u8]) -> result::Result<String, FromUtf8Error> {
        let mut v = nwid.to_vec();

        if nwid[0] == 0 {
            return Ok(String::new());
        }

        v.retain(|&x| x != 0);

        String::from_utf8(v)
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct ifmediareq {
        pub name: [i8; IFNAMSIZ],
        pub current: u64,
        pub mask: u64,
        pub status: u64,
        pub active: u64,
        pub count: c_int,
        pub ulist: *mut u64,
    }

    impl ifmediareq {
        pub fn new(name: &ffi::CStr) -> Self {
            ifmediareq {
                name: cstr_to_ifname(name),
                current: 0,
                mask: 0,
                status: 0,
                active: 0,
                count: 0,
                ulist: ptr::null_mut(),
            }
        }
    }

    #[repr(C)]
    union ifr_ifru {
        pub addr: sockaddr,
        pub dstaddr: sockaddr,
        pub broadaddr: sockaddr,
        pub flags: c_short,
        pub metric: c_int,
        pub media: uint64_t,
        pub data: *mut c_void
    }

    #[repr(C)]
    pub struct ifreq {
        name: [i8; IFNAMSIZ],
        ifru: ifr_ifru,
    }

    impl ifreq {
        pub fn new(name: &ffi::CStr) -> Self {
            ifreq {
                name: cstr_to_ifname(name),
                ifru: unsafe { mem::zeroed() },
            }
        }
    }

    // /usr/include/net80211/ieee80211_ioctl.h
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct ieee80211_nodereq {
        pub name: [i8; IFNAMSIZ],

        pub macaddr: [u8; IEEE80211_ADDR_LEN],
        pub bssid: [u8; IEEE80211_ADDR_LEN],
        pub nwid_len: u8,
        pub nwid: [u8; IEEE80211_NWID_LEN],

        pub channel: u16,
        pub chan_flags: u16,
        pub nrates: u8,
        pub rates: [u8; IEEE80211_RATE_MAXSIZE],

        pub rssi: i8,
        pub max_rssi: i8,
        pub tstamp: [u8; 8],
        pub intval: u16,
        pub capinfo: u16,
        pub erp: u8,
        pub pwrsave: u8,
        pub associd: u16,
        pub txseq: u16,
        pub rxseq: u16,
        pub fails: u32,
        pub inact: u32,
        pub txrate: u8,
        pub state: u16,

        pub rsnproto: c_uint,
        pub rsnciphers: c_uint,
        pub rsnakms: c_uint,

        pub flags: u8,

        pub htcaps: u16,
        pub rxmcs: [u8; 10], // howmany(80, NBBY) where NBBY = 8
        pub max_rxrate: u16,
        pub tx_mcs_set: u8,
        pub txmcs: u8,
    }

    // /usr/include/net80211/ieee80211_ioctl.h
    #[repr(C)]
    #[derive(Debug)]
    pub struct ieee80211_nodereq_all {
        pub name: [i8; IFNAMSIZ],
        pub nodes: c_int,
        pub size: size_t,
        pub node: *const ieee80211_nodereq,
        pub flags: u8,
    }

    impl Default for ieee80211_nodereq_all {
        fn default() -> Self {
            unsafe { mem::zeroed() }
        }
    }

    impl ieee80211_nodereq_all {
        pub fn new(name: &ffi::CStr, node: *const ieee80211_nodereq, size: size_t) -> Self {
            let mut ret = Self::default();

            ret.name = cstr_to_ifname(name);
            ret.node = node;
            ret.size = size;
            ret
        }
    }


    const SIOCIF_MAGIC: u8 = b'i';
    const SIOCGIFMEDIA: u8 = 56;
    const SIOCS80211SCAN: u8 = 210;
    const SIOCG80211ALLNODES: u8 = 214;

    ioctl!(readwrite get_ifmedia with SIOCIF_MAGIC, SIOCGIFMEDIA; ifmediareq);
    ioctl!(write_ptr set_80211scan with SIOCIF_MAGIC, SIOCS80211SCAN; ifreq);
    ioctl!(readwrite get_allnodes with SIOCIF_MAGIC, SIOCG80211ALLNODES; ieee80211_nodereq_all);
}
