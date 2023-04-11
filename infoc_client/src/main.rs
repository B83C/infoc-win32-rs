use infoc::rkyv;
use infoc::*;
use inquire::{Confirm, CustomType, Select, Text};
use std::error::Error;
use tokio::net::TcpStream;
use wmi::WMIDateTime;
use wmi::{COMLibrary, WMIConnection};
// use sysinfo::{Component, Cpu, Disk, MacAddr, NetworkExt, NetworksExt, System, SystemExt, User};

#[inline]
fn prompt() -> Result<(String, Info), Box<dyn Error>> {
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
        Info {
            accessories,
            pos,
            sysinfo: Default::default(),
        },
    ))
}

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

    let results: Vec<NetworkAdapterConfiguration> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }

    let results: Vec<DiskDrive> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }

    let results: Vec<Processor> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os);
    }

    let mut stream = TcpStream::connect(CONNECTION_STR).await?;

    let dum = prompt()?;

    let ans = Confirm::new("Do you want to submit now?")
        .with_default(true)
        .prompt()?;

    if ans {
        println!("Submitting");
        let enc = encode(&dum.1);

        //MANUFACTURER, MODEL, SERIAL NUMBER, CPU, RAM, DISK, SYSTEM, MSOFFICE,
        //IP, HOSTNAME, MAC

        use std::io;
        let buf = [
            io::IoSlice::new(MAGIC),
            io::IoSlice::new(&VERSION),
            io::IoSlice::new(&dum.0.as_bytes()),
            io::IoSlice::new(enc.as_slice()),
        ];

        loop {
            stream.writable().await?;

            match stream.try_write_vectored(&buf) {
                Ok(n) => {
                    println!("Data uploaded to server");

                    let mut buf = [0u8; MAGIC.len() + VERSION.len() + 5];
                    stream.readable().await?;
                    let read = stream.try_read(&mut buf)?;

                    assert_eq!(&buf[(MAGIC.len() + VERSION.len())..read], "OK".as_bytes());

                    println!("Submitted successfully");
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    Ok(())
}
