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
use ipconfig::*;
use smbioslib::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::size_of;
use std::result::Result;

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

fn get_sys_info() -> SysInfo {
    let mut sysinfo: SysInfo = Default::default();
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
            let res = DeviceIoControl(
                handle,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some(&mut dn as *mut STORAGE_DEVICE_NUMBER as *mut c_void),
                size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                None,
                None,
            )
            .as_bool();

            assert!(res);

            let ent = (dn.DeviceType, dn.DeviceNumber);

            if ent.0 != DRIVE_FIXED && ent.0 != DRIVE_REMOVABLE {
                continue;
            }

            match disks.entry(ent) {
                Entry::Occupied(_) => {
                    CloseHandle(handle);
                    continue;
                }
                Entry::Vacant(v) => {
                    v.insert(());
                }
            }
        }

        let squery = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceSeekPenaltyProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };

        let mut desc: DEVICE_SEEK_PENALTY_DESCRIPTOR = Default::default();
        let mut dgeo: DISK_GEOMETRY_EX = Default::default();

        let mut size: u32 = 0;

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
            assert!(res);
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
            assert!(res);
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
            model: String::new(),
        });

        drives.push(desc);
    }

    match table_load_from_device() {
        Ok(data) => {
            let processor = data
                .first::<SMBiosProcessorInformation>()
                .expect("Unable to retrieve processor info");
            sysinfo.cpu = processor.processor_version().to_string();
            dbg!(&sysinfo.cpu);
            let system = data
                .first::<SMBiosSystemInformation>()
                .expect("Unable to retrieve System info");
            sysinfo.serial_number = system.serial_number().to_string();
            dbg!(&sysinfo.serial_number);
            sysinfo.product_name = system.product_name().to_string();
            dbg!(&sysinfo.product_name);
            sysinfo.manufacturer = system.manufacturer().to_string();
            dbg!(&sysinfo.manufacturer);
            sysinfo.sku_number = system.sku_number().to_string();
            dbg!(&sysinfo.sku_number);
            sysinfo.version = system.version().to_string();
            dbg!(&sysinfo.version);
            sysinfo.uuid = if let Some(SystemUuidData::Uuid(x)) = system.uuid() {
                Some(x.raw)
            } else {
                None
            };
            dbg!(&sysinfo.uuid);
        }
        Err(e) => {}
    }

    sysinfo.os = os_info::get().edition().unwrap().to_string();

    // let addr = MacAddressIterator::new().expect("Unable to fetch MAC Addresses");

    // if let Ok(adapters) = get_adapters() {
    //     let valid: Vec<Option<_>> = adapters
    //         .iter()
    //         .filter_map(|x| {
    //             if x.prefixes().iter().any(|(y, _)| {
    //                 if let std::net::IpAddr::V4(w) = y {
    //                     let octets = w.octets();
    //                     octets[0] == 10
    //                         && [10, 15].iter().any(|x| *x == octets[1])
    //                         && x.oper_status() == OperStatus::IfOperStatusUp
    //                 } else {
    //                     false
    //                 }
    //             }) {
    //                 Some(x.physical_address())
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect();
    //     dbg!(valid);
    // }

    unsafe {
        let mut memory = 0u64;
        GetPhysicallyInstalledSystemMemory(&mut memory);
        sysinfo.memory_size = memory;
        dbg!(memory >> 20);

        // let size = GetSystemFirmwareTable(
        //     RSMB,
        //     FIRMWARE_TABLE_ID(0),
        //     None,
        //     0,
        // );
        // dbg!(size);
        // // let mut table: Vec<u8> = Vec::with_capacity(size as usize);
        // let mut table: Vec<u8> = vec![0u8; size as usize];
        // let size = GetSystemFirmwareTable(
        //     RSMB,
        //     FIRMWARE_TABLE_ID(0),
        //     Some(table.as_mut_ptr() as *mut c_void),
        //     size,
        // );
        // let (_, header, _) = table.align_to::<RawSMBIOSDataHeader>();
        // let smbios = &table[size_of::<RawSMBIOSDataHeader>()..];
        // dbg!(&header[0]);
        // dbg!(smbios);
        // dbg!(&table);
    }

    sysinfo
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (read, write) = TcpStream::connect(CONNECTION_STR).await?.into_split();

    let (staffid, info) = prompt()?;

    let ans = Confirm::new("Do you want to submit now?")
        .with_default(true)
        .prompt()?;

    if ans {
        println!("Submitting");
        let enc = encode(&info).to_vec();

        //MANUFACTURER, MODEL, SERIAL NUMBER, CPU, RAM, DISK, SYSTEM, MSOFFICE,
        //IP, HOSTNAME, MAC
        let packet = Packet {
            magic: MAGIC,
            version: VERSION,
            staffid,
        };

        let mut transport = LengthDelimitedCodec::builder()
            .length_field_type::<u32>()
            .new_write(write);

        let t = rkyv::to_bytes::<_, 64>(&packet).unwrap().to_vec();
        transport.send(t.into()).await?;

        // assert_eq!(transport.next().await.unwrap().unwrap(), Bytes::from("OK"));

        transport.send(enc.into()).await?;

        let mut transport = LengthDelimitedCodec::builder()
            .length_field_type::<u32>()
            .new_read(read);

        assert_eq!(transport.next().await.unwrap().unwrap(), Bytes::from("OK"));

        println!("Uploaded successfully");
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

    Ok(())
}
