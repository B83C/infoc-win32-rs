use infoc::*;
use infoc_server::*;
use std::result::Result;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(CONNECTION_STR_SERVER).await?;

    println!("Listening on {}", CONNECTION_STR_SERVER);

    use std::sync::Arc;

    let db = microkv_open()?.set_auto_commit(true);

    let db = Arc::new(db);

    loop {
        let (socket, addr) = listener.accept().await?;

        println!("Recieved connections from {:?}", addr);

        let db = db.clone();

        tokio::spawn(async move {
            let (read, write) = socket.into_split();

            let mut transport = LengthDelimitedCodec::builder()
                .length_field_type::<u32>()
                .new_read(read);

            let msg = transport
                .next()
                .await
                .unwrap()
                .expect("Unable to read value from client");

            let packet = rkyv::check_archived_root::<Packet>(&msg[..]).ok();

            if let Some(header) = packet {
                if header.magic == MAGIC {
                    match header.version {
                        ArchivedVERSION::V1 => {
                            db.put(header.staffid.as_ref(), &header.encinfo.as_ref())
                                .expect("Unable to put to database");
                            db.commit().expect("Unable to commit to database");
                            let mut transport = LengthDelimitedCodec::builder()
                                .length_field_type::<u32>()
                                .new_write(write);

                            transport.send(Bytes::from("OK")).await.unwrap();
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
