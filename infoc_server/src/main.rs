use infoc::*;
use redb::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("info");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(CONNECTION_STR).await?;

    let db = Database::create("info.redb")?;
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 8196 * 2];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                let buf = [MAGIC, &VERSION, "OK".as_bytes()].concat();

                if let Err(e) = socket.try_write(&buf) {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }

    Ok(())
}
