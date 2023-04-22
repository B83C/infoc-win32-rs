#![feature(generic_arg_infer)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// pub use async_std;
// pub use async_std::net::*;
pub use bytes::Bytes;
pub use enum_iterator::{all, Sequence};
pub use futures::{SinkExt, StreamExt};
pub use rkyv;
use rkyv::with::Skip;
use rkyv::*;
// pub use rkyv_codec::{archive_stream, RkyvWriter, VarintLength};
pub use lazy_static;
pub use std::net::IpAddr;
pub use strum::VariantNames;
use strum_macros::*;
pub use tokio;
pub use tokio::net::*;
pub use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(
    Archive,
    Serialize,
    Deserialize,
    serde_repr::Deserialize_repr,
    Debug,
    Eq,
    PartialEq,
    EnumVariantNames,
    Sequence,
    Default,
)]
#[archive(check_bytes)]
#[repr(u16)]
#[serde(untagged)]
#[archive_attr(derive(Debug, Display))]
pub enum DiskType {
    #[default]
    None = 0,
    HDD = 3,
    SSD = 4,
    SCM = 5,
}

#[derive(
    Archive, Serialize, Deserialize, serde_repr::Deserialize_repr, Debug, Eq, PartialEq, Default,
)]
#[archive(check_bytes)]
#[repr(u16)]
#[serde(untagged)]
#[archive_attr(derive(Debug, Display))]
pub enum NetConnectionStatus {
    #[default]
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Disconnecting = 3,
    Hardware_Not_Present = 4,
    Hardware_Disabled = 5,
    Hardware_Malfunction = 6,
    Media_Disconnected = 7,
    Authenticating = 8,
    Authentication_Succeeded = 9,
    Authentication_Failed = 10,
    Invalid_Address = 11,
    Credentials_Required = 12,
}

// #[derive(Archive, Serialize, Deserialize, Debug)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// pub struct Disk {
//     pub disktype: DiskType,
//     pub disksize: u64,
//     pub model: Option<String>,
// }

