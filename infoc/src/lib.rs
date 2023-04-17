pub use bytes::Bytes;
pub use futures::{SinkExt, StreamExt};
pub use rkyv;
use rkyv::*;
pub use strum::VariantNames;
use strum_macros::*;
pub use tokio;
pub use tokio::net::*;
pub use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug, Display))]
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
    pub addr: Vec<std::net::Ipv4Addr>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct SysInfoV1 {
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
    pub microsoft_offices: Vec<String>,
    pub networkadapters: Vec<NetworkAdapter>,
}

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Details {
    pub count: u8,
    pub remarks: Option<String>,
}

#[derive(Archive, Serialize, Deserialize, Default, Debug, EnumVariantNames)]
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

#[derive(Archive, Serialize, Deserialize, Default, Debug)]
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

#[derive(Archive, Deserialize, Serialize, Debug)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Packet {
    pub magic: u32,
    pub version: VERSION,
    pub staffid: String,
    pub encinfo: AlignedVec,
}

#[cfg(debug_assertions)]
pub const CONNECTION_STR_CLIENT: &str = "localhost:8989";
#[cfg(debug_assertions)]
pub const CONNECTION_STR_SERVER: &str = "localhost:8989";

// #[cfg(debug_assertions)]
// pub const CONNECTION_STR_CLIENT: &str = "10.20.63.164:8989";
// #[cfg(debug_assertions)]
// pub const CONNECTION_STR_SERVER_CLIENT: &str = "0.0.0.0:8989";

#[cfg(not(debug_assertions))]
pub const CONNECTION_STR_CLIENT: &str = "asset.chonghwakl.edu.my:8989";
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
