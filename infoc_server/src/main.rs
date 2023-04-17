use infoc::*;
use infoc_server::*;
use std::result::Result;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(CONNECTION_STR).await?;
    dbg!(CONNECTION_STR);
    use std::sync::Arc;

    let db = microkv_open()?.set_auto_commit(true);

    let db = Arc::new(db);

    loop {
        let (socket, _) = listener.accept().await?;

        let db = db.clone();

        tokio::spawn(async move {
            let (read, write) = socket.into_split();

            let mut transport = LengthDelimitedCodec::builder()
                .length_field_type::<u32>()
                .new_read(read);

            let msg = transport.next().await.unwrap().unwrap();

            let header = unsafe { rkyv::archived_root::<Packet>(&msg[..]) };

            dbg!(header);

            let msg: Bytes = transport.next().await.unwrap().unwrap().into();

            dbg!(&msg);

            db.put(header.staffid.as_ref(), &msg.as_ref()).unwrap();

            dbg!(db.keys().unwrap());

            db.commit();

            let mut transport = LengthDelimitedCodec::builder()
                .length_field_type::<u32>()
                .new_write(write);

            transport.send(Bytes::from("OK")).await.unwrap();
        });
    }
}