// #[derive(Archive, Serialize, Deserialize, Debug)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// pub struct NetworkAdapter {
//     pub physical_address: [u8; 6],
//     pub addr: Vec<std::net::Ipv4Addr>,
// }

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_ComputerSystem {
    pub DNSHostName: Option<String>,
    pub Manufacturer: Option<String>,
    pub Model: Option<String>,
    pub TotalPhysicalMemory: u64,
    // pub SystemFamily: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_BIOS {
    pub SerialNumber: Option<String>,
    pub Manufacturer: Option<String>,
    pub Description: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_Printer {
    pub Attributes: u32,
    pub Name: String,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_Processor {
    pub Name: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_OperatingSystem {
    pub Caption: Option<String>,
    pub OSArchitecture: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_DiskDrive {
    pub Model: Option<String>,
    pub Manufacturer: Option<String>,
    pub Size: u64,
    #[serde(skip)]
    pub MediaType: DiskType,
    #[with(Skip)]
    pub DeviceID: String,
    #[with(Skip)]
    pub Capabilities: Vec<u16>,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct MSFT_PhysicalDisk {
    pub Size: u64,
    pub MediaType: DiskType,
    pub Model: Option<String>,
    pub Manufacturer: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum Disk {
    MSFT(Vec<MSFT_PhysicalDisk>),
    W32(Vec<Win32_DiskDrive>),
}

#[derive(Archive, Serialize, Deserialize, serde::Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Win32_NetworkAdapter {
    pub MacAddress: Option<String>,
    pub AdapterTypeID: Option<u16>,
    pub Description: Option<String>,
    pub NetConnectionStatus: NetConnectionStatus,
    #[with(Skip)]
    pub PNPDeviceID: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct SysInfoV1 {
    pub bios: Win32_BIOS,
    pub os: Win32_OperatingSystem,
    pub cs: Win32_ComputerSystem,
    pub cpu: Win32_Processor,
    pub net: Vec<Win32_NetworkAdapter>,
    pub disks: Disk,
    pub msoffice: Vec<String>,
}

// #[derive(Default, Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// pub struct SysInfoV1 {
//     pub cpu: String,
//     pub os: String,
//     pub disks: Vec<Disk>,
//     pub memory_size: u64,
//     pub serial_number: String,
//     pub product_name: String,
//     pub manufacturer: String,
//     pub sku_number: String,
//     pub version: String,
//     pub uuid: Option<[u8; 16]>,
//     pub microsoft_offices: Vec<String>,
//     pub networkadapters: Vec<NetworkAdapter>,
// }

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Details {
    pub count: u8,
    pub remarks: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, Default, Debug, EnumVariantNames, Sequence)]
#[repr(u8)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug, Eq, PartialEq))]
pub enum Item {
    Mouse,
    Keyboard,
    Printer,
    Others,
    #[default]
    None,
}

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Accessories {
    pub item: Item,
    pub details: Details,
}

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Position {
    pub department: u8,
    pub position: u8,
    pub remarks: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Infoc {
    pub accessories: Vec<Accessories>,
    pub pos: Position,
    pub sysinfo: SysInfoV1,
}

pub const DEPARTMENT: [&str; 25] = [
    "行政办事处",
    "教务处",
    "训育处",
    "辅导处",
    "升学及国际事务处",
    "教职研修处",
    "联课活动处",
    "福利处",
    "事务处",
    "会计处",
    "贩卖部",
    "资讯中心",
    "资源中心",
    "学术竞赛处",
    "体育处",
    "义工处",
    "舍务处",
    "媒体中心",
    "保安及交通处",
    "科技研发处",
    "文史馆",
    "校友联络室",
    "科学馆",
    "出版组",
    "其他",
];

pub const ADAPTERTYPE: [&str; 14] = [
    "Ethernet 802.3",
    "Token Ring 802.5",
    "Fiber Distributed Data Interface",
    "Wide Area Network",
    "LocalTalk",
    "Ethernet using DIX header format",
    "ARCNET",
    "ARCNET",
    "ATM",
    "Wireless",
    "Infrared Wireless",
    "Bpc",
    "CoWan",
    "1394",
];

#[derive(Archive, Deserialize, Serialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Packet {
    pub magic: u32,
    pub version: VERSION,
    pub staffid: String,
    pub encinfo: Vec<u8>,
}

#[cfg(debug_assertions)]
pub const CONNECTION_STR_CLIENT: &[&str] = &["localhost:8989", "192.168.1.53:8989"];
#[cfg(debug_assertions)]
pub const CONNECTION_STR_SERVER: &str = "0.0.0.0:8989";

// #[cfg(debug_assertions)]
// pub const CONNECTION_STR_CLIENT: &str = "10.20.63.164:8989";
// #[cfg(debug_assertions)]
// pub const CONNECTION_STR_SERVER_CLIENT: &str = "0.0.0.0:8989";

// #[cfg(not(debug_assertions))]
// pub const CONNECTION_STR_CLIENT: &str = "10.15.9.36:8989";
#[cfg(not(debug_assertions))]
pub const CONNECTION_STR_CLIENT: [&str; _] = ["asset.chonghwakl.edu.my:8989"];
#[cfg(not(debug_assertions))]
pub const CONNECTION_STR_SERVER: &str = "0.0.0.0:8989";

pub const MAGIC: u32 = u32::from_le_bytes([b'C', b'H', b'K', b'L']);

#[derive(Archive, Deserialize, Serialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum VERSION {
    V1,
}

pub const CUR_VERSION: VERSION = VERSION::V1;

pub fn decode<'a>(bytes: &'a [u8]) -> &'a ArchivedInfoc {
    rkyv::check_archived_root::<Infoc>(bytes).unwrap()
}

pub fn encode(info: &Infoc) -> AlignedVec {
    rkyv::to_bytes::<_, 16384>(info).unwrap()
}

pub use kip_db::kernel::hash_kv::*;
pub use kip_db::kernel::*;
pub use kip_db::*;

pub const DB_NAME_TEMP: &str = "kv_temp.db";

#[cfg(debug_assertions)]
pub const DB_NAME: &str = "kv_debug.db";

#[cfg(not(debug_assertions))]
pub const DB_NAME: &str = "kv.db";

// pub use rust_kv::{KvEngine, KvStore, Result};
// pub const DB_NAME_TEMP: &str = "kv_temp.db";

// #[cfg(debug_assertions)]
// pub const DB_NAME: &str = "kv_debug.db";

// #[cfg(not(debug_assertions))]
// pub const DB_NAME: &str = "kv.db";

// pub use microkv::MicroKV;

// #[cfg(debug_assertions)]
// const DB_NAME: &str = "microkv_debug.db";

// #[cfg(not(debug_assertions))]
// const DB_NAME: &str = "microkv.db";

// const DB_NAME_TEMP: &str = "microkv_temp.db";

// pub fn microkv_open() -> Result<MicroKV, Box<dyn std::error::Error>> {
//     Ok(
//         MicroKV::open_with_base_path(DB_NAME, std::env::current_dir()?)
//             .expect("Failed to create MicroKV On-disk database"),
//     )
// }

// pub fn microkv_open_temp() -> Result<MicroKV, Box<dyn std::error::Error>> {
//     Ok(
//         MicroKV::open_with_base_path(DB_NAME_TEMP, std::env::current_dir()?)
//             .expect("Failed to create MicroKV On-disk database"),
//     )
// }
