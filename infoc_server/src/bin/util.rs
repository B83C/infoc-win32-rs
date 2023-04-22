#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use clap::Parser;
use infoc::*;
use rust_xlsxwriter::Workbook;
use std::result::Result;
use std::{collections::HashMap, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Mode {
    /// Command to operate
    command: String,

    /// Output file w/wo file extension
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<PathBuf>,
}

#[derive(Debug, serde::Deserialize, Eq, PartialEq)]
struct Mapping {
    S_CODE: String,
    S_CNAME: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = Mode::parse();
    let db = HashStore::open(DB_NAME).await.expect("Unable to open db");
    match mode.command.as_str() {
        "export" => {
            let mut output = mode.output.unwrap_or_else(|| PathBuf::from("chkl_data"));
            output.set_extension("xlsx");
            let output = output.to_str().expect("Unable to set path to string");

            let mut table = csv::Reader::from_path("mapping.csv")
                .expect("Unable to find employee mapping table");
            let tsmapping = table.deserialize();

            let mut map: HashMap<Vec<u8>, String> = HashMap::new();
            for i in tsmapping {
                let m: Mapping = i.expect("Unable to parse tsmapping");
                map.insert(m.S_CODE.to_ascii_uppercase().into_bytes(), m.S_CNAME);
            }

            let mut workbook = Workbook::new();

            // dbg!(db.keys())?;

            let worksheet = workbook.add_worksheet().set_name("2023年")?;

            let cols = [
                "No",
                "厂家",
                "MODEL",
                "S/No",
                "CPU",
                "RAM",
                "HDD/SSD",
                "系统",
                "Microsoft Office",
                "部门",
                "使用者",
                "Printer",
                "Desktop 年份",
            ];

            cols.iter().enumerate().for_each(|(i, x)| {
                worksheet
                    .write(0, i as u16, *x)
                    .expect("Error occured whilst writing column names");
            });

            db.keys_from_index()
                .await
                .iter()
                .enumerate()
                .for_each(|(i, x)| {
                    let i = i + 1;
                    use futures::executor::block_on;
                    let ent = block_on(db.get(x))
                        .expect("Unable to get value")
                        .expect("Unable to unwrap value");
                    let enc = decode(&ent);
                    worksheet
                        .write(i as u32, 0, i as u32)
                        .expect("Unable to write to No");
                    worksheet
                        .write(
                            i as u32,
                            1,
                            enc.sysinfo
                                .cs
                                .Manufacturer
                                .as_ref()
                                .map(|x| x.as_ref())
                                .unwrap_or(""),
                        )
                        .expect("Unable to write to Manufacturer");
                    worksheet
                        .write(
                            i as u32,
                            2,
                            enc.sysinfo
                                .cs
                                .Model
                                .as_ref()
                                .map(|x| x.as_ref())
                                .unwrap_or(""),
                        )
                        .expect("Unable to write to Model");
                    worksheet
                        .write(
                            i as u32,
                            3,
                            enc.sysinfo
                                .bios
                                .SerialNumber
                                .as_ref()
                                .map(|x| x.as_ref())
                                .unwrap_or(""),
                        )
                        .expect("Unable to write to S/No");
                    worksheet
                        .write(
                            i as u32,
                            4,
                            enc.sysinfo
                                .cpu
                                .Name
                                .as_ref()
                                .map(|x| x.as_ref())
                                .unwrap_or(""),
                        )
                        .expect("Unable to write to CPU");
                    worksheet
                        .write(
                            i as u32,
                            5,
                            format!(
                                "{}GB",
                                enc.sysinfo.cs.TotalPhysicalMemory.div_euclid(1000000)
                            )
                            .as_str(),
                        )
                        .expect("Unable to write to CPU");
                    worksheet
                        .write(
                            i as u32,
                            6,
                            match enc.sysinfo.disks {
                                ArchivedDisk::MSFT(ref x) => x
                                    .as_ref()
                                    .iter()
                                    .map(|x| {
                                        format!(
                                            "{}GB {}",
                                            x.Size.div_euclid(1000000000),
                                            x.MediaType.to_string(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                ArchivedDisk::W32(ref x) => x
                                    .as_ref()
                                    .iter()
                                    .map(|x| {
                                        format!(
                                            "{}GB {}",
                                            x.Size.div_euclid(1000000000),
                                            x.MediaType.to_string(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            }
                            .as_str(),
                        )
                        .expect("Unable to write to disks");
                    worksheet
                        .write(
                            i as u32,
                            7,
                            enc.sysinfo
                                .os
                                .Caption
                                .as_ref()
                                .map(|x| x.as_ref())
                                .unwrap_or(""),
                        )
                        .expect("Unable to write to OS Name");
                    worksheet
                        .write(
                            i as u32,
                            8,
                            enc.sysinfo
                                .msoffice
                                .as_ref()
                                .iter()
                                .map(|x| ["Microsoft Office ", x].concat())
                                .collect::<Vec<String>>()
                                .join(", ")
                                .as_str(),
                        )
                        .expect("Unable to write to OS");
                    worksheet
                        .write(i as u32, 9, DEPARTMENT[enc.pos.department as usize])
                        .expect("Unable to write to Department");
                    worksheet
                        .write(
                            i as u32,
                            10,
                            map.get(x.as_slice())
                                .map(|x| x.as_str())
                                .unwrap_or("Unknown"),
                        )
                        .expect("Unable to write to Name");
                    worksheet
                        .write(
                            i as u32,
                            11,
                            enc.accessories
                                .iter()
                                .filter_map(|x| {
                                    if x.item == ArchivedItem::Printer {
                                        if let Some(x) = x.details.remarks.as_ref() {
                                            Some(x.as_ref())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                                .as_str(),
                        )
                        .expect("Unable to write to Printer");
                });

            worksheet.autofit();
            worksheet.set_freeze_panes(1, 0)?;
            workbook.save(output)?;
        } //
        _ => {}
    }

    //  Ok(// ())
    Ok(())
}
