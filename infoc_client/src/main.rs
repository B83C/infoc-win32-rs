#![feature(vec_push_within_capacity)]
#![feature(let_chains)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use futures::executor::block_on;
use infoc::*;
use inquire::{length, Confirm, CustomType, Select, Text};
use std::error::Error;
use ::windows::{
    core::*, Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::Ioctl::*,
    Win32::System::IO::*,
};
use core::ffi::c_void;
use std::collections::HashMap;
use futures::StreamExt;
use std::mem::size_of;
use std::result::Result;
use winreg::enums::*;
use winreg::RegKey;
use wmi::*;

#[inline]
fn prompt() -> Result<(String, Vec<Accessories>, Position), Box<dyn Error>> {
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

    block_on(get_printers(&mut accessories));

    loop {
        let ans = Confirm::new("Any accessories to add?")
            .with_default(false)
            .prompt()?;

        if ans {
            if let Some(item) = Select::new("Item", Item::VARIANTS.to_vec())
                .raw_prompt()
                .ok()
                .map(|x| all::<Item>().nth(x.index).expect("Invalid selection"))
            {
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
            }
        } else {
            break;
        }
    }

    Ok((staffid, accessories, pos))
}

async fn get_sys_info() -> Result<SysInfoV1, Box<dyn std::error::Error>> {
    let com = COMLibrary::new().expect("COM Failed");
    let wmi_con = WMIConnection::new(com).expect("WMI Connection failed");

    let filter = HashMap::from([("PhysicalAdapter".into(), FilterValue::Bool(true))]);

    let net = if let Ok(mut res) = wmi_con
        .async_filtered_query::<Win32_NetworkAdapter>(&filter)
        .await
    {
        res.retain(|x| x.PNPDeviceID.as_ref().is_some_and(|x| x.contains("PCI")));
        res
    } else {
        eprintln!("No valid network adapter can be found");
        let mut vec = Vec::with_capacity(8);
        loop {
            let ans = Confirm::new("Add new adapter?")
                .with_default(true)
                .prompt_skippable()
                .is_ok_and(|x| x.is_some_and(|x| x));

            if ans {
                let MacAddress = Text::new("Mac Addr: ")
                    .with_validator(length!(17))
                    .prompt()
                    .ok();

                let AdapterTypeID = Select::new("AdapterType: ", ADAPTERTYPE.into())
                    .raw_prompt()
                    .ok()
                    .map(|x| x.index as u16);

                let Description = Text::new("Desc: ").prompt().ok();
               
                let netconstatus = if Confirm::new("Status(y for online, n for offline)").with_default(true).prompt().is_ok_and(|x| x) { NetConnectionStatus::Connected } else { NetConnectionStatus::Disconnected};

                if vec
                    .push_within_capacity(Win32_NetworkAdapter {
                        MacAddress,
                        AdapterTypeID,
                        Description,
                        PNPDeviceID: None,
                        NetConnectionStatus : netconstatus
                    })
                    .is_err()
                {
                    break;
                }
            } else {
                break;
            }
        }
        vec
    };

    let disks = {
        if let Ok(wmi_con)= WMIConnection::with_namespace_path("Root\\Microsoft\\Windows\\Storage", com) && let Ok(res) = wmi_con.async_query::<MSFT_PhysicalDisk>().await {
            Disk::MSFT(
                res.into_iter()
                    .filter(|x| x.MediaType == DiskType::SSD || x.MediaType == DiskType::HDD)
                    .collect(),
            )
        } else {
        // let disk = {
            (|| -> Disk {
                if let Ok(disks) = wmi_con.query::<Win32_DiskDrive>() {
                    return Disk::W32(
                        disks
                            .into_iter()
                            .filter_map(|mut x| {
                                if x.Capabilities.contains(&7) { //7 - USB
                                    return None;
                                }

                                let handle = unsafe {
                                    CreateFileA(
                                        // PCSTR(&x.DeviceID.as_bytes()[0] as *const u8),
                                        PCSTR(x.DeviceID.replace("\\\\", "\\").as_ptr()),
                                        0,
                                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                                        None,
                                        OPEN_EXISTING,
                                        FILE_FLAGS_AND_ATTRIBUTES(0),
                                        HANDLE(0),
                                    )
                                };

                                if let Ok(handle) = handle {
                                    let squery = STORAGE_PROPERTY_QUERY {
                                        PropertyId: StorageDeviceTrimProperty,
                                        QueryType: PropertyStandardQuery,
                                        AdditionalParameters: [0],
                                    };

                                    let mut desc: DEVICE_TRIM_DESCRIPTOR = Default::default();
                                    unsafe {
                                        if DeviceIoControl(
                                            handle,
                                            IOCTL_STORAGE_QUERY_PROPERTY,
                                            Some(
                                                &squery as *const STORAGE_PROPERTY_QUERY
                                                    as *const c_void,
                                            ),
                                            size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                                            Some(
                                                &mut desc as *mut DEVICE_TRIM_DESCRIPTOR
                                                    as *mut c_void,
                                            ),
                                            size_of::<DEVICE_TRIM_DESCRIPTOR>() as u32,
                                            None,
                                            None,
                                        )
                                        .as_bool()
                                        {
                                            CloseHandle(handle);
                                            x.MediaType = if desc.TrimEnabled.as_bool() {
                                                DiskType::SSD
                                            } else {
                                                DiskType::HDD
                                            };
                                            return Some(x);
                                        }
                                    }
                                }

                                eprintln!(
                                    "Unable to detect disk type for {:?} {:?}",
                                    x.Model, x.DeviceID
                                );

                                let selection = Select::new(
                                    "What's your decision? Or skip (ESC)?",
                                    DiskType::VARIANTS.into(),
                                )
                                .raw_prompt()
                                .map_or(
                                    DiskType::None,
                                    |x| { all::<DiskType>().nth(x.index).expect("Invalid selection")},
                                );
                                match selection {
                                    DiskType::SSD | DiskType::HDD => {
                                        x.MediaType = selection;
                                        Some(x)
                                    }
                                    _ => None,
                                }
                            })
                            .collect(),
                    ); 
                }

                eprintln!("Unable to fetch disks, please enter manually");
                
                let mut vec = Vec::with_capacity(8);

                loop {
                    let ans = Confirm::new("Add new disk?")
                        .with_default(true)
                        .prompt_skippable()
                        .is_ok_and(|x| x.is_some_and(|x| x));

                    if ans {
                        let Model = Text::new("Model: ").prompt().ok();
                        let Manufacturer = Text::new("Manufacturer: ").prompt().ok();
                        let MediaType = Select::new(
                            "What's your decision?",
                            DiskType::VARIANTS.into(),
                        )
                        .raw_prompt()
                        .ok()
                        .map(
                            |x| { all::<DiskType>().nth(x.index).expect("Invalid selection")},
                        ).unwrap_or_default();

                        let Size = CustomType::<u64>::new("Disk Size in bytes : ")
                            .prompt()
                            .unwrap_or(0);

                        vec.push(Win32_DiskDrive {
                            Model,
                            Manufacturer,
                            MediaType,
                            Size,
                            Capabilities: Default::default(),
                            DeviceID: Default::default(),
                        })
                    } else {
                        break;
                    }
                }
                Disk::W32(vec)
            })()
        }
    };

    let cs = if let Ok(res) = wmi_con.async_query::<Win32_ComputerSystem>().await && let Some(entry) = res.into_iter().nth(0) {
        entry
    } else {
        eprintln!("ComputerSystem wmi fetch failed, please input manually: ");
        let DNSHostName = Text::new("DNSHostname: ").prompt().ok();

        let Manufacturer = Text::new("Manufacturer: ").prompt().ok();
        let Model = Text::new("Model: ").prompt().ok();
        let TotalPhysicalMemory = CustomType::<u64>::new("TotalPhysicalMemory: ")
            .prompt()
            .unwrap_or(0);

        Win32_ComputerSystem {
            DNSHostName,
            Manufacturer,
            Model,
            TotalPhysicalMemory,
        }
    };

    let os = if let Ok(res) = wmi_con.async_query::<Win32_OperatingSystem>().await && let Some(entry) = res.into_iter().nth(0) {
        entry
    } else {
        eprintln!("OperatingSystem wmi fetch failed, please input manually: ");
        let Caption = Text::new("Caption: ").prompt().ok();

        let OSArchitecture = Select::new(
            "OSArchitecture: ",
            ["64-bit".into(), "32-bit".into()].into(),
        )
        .prompt()
        .ok();

        Win32_OperatingSystem {
            Caption,
            OSArchitecture,
        }
    };
    let cpu = if let Ok(res) = wmi_con.async_query::<Win32_Processor>().await && let Some(entry) = res.into_iter().nth(0) {
        entry
    } else {
        eprintln!("Unable to fetch cpu details");
        let Name = Text::new("CPU Name: ").prompt().ok();

        Win32_Processor { Name }
    };

    let bios = if let Ok(res) = wmi_con.async_query::<Win32_BIOS>().await && let Some(entry) = res.into_iter().nth(0) {
        entry
    } else {
        eprintln!("Unable to fetch BIOS details");
        let SerialNumber = Text::new("SerialNumber: ").prompt().ok();
        let Manufacturer = Text::new("Manufacturer: ").prompt().ok();
        let Description = Text::new("Description: ").prompt().ok();

        Win32_BIOS { SerialNumber, Manufacturer, Description}
    };

    let office = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("Software\\Microsoft\\Office")
        .expect("Unable to open registry subkey");
    
    let mut msoffice = Vec::new();
    office.enum_keys().for_each(|x| {
        let x = x.unwrap();
        if let Ok(num) = x.parse::<f32>() {
            let num = num as u8;
            dbg!(num);
            // let subkey = office.open_subkey(x);

            // if let Ok(subkey) = subkey {
            //     if let Ok(x) = subkey.get_value::<String, _>("DisplayName") {
            //         offices.push(x);
            //     }
            // }
            msoffice.push(match num {
                11 => "Too old!".into(),
                12 => "2007".into(),
                14 => "2010".into(),
                15 => "2013".into(),
                16 => {
                    match office.open_subkey([x.as_str(), "\\Common\\Licensing\\LicensingNext"].concat()) {
                        Ok(x) if x.enum_values().any(|x| x.is_ok_and(|x| x.0.contains("o365"))) => "O365".into(),
                        Ok(_) => "2019".into(),
                        Err(_) => "2016".into(),
                    }
                }
                _ => {
                    Text::new(format!("Not Sure what office it (num : {}), please tell me : ", num).as_str()).prompt().unwrap_or_default()
                }
            });
        }
    });
    
    loop {
        let ans = Confirm::new("Add missing ms office?")
            .with_default(false)
            .prompt().is_ok_and(|x| x);

        if ans {
            msoffice.push(Text::new("Not Sure what office it, please tell me : ").prompt().unwrap_or_default());
        } else {
            break;
        }
    }

    Ok(SysInfoV1 {
        bios,
        os,
        cs,
        cpu,
        net,
        disks,
        msoffice,
    })
    // dbg!(mounts);

    // let mut drives = Vec::with_capacity(16);
    // let mut disks = HashMap::<(u32, u32), ()>::with_capacity(16);

    // loop {
    //     if mounts == 0 {
    //         break;
    //     };
    //     let tzcnt = mounts.trailing_zeros() + 1;
    //     accumulator += (tzcnt - 1) as u8;
    //     mounts >>= tzcnt;
    //     let drive_name = [b'\\', b'\\', b'.', b'\\', (b'A' + accumulator), b':', 0];
    //     let handle = unsafe {
    //         CreateFileA(
    //             PCSTR(&drive_name as *const u8),
    //             0,
    //             FILE_SHARE_READ | FILE_SHARE_WRITE,
    //             None,
    //             OPEN_EXISTING,
    //             FILE_FLAGS_AND_ATTRIBUTES(0),
    //             HANDLE(0),
    //         )
    //         .expect("Handle failure")
    //     };

    //     unsafe {
    //         let mut dn: STORAGE_DEVICE_NUMBER = Default::default();
    //         DeviceIoControl(
    //             handle,
    //             IOCTL_STORAGE_GET_DEVICE_NUMBER,
    //             None,
    //             0,
    //             Some(&mut dn as *mut STORAGE_DEVICE_NUMBER as *mut c_void),
    //             size_of::<STORAGE_DEVICE_NUMBER>() as u32,
    //             None,
    //             None,
    //         )
    //         .expect("IOCTL_STORAGE_GET_DEVICE_NUMBER has failed");

    //         let ent = (dn.DeviceType, dn.DeviceNumber);

    //         // dbg!(&ent);

    //         // if ent.0 != DRIVE_FIXED && ent.0 != DRIVE_REMOVABLE {
    //         //     continue;
    //         // }

    //         match disks.entry(ent) {
    //             Entry::Occupied(_) => {
    //                 continue;
    //             }
    //             Entry::Vacant(v) => {
    //                 v.insert(());
    //             }
    //         }
    //     }

    //     let mut squery = STORAGE_PROPERTY_QUERY {
    //         PropertyId: StorageDeviceSeekPenaltyProperty,
    //         QueryType: PropertyStandardQuery,
    //         AdditionalParameters: [0],
    //     };

    //     let mut desc: DEVICE_SEEK_PENALTY_DESCRIPTOR = Default::default();
    //     let mut dgeo: DISK_GEOMETRY_EX = Default::default();
    //     let mut model: STORAGE_ADAPTER_SERIAL_NUMBER = Default::default();

    //     let mut size: u32 = 0;
    //     let mut model_name = None;

    //     unsafe {
    //         let res = DeviceIoControl(
    //             handle,
    //             IOCTL_STORAGE_QUERY_PROPERTY,
    //             Some(&squery as *const STORAGE_PROPERTY_QUERY as *const c_void),
    //             size_of::<STORAGE_PROPERTY_QUERY>() as u32,
    //             Some(&mut desc as *mut DEVICE_SEEK_PENALTY_DESCRIPTOR as *mut c_void),
    //             size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32,
    //             Some(&mut size),
    //             None,
    //         )
    //         .as_bool();
    //         if res {
    //             eprintln!("Unable to query storage property");
    //         }
    //         // dbg!(&handle);
    //         let res = DeviceIoControl(
    //             handle,
    //             IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
    //             None,
    //             0,
    //             Some(&mut dgeo as *mut DISK_GEOMETRY_EX as *mut c_void),
    //             size_of::<DISK_GEOMETRY_EX>() as u32,
    //             Some(&mut size),
    //             None,
    //         )
    //         .as_bool();
    //         if res {
    //             eprintln!("Unable to query storage geometry");
    //         }
    //         squery.PropertyId = StorageAdapterSerialNumberProperty;
    //         let res = DeviceIoControl(
    //             handle,
    //             IOCTL_STORAGE_QUERY_PROPERTY,
    //             Some(&squery as *const STORAGE_PROPERTY_QUERY as *const c_void),
    //             size_of::<STORAGE_PROPERTY_QUERY>() as u32,
    //             Some(&mut model as *mut STORAGE_ADAPTER_SERIAL_NUMBER as *mut c_void),
    //             size_of::<STORAGE_ADAPTER_SERIAL_NUMBER>() as u32,
    //             Some(&mut size),
    //             None,
    //         )
    //         .as_bool();
    //         if res {
    //             let range_end = model
    //                 .SerialNumber
    //                 .iter()
    //                 .position(|&c| c == '\0' as u16 || c == ' ' as u16)
    //                 .unwrap_or(model.SerialNumber.len());
    //             model_name = Some(String::from_utf16_lossy(&model.SerialNumber[..range_end]));
    //         }
    //         CloseHandle(handle);
    //     }

    //     let disktype = if !desc.IncursSeekPenalty.as_bool() {
    //         DiskType::SSD
    //     } else {
    //         DiskType::HDD
    //     };

    //     sysinfo.disks.push(Disk {
    //         disktype,
    //         disksize: dgeo.DiskSize as u64,
    //         model: model_name,
    //     });

    //     drives.push(desc);
    // }

    // match table_load_from_device() {
    //     Ok(data) => {
    //         let processor = data
    //             .first::<SMBiosProcessorInformation>()
    //             .expect("Unable to retrieve processor info");
    //         sysinfo.cpu = processor.processor_version().to_string();
    //         // dbg!(&sysinfo.cpu);
    //         let system = data
    //             .first::<SMBiosSystemInformation>()
    //             .expect("Unable to retrieve System info");
    //         sysinfo.serial_number = system.serial_number().to_string();
    //         // dbg!(&sysinfo.serial_number);
    //         sysinfo.product_name = system.product_name().to_string();
    //         // dbg!(&sysinfo.product_name);
    //         sysinfo.manufacturer = system.manufacturer().to_string();
    //         // dbg!(&sysinfo.manufacturer);
    //         sysinfo.sku_number = system.sku_number().to_string();
    //         // dbg!(&sysinfo.sku_number);
    //         sysinfo.version = system.version().to_string();
    //         // dbg!(&sysinfo.version);
    //         sysinfo.uuid = if let Some(SystemUuidData::Uuid(x)) = system.uuid() {
    //             Some(x.raw)
    //         } else {
    //             None
    //         };
    //         // dbg!(&sysinfo.uuid);
    //     }
    //     Err(_) => {}
    // }

    // sysinfo.os = if let Some(os) = os_info::get().edition() {
    //     os.to_string()
    // } else {
    //     Text::new("OS version can't be detected, please enter manually : ")
    //         .prompt()
    //         .expect("Unable to get input")
    // };

    // let office = RegKey::predef(HKEY_LOCAL_MACHINE)
    //     .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    //     .expect("Unable to open registry subkey");

    // let mut offices: Vec<String> = Vec::new();
    // office.enum_keys().map(|x| x.unwrap()).for_each(|x| {
    //     if x.starts_with("Office") {
    //         let subkey = office.open_subkey(x);

    //         if let Ok(subkey) = subkey {
    //             if let Ok(x) = subkey.get_value::<String, _>("DisplayName") {
    //                 offices.push(x);
    //             }
    //         }
    //     }
    // });

    // sysinfo.microsoft_offices = if offices.len() != 0 {
    //     offices
    // } else {
    //     vec![Text::new("Offices not detected, enter manually")
    //         .prompt()
    //         .expect("Unable to get input")]
    // };

    // let interfaces = get_interfaces()
    //     .into_iter()
    //     .filter_map(|x| match x.if_type {
    //         interface::InterfaceType::Wireless80211 | interface::InterfaceType::Ethernet => {
    //             if let Some(mac_addr) = x.mac_addr {
    //                 Some(NetworkAdapter {
    //                     physical_address: mac_addr.octets(),
    //                     addr: x.ipv4.iter().map(|x| x.addr).collect(),
    //                 })
    //             } else {
    //                 None
    //             }
    //         }
    //         _ => None,
    //     })
    //     .collect();
    // // dbg!(&interfaces);
    // sysinfo.networkadapters = interfaces;
    // // let adapters = get_adapters()
    // //     .expect("Unable to get network adapters")
    // //     .iter()
    // //     .filter_map(|x| match x.if_type() {
    // //         IfType::EthernetCsmacd | IfType::Ieee80211 => match x.oper_status() {
    // //             OperStatus::IfOperStatusUp
    // //             | OperStatus::IfOperStatusUnknown
    // //             | OperStatus::IfOperStatusDown => Some(NetworkAdapter {
    // //                 physical_address: x.physical_address().map(|x| x.to_owned()),
    // //                 addr: x.ip_addresses().to_vec(),
    // //             }),
    // //             _ => None,
    // //         },
    // //         _ => None,
    // //     })
    // //     .collect();
    // // dbg!(&adapters);
    // // sysinfo.networkadapters = adapters;

    // unsafe {
    //     let mut memory = 0u64;
    //     GetPhysicallyInstalledSystemMemory(&mut memory);
    //     sysinfo.memory_size = memory;
    //     dbg!(memory >> 20);
    // }
}

