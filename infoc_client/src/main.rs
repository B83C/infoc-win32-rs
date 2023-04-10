use inquire::{Confirm, CustomType, Select, Text};
use serde::Deserialize;
use tokio::net::TcpStream;
use rkyv::*;
use std::error::Error;
use strum::VariantNames;
use strum_macros::*;
// use sysinfo::{Component, Cpu, Disk, MacAddr, NetworkExt, NetworksExt, System, SystemExt, User};
use wmi::WMIDateTime;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_OperatingSystem")]
#[serde(rename_all = "PascalCase")]
struct OperatingSystem {
    caption: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystem")]
#[serde(rename_all = "PascalCase")]
struct ComputerSystem {
    name: String,
    model: String,
    manufacturer: String,
    systemskunumber: String,
    systemtype: String,
    installdate: Option<WMIDateTime>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystemProduct")]
#[serde(rename_all = "PascalCase")]
struct ComputerSystemProduct {
    identifyingnumber: Option<String>,
    name: String,
    skunumber: Option<String>,
    vendor: Option<String>,
    version: Option<String>,
    uuid: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_NetworkAdapter")]
#[serde(rename_all = "PascalCase")]
struct NetworkAdapter {
    macaddress: Option<String>,
    physicaladapter : bool,
}

#[derive(Deserialize)]
struct SysInfo {
    os : OperatingSystem,
    cs: ComputerSystem,
    csp: ComputerSystemProduct,
    na : NetworkAdapter,
}

#[derive(Default, Debug)]
struct Details {
    count: u8,
    remarks: Option<String>,
}

#[derive(Default, Debug, EnumVariantNames)]
enum Item {
    Mouse,
    Keyboard,
    Printer,
    Others,
    #[default]
    None,
}

#[derive(Default, Debug)]
struct Accessories {
    item: Item,
    details: Details,
}

#[derive(Default, Debug)]
struct Position {
    department: u8,
    position: u8,
    remarks: Option<String>,
}

#[derive(Default, Debug)]
struct Info<'a> {
    staffid: String,
    accessories: &'a [Accessories],
    pos: Position,
    sysinfo: String,
}

const DEPARTMENT: [&str; 25] = [
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;
    let results: Vec<OperatingSystem> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }
    let results: Vec<ComputerSystem> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }

    let results: Vec<ComputerSystemProduct> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }
    let results: Vec<NetworkAdapter> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }

    let mut stream = TcpStream::connect("asset.chonghwakl.edu.my:8989").await?;

    let staffid = Text::new("Staff ID : ").prompt()?;

    let department = Select::new("Department", DEPARTMENT.to_vec()).raw_prompt()?;
    let mut remarks = None;
    if department.index == (DEPARTMENT.len() - 1) {
        remarks = Some(Text::new("Please enter your department : ").prompt()?);
    }
    let department = department.index as u8;
    let position = CustomType::<u8>::new("Position : ").prompt()?;

    let pos = Position {
        department,
        position,
        remarks,
    };

    let mut accessories: Vec<Accessories> = vec![];

    loop {
        let ans = Confirm::new("Any accessories to add?")
            .with_default(false)
            .prompt()?;

        if ans {
            let item = Select::new("Item", Item::VARIANTS.to_vec()).raw_prompt()?;
            let item: Item = unsafe { std::mem::transmute(item.index as u8) };

            let count = CustomType::<u8>::new("Count : ").prompt()?;

            let ans = Confirm::new("Any remarks to add?")
                .with_default(false)
                .prompt()?;

            let mut remarks = None;
            if ans {
                remarks = Some(Text::new("Enter here : ").prompt()?);
            }

            let details = Details { count, remarks };
            accessories.push(Accessories { item, details })
        } else {
            break;
        }
    }

    dbg!(accessories);

    //MANUFACTURER, MODEL, SERIAL NUMBER, CPU, RAM, DISK, SYSTEM, MSOFFICE

    // let sys = System::new_all();
    // let net = sys.networks();
    // for (i, n) in net {
    //     dbg!(n);
    // }

    // dbg!(sys.global_cpu_info());
    // dbg!(sys.name());
    let ans = Confirm::new("Do you want to submit now?")
        .with_default(true)
        .prompt()?;

    if ans {
        println!("Submited");
    }


    Ok(())
}
