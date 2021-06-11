use std::net::UdpSocket;

fn main() -> () {
	let socket;
	match UdpSocket::bind("127.0.0.1:34256") {
		Ok(x) => socket = x,
		Err(_) => panic!("AA"),
	}
	let buf = b"testing";
	socket.send_to(buf, "127.0.0.1:34254");
}
