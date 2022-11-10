use std::net::UdpSocket;
use anyhow::Result;

fn main() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    println!("Hello, world!");

    loop {
        let mut buf = [0; 10];
        let (amt, src) = socket.recv_from(&mut buf)?;

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        let buf = &mut buf[..amt];
        buf.reverse();
        socket.send_to(buf, &src)?;
    }

    Ok(())
}
