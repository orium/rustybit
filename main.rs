use std::io::TcpStream;

mod marshalling;
mod message;
mod config;

fn send_version(socket : &mut TcpStream)
{
    let version = message::Version::new(config::NAME.to_string(),
                                        config::version());

    socket.write(version.serialize().as_slice());
}

fn main()
{
    let mut socket = TcpStream::connect("192.168.1.2", 8333).unwrap();

    send_version(&mut socket);

    let response = socket.read_to_end();

    println!("{}", response);
}
