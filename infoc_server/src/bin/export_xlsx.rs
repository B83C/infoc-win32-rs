use clap::Parser;
use infoc::*;
use infoc_server::*;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Image, Workbook, XlsxError};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Mode {
    /// Command to operate
    command: String,

    /// Output file w/wo file extension
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = Mode::parse();
    let db = microkv_open()?;

    match mode.command.as_str() {
        "export" => {
            let mut output = mode.output.unwrap_or_else(|| PathBuf::from("chkl_data"));
            output.set_extension("xlsx");
            let output = output.to_str().expect("Unable to set path to string");

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
            let mapping : Vec<(&str, &[u8])> = keys.iter().map(|x| (x.as_str(), db.get_unwrap::<&[u8]>(x).unwrap())).collect();

            dbg!(mapping);

            worksheet.autofit();
            worksheet.set_freeze_panes(1, 0)?;
            workbook.save(output)?;
        }
        _ => {}
    }

    Ok(())
}
