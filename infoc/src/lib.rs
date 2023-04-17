pub use bytes::Bytes;
pub use futures::{SinkExt, StreamExt};
pub use rkyv;
use rkyv::*;
use serde::Deserialize;
pub use strum::VariantNames;
use strum_macros::*;
pub use tokio;
pub use tokio::net::*;
pub use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum DiskType {
    SSD,
    HDD,
    #[default]
    None,
}

#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Disk {
    pub disktype: DiskType,
    pub disksize: u64,
    pub model: String,
}

#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct NetworkAdapter {
    pub physical_address: [u8; 6],
}

#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct SysInfo {
    pub cpu: String,
    pub os: String,
    pub disks: Vec<Disk>,
    pub memory_size: u64,
    pub serial_number: String,
    pub product_name: String,
    pub manufacturer: String,
    pub sku_number: String,
    pub version: String,
    pub uuid: Option<[u8; 16]>,
    pub microsoft_office: Vec<String>,
    pub networkadapters: Vec<NetworkAdapter>,
}

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// #[serde(rename = "Win32_OperatingSystem")]
// #[serde(rename_all = "PascalCase")]
// pub struct OperatingSystem {
//     caption: String,
// }

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// #[serde(rename = "Win32_ComputerSystem")]
// #[serde(rename_all = "PascalCase")]
// pub struct ComputerSystem {
//     name: String,
//     model: String,
//     manufacturer: String,
//     systemskunumber: String,
//     systemtype: String,
//     totalphysicalmemory: u64,
//     username: String,
// }

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// #[serde(rename = "Win32_ComputerSystemProduct")]
// #[serde(rename_all = "PascalCase")]
// pub struct ComputerSystemProduct {
//     identifyingnumber: Option<String>,
//     name: String,
//     skunumber: Option<String>,
//     vendor: Option<String>,
//     version: Option<String>,
// }

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// #[serde(rename = "Win32_DiskDrive")]
// #[serde(rename_all = "PascalCase")]
// pub struct DiskDrive {
//     caption: String,
//     size: u64,
// }

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// #[serde(rename = "Win32_NetworkAdapterConfiguration")]
// #[serde(rename_all = "PascalCase")]
// pub struct NetworkAdapterConfiguration {
//     dnshostname: Option<String>,
//     ipaddress: Vec<String>,
//     dhcpserver: Option<String>,
//     macaddress: Option<String>,
// }

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// #[serde(rename = "Win32_Processor")]
// #[serde(rename_all = "PascalCase")]
// pub struct Processor {
//     caption: String,
//     name: String,
//     manufacturer: String,
// }

// #[derive(Archive, Serialize, Deserialize, Debug, Default)]
// #[archive(check_bytes)]
// #[archive_attr(derive(Debug))]
// pub struct SysInfo {
//     os: OperatingSystem,
//     cs: ComputerSystem,
//     csp: ComputerSystemProduct,
//     nac: NetworkAdapterConfiguration,
// }

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Details {
    pub count: u8,
    pub remarks: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, Default, Debug, EnumVariantNames)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
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

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Infoc {
    pub accessories: Vec<Accessories>,
    pub pos: Position,
    pub sysinfo: SysInfo,
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

#[derive(Archive, Deserialize, Serialize, Debug)]
#[archive_attr(derive(Debug))]
pub struct Packet {
    pub magic: u32,
    pub version: u16,
    pub staffid: String,
}

#[cfg(debug_assertions)]
pub const CONNECTION_STR: &str = "localhost:8989";

#[cfg(not(debug_assertions))]
pub const CONNECTION_STR: &str = "asset.chonghwakl.edu.my:8989";

pub const MAGIC: u32 = u32::from_le_bytes([b'C', b'H', b'K', b'L']);
pub const VERSION: u16 = 1;

pub fn decode<'a>(bytes: &'a [u8]) -> &'a ArchivedInfoc {
    rkyv::check_archived_root::<Infoc>(bytes).unwrap()
}

pub fn encode(info: &Infoc) -> AlignedVec {
    rkyv::to_bytes::<_, 16384>(info).unwrap()
}
