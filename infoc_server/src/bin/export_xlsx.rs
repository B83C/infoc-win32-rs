use clap::Parser;
use infoc::*;
use infoc_server::*;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Image, Workbook, XlsxError};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = Mode::parse();
    let db = microkv_open()?;

    match mode.command.as_str() {
        "export" => {
            let mut output = mode.output.unwrap_or_else(|| PathBuf::from("chkl_data"));
            output.set_extension("xlsx");
            let output = output.to_str().expect("Unable to set path to string");

            let mut table = csv::Reader::from_path("mapping.csv")
                .expect("Unable to find employee mapping table");
            let tsmapping = table.deserialize();

            let mut map: HashMap<String, String> = HashMap::new();
            for i in tsmapping {
                let m: Mapping = i.expect("Unable to parse tsmapping");
                map.insert(m.S_CODE.to_ascii_uppercase(), m.S_CNAME);
            }

            let mut workbook = Workbook::new();

            dbg!(db.keys())?;

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

            let keys = db.keys()?;
            keys.iter().enumerate().for_each(|(i, x)| {
                let i = i + 1;
                let enc = db
                    .get_unwrap::<Vec<u8>>(x)
                    .expect("Unable to get entry by key");
                dbg!(&enc);
                let enc = decode(enc.as_slice());
                worksheet
                    .write(i as u32, 0, i as u32)
                    .expect("Unable to write to No");
                worksheet
                    .write(i as u32, 1, enc.sysinfo.manufacturer.as_str())
                    .expect("Unable to write to Manufacturer");
                worksheet
                    .write(i as u32, 2, enc.sysinfo.manufacturer.as_str())
                    .expect("Unable to write to Manufacturer");
                worksheet
                    .write(i as u32, 3, enc.sysinfo.serial_number.as_str())
                    .expect("Unable to write to S/No");
                worksheet
                    .write(i as u32, 4, enc.sysinfo.cpu.as_str())
                    .expect("Unable to write to CPU");
                worksheet
                    .write(
                        i as u32,
                        5,
                        format!("{}GB", enc.sysinfo.memory_size.div_euclid(1000000)).as_str(),
                    )
                    .expect("Unable to write to CPU");
                worksheet
                    .write(
                        i as u32,
                        6,
                        enc.sysinfo
                            .disks
                            .iter()
                            .map(|x| {
                                format!(
                                    "{}GB {}",
                                    x.disksize.div_euclid(1000000),
                                    x.disktype.to_string(),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                            .as_str(),
                    )
                    .expect("Unable to write to disks");
                worksheet
                    .write(i as u32, 7, enc.sysinfo.os.as_str())
                    .expect("Unable to write to OS");
                worksheet
                    .write(
                        i as u32,
                        8,
                        enc.sysinfo.microsoft_offices.join(", ").as_str(),
                    )
                    .expect("Unable to write to OS");
                worksheet
                    .write(i as u32, 9, DEPARTMENT[enc.pos.department as usize])
                    .expect("Unable to write to Department");
                worksheet
                    .write(
                        i as u32,
                        10,
                        map.get(x).map(|x| x.as_str()).unwrap_or("Unknown"),
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
                dbg!(enc);
            });

            worksheet.autofit();
            worksheet.set_freeze_panes(1, 0)?;
            workbook.save(output)?;
        }
        _ => {}
    }

    Ok(())
}
