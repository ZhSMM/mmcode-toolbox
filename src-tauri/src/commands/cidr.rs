//! CIDR 子网计算。

use ipnetwork::IpNetwork;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct CidrReq {
    pub input: String,
}

#[derive(Debug, serde::Serialize)]
pub struct CidrResp {
    pub network: String,
    pub broadcast: Option<String>,
    pub netmask: String,
    pub wildcard: String,
    pub total_addresses: String,
    pub usable_addresses: String,
    pub first_host: Option<String>,
    pub last_host: Option<String>,
    pub version: u8,
    pub is_private: bool,
}

fn v4_mask_from_prefix(prefix: u8) -> Ipv4Addr {
    if prefix == 0 { Ipv4Addr::from(u32::MAX) }
    else { Ipv4Addr::from((!0u32) << (32 - prefix)) }
}

fn v4_wildcard_from_prefix(prefix: u8) -> Ipv4Addr {
    if prefix == 0 { Ipv4Addr::from(0u32) }
    else { Ipv4Addr::from((!0u32) >> prefix) }
}

fn v6_mask_str(prefix: u8) -> String {
    let mut parts = Vec::new();
    for i in 0..8 {
        let start = i * 16;
        let end = start + 16;
        if prefix >= end { parts.push("ffff".to_string()); }
        else if prefix <= start { parts.push("0".to_string()); }
        else {
            let bits = prefix - start;
            let v: u16 = ((!0u16) << (16 - bits)) as u16;
            parts.push(format!("{:x}", v));
        }
    }
    parts.join(":")
}

fn v6_wildcard_str(prefix: u8) -> String {
    let mut parts = Vec::new();
    for i in 0..8 {
        let start = i * 16;
        let end = start + 16;
        if prefix >= end { parts.push("0".to_string()); }
        else if prefix <= start { parts.push("ffff".to_string()); }
        else {
            let bits = prefix - start;
            let v: u16 = ((!0u16) >> bits) as u16;
            parts.push(format!("{:x}", v));
        }
    }
    parts.join(":")
}

fn is_ula_v6(addr: Ipv6Addr) -> bool {
    let first = addr.segments()[0];
    (first & 0xfe00) == 0xfc00
}

fn last_v6(network: Ipv6Addr, prefix: u8) -> Ipv6Addr {
    let bits = 128 - prefix as u32;
    if bits == 0 { return network; }
    let v = u128::from(network) | (((1u128) << bits) - 1);
    Ipv6Addr::from(v)
}

fn ipv4_add(a: Ipv4Addr, n: u32) -> Option<Ipv4Addr> {
    let v = u32::from(a).checked_add(n)?;
    Some(Ipv4Addr::from(v))
}
fn ipv4_sub(a: Ipv4Addr, n: u32) -> Option<Ipv4Addr> {
    let v = u32::from(a).checked_sub(n)?;
    Some(Ipv4Addr::from(v))
}

#[tauri::command]
pub fn cidr_info(req: CidrReq) -> AppResult<CidrResp> {
    let net = IpNetwork::from_str(&req.input).map_err(|e| AppError::Invalid(format!("非法 CIDR: {e}")))?;
    match net {
        IpNetwork::V4(n) => {
            let network = n.network();
            let broadcast = n.broadcast();
            let prefix = n.prefix();
            let mask = v4_mask_from_prefix(prefix);
            let wildcard = v4_wildcard_from_prefix(prefix);
            let total = n.size() as u128;
            let usable = if prefix >= 31 { total } else { (total as u128).saturating_sub(2) };
            let first = if prefix >= 31 { Some(network) } else if let Some(next) = ipv4_add(network, 1) { Some(next) } else { None };
            let last = if prefix >= 31 { Some(broadcast) } else if let Some(prev) = ipv4_sub(broadcast, 1) { Some(prev) } else { None };
            Ok(CidrResp {
                network: network.to_string(),
                broadcast: Some(broadcast.to_string()),
                netmask: mask.to_string(),
                wildcard: wildcard.to_string(),
                total_addresses: total.to_string(),
                usable_addresses: usable.to_string(),
                first_host: first.map(|a| a.to_string()),
                last_host: last.map(|a| a.to_string()),
                version: 4,
                is_private: network.is_private() || network.is_loopback(),
            })
        }
        IpNetwork::V6(n) => {
            let network = n.network();
            let prefix = n.prefix();
            let total: u128 = if prefix == 128 { 1 } else { 1u128 << (128 - prefix as u32) };
            let last = last_v6(network, prefix);
            Ok(CidrResp {
                network: network.to_string(),
                broadcast: None,
                netmask: v6_mask_str(prefix),
                wildcard: v6_wildcard_str(prefix),
                total_addresses: total.to_string(),
                usable_addresses: total.to_string(),
                first_host: Some(network.to_string()),
                last_host: Some(last.to_string()),
                version: 6,
                is_private: is_ula_v6(network),
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IpLookupReq {
    pub ip: String,
}

#[derive(Debug, serde::Serialize)]
pub struct IpLookupResp {
    pub ip: String,
    pub version: u8,
    pub is_private: bool,
    pub is_loopback: bool,
    pub is_link_local: bool,
    pub is_multicast: bool,
    pub is_unspecified: bool,
    pub is_documentation: bool,
    pub reverse_dns_hint: String,
}

#[tauri::command]
pub fn ip_info(req: IpLookupReq) -> AppResult<IpLookupResp> {
    let ip = IpAddr::from_str(&req.ip).map_err(|e| AppError::Invalid(format!("非法 IP: {e}")))?;
    let (version, is_private, is_loopback, is_link_local, is_multicast, is_unspecified, is_documentation) = match ip {
        IpAddr::V4(v) => (
            4u8,
            v.is_private() || v.is_loopback() || v.is_link_local(),
            v.is_loopback(),
            v.is_link_local(),
            v.is_multicast(),
            v.is_unspecified(),
            v.is_documentation(),
        ),
        IpAddr::V6(v) => {
            let ll = v.segments()[0] == 0xfe80;
            let doc = v.segments()[0] == 0x2001 && v.segments()[1] == 0x0db8;
            (
                6u8,
                v.is_loopback() || ll || is_ula_v6(v),
                v.is_loopback(),
                ll,
                v.is_multicast(),
                v.is_unspecified(),
                doc,
            )
        }
    };
    let rdns = match ip {
        IpAddr::V4(v) => {
            let octets: Vec<u8> = v.octets().to_vec();
            format!("{}.{}.{}.{}.in-addr.arpa", octets[3], octets[2], octets[1], octets[0])
        }
        IpAddr::V6(v) => {
            let mut nibbles = String::new();
            for seg in v.segments().iter().rev() {
                nibbles.push_str(&format!("{:04x}.{:04x}.", seg & 0xff, seg >> 8));
            }
            format!("{}ip6.arpa", nibbles)
        }
    };
    Ok(IpLookupResp {
        ip: ip.to_string(), version, is_private, is_loopback, is_link_local, is_multicast, is_unspecified, is_documentation,
        reverse_dns_hint: rdns,
    })
}