async fn get_printers(accessories: &mut Vec<Accessories>) {
    let com = COMLibrary::new().expect("COM Failed");
    let wmi_con = WMIConnection::new(com).expect("WMI Connection failed");

    let ans = Confirm::new("Any printers to add?")
        .with_default(false)
        .prompt().is_ok_and(|x| x);

    if ans {
        if let Ok(res) = wmi_con.async_query::<Win32_Printer>().await {
            res.into_iter().for_each(|x| {
                if x.Attributes & 64 != 0 && Confirm::new(format!("Add {}?", x.Name).as_str()).with_default(false).prompt_skippable().is_ok_and(|x| x.is_some_and(|x| x)){
                    accessories.push(Accessories { item: Item::Printer, details: Details { count: 1, remarks: Some(x.Name)  }  });
                }});
        }    }
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match win32console::console::WinConsole::set_output_code(936) {
        Err(_) => eprintln!("Warning: Unable to set output code of console, maybe unusable"),
        _ => {}
    }

    let (staffid, accessories, pos) = prompt()?;

    let sysinfo = get_sys_info().await?;

    let infoc = Infoc {accessories, pos, sysinfo};

    println!("Info is as follows:\n {:#?}", &infoc);

    let ans = Confirm::new("Do you wish to submit now?")
        .with_default(true)
        .prompt()?;

    if ans {
        let encinfo = rkyv::to_bytes::<_, 63356>(&infoc).expect("Unable to serialize info").to_vec();
        
        let packet = Packet {
            magic: MAGIC,
            version: CUR_VERSION,
            staffid : staffid.to_owned(),
            encinfo,
        };
        for cstr in CONNECTION_STR_CLIENT {
            if let Ok(stream) = TcpStream::connect(cstr).await {
                println!("Connected to server at {}", cstr);
                // let mut writer = RkyvWriter::<_, VarintLength>::new(stream.clone());

                // writer.send(&packet).await.expect("Unable to send packet to server");

                // let mut buffer = AlignedVec::new();
                
                // let data : &Archived<Packet> = archive_stream::<_, Packet, VarintLength>(&mut stream, &mut buffer).await.unwrap();
                // dbg!(data);

                let encpacket = rkyv::to_bytes::<_, 65536>(&packet).expect("Unable to serialize packet");
                let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
                
                println!("Submitting");

                stream.send(Bytes::from(encpacket.into_vec())).await?;

                let response = stream.next().await.unwrap().expect("Unable to receive response from server");
                if response == Bytes::from("OK") {
                    println!("Uploaded successfully");
                } else {
                    println!("Server error : {:?}", response);
                }
                
                let _ = Text::new("Quit now?").prompt()?;
                return Ok(());
            }
        }
        if Confirm::new("Looks like we haven't been able to connect to server, save offline?").with_default(true).prompt().is_ok_and(|x| x) {
            
            let db = block_on(HashStore::open(DB_NAME_TEMP)).expect("Unable to open db");

            println!("Saving data to disk");

            block_on(db.set(packet.staffid.as_bytes(), packet.encinfo.into())).expect("Unable to set data");
            block_on(db.flush()).expect("Unable to flush data");
        }
    }


    Ok(())
}
