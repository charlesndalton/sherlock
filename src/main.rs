use std::net::UdpSocket;
use std::net::Ipv4Addr;
use anyhow::Result;
use rand::random;

const GOOGLE_PUBLIC_DNS: &str = "8.8.8.8";
const DNS_UDP_PORT: u16 = 53;

#[derive(Debug)]
enum Type {
    A = 1,
    NS = 2,
    CNAME = 5,
}

fn main() -> Result<()> {
    let socket = UdpSocket::bind("192.168.0.51:34258")?;
    socket.connect(("192.168.0.1", DNS_UDP_PORT))?;

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

    let mut buf = [0; 1000];
    match socket.recv(&mut buf) {
        Ok(received) => {
            println!("received {received} bytes {:?}", &buf[..received]);
            let res_header = &buf[0..12];
            let res_id = u16::from_be_bytes(res_header[0..2].try_into()?);
            assert!(res_id == id);

            let res_question_count = u16::from_be_bytes(res_header[4..6].try_into()?);
            let res_answer_count = u16::from_be_bytes(res_header[6..8].try_into()?);

            println!("question count: {}, answer count: {}", res_question_count, res_answer_count);

            let mut buf_p = 12;
            for _ in [0..res_question_count] {
                loop {
                    let tag_len = buf[buf_p];
                    buf_p += 1;

                    if tag_len == 0 {
                        break;
                    }

                    buf_p += tag_len as usize;
                }
                buf_p += 4;
            }

            for _ in [0..res_answer_count] {
                loop {
                    buf_p += 2;

                    let res_type = match u16::from_be_bytes(buf[buf_p..buf_p+2].try_into()?) {
                        1 => Type::A,
                        2 => Type::NS,
                        5 => Type::CNAME,
                        _ => unimplemented!(),
                    };

                    buf_p += 2;

                    // lol using a struct for the type but using a string for the class...
                    // this really is garbage code :shrug:
                    let res_class = match u16::from_be_bytes(buf[buf_p..buf_p+2].try_into()?) {
                        1 => "Internet",
                        _ => unimplemented!(),
                    };

                    buf_p += 2;

                    let res_ttl = u32::from_be_bytes(buf[buf_p..buf_p+4].try_into()?);

                    buf_p += 4;

                    let res_rd_len = u16::from_be_bytes(buf[buf_p..buf_p+2].try_into()?);

                    buf_p += 2;

                    let ip_addr = Ipv4Addr::new(buf[buf_p], buf[buf_p+1], buf[buf_p+2], buf[buf_p+3]);

                    buf_p += 4;

                    println!("RECORD OF TYPE {:?} WITH CLASS {:?} AND TTL {:?}, IP ADDR: {:?}", res_type, res_class, res_ttl, ip_addr);
                } 
            }




        },
        Err(e) => println!("recv function failed: {e:?}"),
    }

    Ok(())
}
