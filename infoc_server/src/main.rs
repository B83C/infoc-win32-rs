use infoc::*;
use std::{process, result::Result};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(CONNECTION_STR_SERVER).await?;

    println!("Listening on {}", CONNECTION_STR_SERVER);

    use std::sync::Arc;

    let db = HashStore::open(DB_NAME)
        .await
        .expect("Unable to open database");

    let db = Arc::new(db);
    // let mut db = KvStore::open(DB_NAME).expect("Error opening db");

    // let db = microkv_open()?.set_auto_commit(true);

    {
        let db = db.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            println!("Flushing to disk");
            db.flush().await.unwrap();
            process::exit(1);
        });
    }
    loop {
        let (socket, addr) = listener.accept().await?;

        println!("Recieved connections from {:?}", addr);

        let db = db.clone();

        tokio::spawn(async move {
            let mut transport = Framed::new(socket, LengthDelimitedCodec::new());

            let msg = transport
                .next()
                .await
                .unwrap()
                .expect("Unable to read value from client");

            // let packet = rkyv::check_archived_root::<Packet>(&msg[..]).ok();
            let packet = rkyv::from_bytes::<Packet>(&msg[..]).ok();

            if let Some(header) = packet {
                if header.magic == MAGIC {
                    match header.version {
                        VERSION::V1 => {
                            // let header =
                            //     rkyv::from_bytes::<Infoc>(header.encinfo.as_slice())
                            //         .expect("Unable to deserialize encinfo");
                            // dbg!(&header);
                            let cmds = vec![
                                CommandData::remove(header.staffid.clone().into_bytes()),
                                CommandData::set(
                                    header.staffid.into_bytes(),
                                    header.encinfo.into(),
                                ),
                            ];
                            db.batch(cmds).await.expect("Unable to put to database");
                            db.flush().await.expect("Unable to flush to database");
                            // db.commit().expect("Unable to commit to database");

                            transport.send(Bytes::from("OK")).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
