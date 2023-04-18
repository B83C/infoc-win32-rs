use infoc::rkyv;
use infoc::*;
use inquire::{Confirm, CustomType, Select, Text};
use std::error::Error;
// use sysinfo::{Cpu, Disk, MacAddr, NetworkExt, NetworksExt, RefreshKind, System, SystemExt, User};
// use wmi::WMIDateTime;
// use wmi::{COMLibrary, WMIConnection};
use ::windows::{
    core::*, Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::Ioctl::*,
    Win32::System::SystemInformation::*, Win32::System::WindowsProgramming::*,
    Win32::System::IO::*,
};
use core::ffi::c_void;
use default_net::*;
use smbioslib::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::size_of;
use std::result::Result;
use winreg::enums::*;
use winreg::RegKey;

#[inline]
fn prompt() -> Result<(String, Infoc), Box<dyn Error>> {
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

    Ok((
        staffid,
        Infoc {
            accessories,
            pos,
            sysinfo: Default::default(),
        },
    ))
}

fn get_sys_info() -> SysInfoV1 {
    let mut sysinfo: SysInfoV1 = Default::default();
    let mut mounts = unsafe { GetLogicalDrives() };
    let mut accumulator = 0u8;

    dbg!(mounts);

    let mut drives = Vec::with_capacity(16);
    let mut disks = HashMap::<(u32, u32), ()>::with_capacity(16);

    loop {
        if mounts == 0 {
            break;
        };
        let tzcnt = mounts.trailing_zeros() + 1;
        accumulator += (tzcnt - 1) as u8;
        mounts >>= tzcnt;
        let drive_name = [b'\\', b'\\', b'.', b'\\', (b'A' + accumulator), b':', 0];
        let handle = unsafe {
            CreateFileA(
                PCSTR(&drive_name as *const u8),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                HANDLE(0),
            )
            .expect("Handle failure")
        };

        unsafe {
            let mut dn: STORAGE_DEVICE_NUMBER = Default::default();
            DeviceIoControl(
                handle,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some(&mut dn as *mut STORAGE_DEVICE_NUMBER as *mut c_void),
                size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                None,
                None,
            )
            .expect("IOCTL_STORAGE_GET_DEVICE_NUMBER has failed");

            let ent = (dn.DeviceType, dn.DeviceNumber);

            // dbg!(&ent);

            // if ent.0 != DRIVE_FIXED && ent.0 != DRIVE_REMOVABLE {
            //     continue;
            // }

            match disks.entry(ent) {
                Entry::Occupied(_) => {
                    continue;
                }
                Entry::Vacant(v) => {
                    v.insert(());
                }
            }
        }

        let mut squery = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceSeekPenaltyProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };

        let mut desc: DEVICE_SEEK_PENALTY_DESCRIPTOR = Default::default();
        let mut dgeo: DISK_GEOMETRY_EX = Default::default();
        let mut model: STORAGE_ADAPTER_SERIAL_NUMBER = Default::default();

        let mut size: u32 = 0;
        let mut model_name = None;

        unsafe {
            let res = DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some(&squery as *const STORAGE_PROPERTY_QUERY as *const c_void),
                size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some(&mut desc as *mut DEVICE_SEEK_PENALTY_DESCRIPTOR as *mut c_void),
                size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32,
                Some(&mut size),
                None,
            )
            .as_bool();
            if res {
                eprintln!("Unable to query storage property");
            }
            // dbg!(&handle);
            let res = DeviceIoControl(
                handle,
                IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                None,
                0,
                Some(&mut dgeo as *mut DISK_GEOMETRY_EX as *mut c_void),
                size_of::<DISK_GEOMETRY_EX>() as u32,
                Some(&mut size),
                None,
            )
            .as_bool();
            if res {
                eprintln!("Unable to query storage geometry");
            }
            squery.PropertyId = StorageAdapterSerialNumberProperty;
            let res = DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some(&squery as *const STORAGE_PROPERTY_QUERY as *const c_void),
                size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some(&mut model as *mut STORAGE_ADAPTER_SERIAL_NUMBER as *mut c_void),
                size_of::<STORAGE_ADAPTER_SERIAL_NUMBER>() as u32,
                Some(&mut size),
                None,
            )
            .as_bool();
            if res {
                let range_end = model
                    .SerialNumber
                    .iter()
                    .position(|&c| c == '\0' as u16 || c == ' ' as u16)
                    .unwrap_or(model.SerialNumber.len());
                model_name = Some(String::from_utf16_lossy(&model.SerialNumber[..range_end]));
            }
            CloseHandle(handle);
        }

        let disktype = if !desc.IncursSeekPenalty.as_bool() {
            DiskType::SSD
        } else {
            DiskType::HDD
        };

        sysinfo.disks.push(Disk {
            disktype,
            disksize: dgeo.DiskSize as u64,
            model: model_name,
        });

        drives.push(desc);
    }

    match table_load_from_device() {
        Ok(data) => {
            let processor = data
                .first::<SMBiosProcessorInformation>()
                .expect("Unable to retrieve processor info");
            sysinfo.cpu = processor.processor_version().to_string();
            // dbg!(&sysinfo.cpu);
            let system = data
                .first::<SMBiosSystemInformation>()
                .expect("Unable to retrieve System info");
            sysinfo.serial_number = system.serial_number().to_string();
            // dbg!(&sysinfo.serial_number);
            sysinfo.product_name = system.product_name().to_string();
            // dbg!(&sysinfo.product_name);
            sysinfo.manufacturer = system.manufacturer().to_string();
            // dbg!(&sysinfo.manufacturer);
            sysinfo.sku_number = system.sku_number().to_string();
            // dbg!(&sysinfo.sku_number);
            sysinfo.version = system.version().to_string();
            // dbg!(&sysinfo.version);
            sysinfo.uuid = if let Some(SystemUuidData::Uuid(x)) = system.uuid() {
                Some(x.raw)
            } else {
                None
            };
            // dbg!(&sysinfo.uuid);
        }
        Err(_) => {}
    }

    sysinfo.os = if let Some(os) = os_info::get().edition() {
        os.to_string()
    } else {
        Text::new("OS version can't be detected, please enter manually : ")
            .prompt()
            .expect("Unable to get input")
    };

    let office = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        .expect("Unable to open registry subkey");

    let mut offices: Vec<String> = Vec::new();
    office.enum_keys().map(|x| x.unwrap()).for_each(|x| {
        if x.starts_with("Office") {
            let subkey = office.open_subkey(x);

            if let Ok(subkey) = subkey {
                if let Ok(x) = subkey.get_value::<String, _>("DisplayName") {
                    offices.push(x);
                }
            }
        }
    });

    sysinfo.microsoft_offices = if offices.len() != 0 {
        offices
    } else {
        vec![Text::new("Offices not detected, enter manually")
            .prompt()
            .expect("Unable to get input")]
    };

    let interfaces = get_interfaces()
        .into_iter()
        .filter_map(|x| match x.if_type {
            interface::InterfaceType::Wireless80211 | interface::InterfaceType::Ethernet => {
                if let Some(mac_addr) = x.mac_addr {
                    Some(NetworkAdapter {
                        physical_address: mac_addr.octets(),
                        addr: x.ipv4.iter().map(|x| x.addr).collect(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    // dbg!(&interfaces);
    sysinfo.networkadapters = interfaces;
    // let adapters = get_adapters()
    //     .expect("Unable to get network adapters")
    //     .iter()
    //     .filter_map(|x| match x.if_type() {
    //         IfType::EthernetCsmacd | IfType::Ieee80211 => match x.oper_status() {
    //             OperStatus::IfOperStatusUp
    //             | OperStatus::IfOperStatusUnknown
    //             | OperStatus::IfOperStatusDown => Some(NetworkAdapter {
    //                 physical_address: x.physical_address().map(|x| x.to_owned()),
    //                 addr: x.ip_addresses().to_vec(),
    //             }),
    //             _ => None,
    //         },
    //         _ => None,
    //     })
    //     .collect();
    // dbg!(&adapters);
    // sysinfo.networkadapters = adapters;

    unsafe {
        let mut memory = 0u64;
        GetPhysicallyInstalledSystemMemory(&mut memory);
        sysinfo.memory_size = memory;
        dbg!(memory >> 20);
    }

    sysinfo
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    win32console::console::WinConsole::set_output_code(936).expect("Unable to set output code");

    let sysinfo = get_sys_info();

    dbg!(&sysinfo);

    let (read, write) = TcpStream::connect(CONNECTION_STR_CLIENT)
        .await?
        .into_split();

    println!("Connecting to server at {}", CONNECTION_STR_CLIENT);

    let (staffid, mut info) = prompt()?;

    info.sysinfo = sysinfo;
    dbg!(&info.sysinfo);

    let ans = Confirm::new("Do you want to submit now?")
        .with_default(true)
        .prompt()?;

    if ans {
        println!("Submitting");

        dbg!(&info);
        let encinfo = encode(&info);
        dbg!(&encinfo);
        let decinfo = decode(&encinfo);
        dbg!(&decinfo);

        let packet = Packet {
            magic: MAGIC,
            version: CUR_VERSION,
            staffid,
            encinfo,
        };

        let mut transport = LengthDelimitedCodec::builder()
            .length_field_type::<u32>()
            .new_write(write);

        let t = rkyv::to_bytes::<_, 16384>(&packet).unwrap();
        transport
            .send(Bytes::from_iter(t.iter().map(|x| *x)))
            .await?;

        let mut transport = LengthDelimitedCodec::builder()
            .length_field_type::<u32>()
            .new_read(read);

        let response = transport
            .next()
            .await
            .unwrap()
            .expect("Unable to receive response from server");

        if response == Bytes::from("OK") {
            println!("Uploaded successfully");
        } else {
            println!("Server error : {:?}", response);
        }
        // loop {
        //     stream.writable().await?;

        //     match stream.try_write_vectored(&buf) {
        //         Ok(n) => {
        //             println!("Data uploaded to server");

        //             let mut buf = [0u8; MAGIC.len() + VERSION.len() + 5];
        //             stream.readable().await?;
        //             let read = stream.try_read(&mut buf)?;

        //             assert_eq!(&buf[(MAGIC.len() + VERSION.len())..read], "OK".as_bytes());

        //             println!("Submitted successfully");
        //             break;
        //         }
        //         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        //             continue;
        //         }
        //         Err(e) => return Err(e.into()),
        //     }
        // }
    }

    let _ = Text::new("Quit now?").prompt()?;

    Ok(())
}
