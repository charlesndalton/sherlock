use std::net::UdpSocket;
use anyhow::Result;
use rand::random;

const GOOGLE_PUBLIC_DNS: &str = "8.8.8.8";
const DNS_UDP_PORT: u16 = 53;

enum Type {
    A = 1,
    NS = 2,
    CNAME = 5,
}

fn main() -> Result<()> {
    let socket = UdpSocket::bind("192.168.0.51:34258")?;
    socket.connect((GOOGLE_PUBLIC_DNS, DNS_UDP_PORT))?;

    let labels = vec!["amazon", "com"];

    let id: u16 = random();
    let flags: u16 = 0x0100;
    let question_count: u16 = 1;
    let answer_count: u16 = 0;
    let ns_count: u16 = 0;
    let ar_count: u16 = 0;

    let mut header = Vec::<u8>::with_capacity(12);
    for (i, field) in [id, flags, question_count, answer_count, ns_count, ar_count].iter().enumerate() {
        if i == 1 { // flags is a special case where we're writing raw hex
            header.extend_from_slice(&field.to_ne_bytes());
        } else {
            header.extend_from_slice(&field.to_be_bytes());
        }
    }

    let mut question = Vec::<u8>::new();
    let mut q_name = Vec::<u8>::new();
    for label in labels {
        assert!(label.is_ascii());
        assert!(label.len() <= 255);

        q_name.push(label.len() as u8);
        q_name.extend_from_slice(label.as_bytes());
    }
    q_name.push(0);

    let q_type = Type::A as u16;
    let q_class: u16 = 1; // the internet; other classes (e.g., CSNET) are obsolete
    question.append(&mut q_name);
    question.extend_from_slice(&q_type.to_be_bytes());
    question.extend_from_slice(&q_class.to_be_bytes());

    let mut message = Vec::<u8>::new();
    message.append(&mut header);
    message.append(&mut question);
    
    socket.send(&message)?;

    Ok(())
}
