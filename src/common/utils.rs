use ::protocol::{Array, BinSafeStr, BulkStr, Resp};
use caseless;
use crc16::{State, XMODEM};
use std::net::{SocketAddr, ToSocketAddrs};
use std::str;

pub trait ThreadSafe: Send + Sync + 'static {}

#[derive(Debug)]
pub struct CmdParseError {}

pub fn has_flags(s: &str, delimiter: char, flag: &'static str) -> bool {
    s.split(delimiter)
        .any(|s| caseless::canonical_caseless_match_str(s, flag))
}

pub fn revolve_first_address(address: &str) -> Option<SocketAddr> {
    match address.to_socket_addrs() {
        Ok(mut address_list) => match address_list.next() {
            Some(address) => Some(address),
            None => {
                error!("can not resolve address {}", address);
                None
            }
        },
        Err(e) => {
            error!("failed to parse address {} {:?}", address, e);
            None
        }
    }
}

pub fn get_key(resp: &Resp) -> Option<BinSafeStr> {
    match resp {
        Resp::Arr(Array::Arr(ref resps)) => resps.get(1).and_then(|resp| match resp {
            Resp::Bulk(BulkStr::Str(ref s)) => Some(s.clone()),
            Resp::Simple(ref s) => Some(s.clone()),
            _ => None,
        }),
        _ => None,
    }
}

pub fn get_commands(resp: &Resp) -> Option<Vec<String>> {
    match resp {
        Resp::Arr(Array::Arr(ref resps)) => {
            let mut commands = vec![];
            for resp in resps.iter() {
                match resp {
                    Resp::Bulk(BulkStr::Str(s)) => {
                        commands.push(str::from_utf8(s).ok()?.to_string())
                    }
                    _ => return None,
                }
            }
            Some(commands)
        }
        _ => None,
    }
}

pub fn gen_moved(slot: usize, addr: String) -> String {
    format!("MOVED {} {}", slot, addr)
}

pub fn get_slot(key: &[u8]) -> usize {
    // TODO: support hash tag.
    State::<XMODEM>::calculate(key) as usize % SLOT_NUM
}

pub const OLD_EPOCH_REPLY: &str = "old_epoch";
pub const SLOT_NUM: usize = 16384;

pub const MIGRATING_TAG: &str = "MIGRATING";
pub const IMPORTING_TAG: &str = "IMPORTING";
