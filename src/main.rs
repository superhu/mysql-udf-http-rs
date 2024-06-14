use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket, TcpStream, TcpListener};
use std::os::unix::prelude::*;
use std::ffi::CStr;
use std::mem;
use std::str;
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::collections::VecDeque;

const BUFSIZE: usize = 65536;
const IPSIZE: usize = 4;
const VERSION: u8 = 0x05;
const NOAUTH: u8 = 0x00;
const CONNECT: u8 = 0x01;
const IP: u8 = 0x01;
const OK: u8 = 0x00;

fn readn(fd: &TcpStream, buf: &mut [u8]) -> io::Result<usize> {
    let mut left = buf.len();
    let mut buf_pos = 0;
    while left > 0 {
        let nread = fd.read(&mut buf[buf_pos..])?;
        if nread == 0 {
            return Ok(buf_pos);
        }
        left -= nread;
        buf_pos += nread;
    }
    Ok(buf_pos)
}

fn socks5_invitation(fd: &TcpStream) -> io::Result<()> {
    let mut init = [0u8; 2];
    readn(fd, &mut init)?;
    if init[0] != VERSION {
        return Err(io::Error::new(io::ErrorKind::Other, "Invalid version"));
    }
    Ok(())
}

fn socks5_auth(fd: &TcpStream) -> io::Result<()> {
    let answer = [VERSION, NOAUTH];
    fd.write_all(&answer)?;
    Ok(())
}

fn socks5_command(fd: &TcpStream) -> io::Result<u8> {
    let mut command = [0u8; 4];
    readn(fd, &mut command)?;
    Ok(command[3])
}

fn socks5_ip_read(fd: &TcpStream) -> io::Result<[u8; IPSIZE]> {
    let mut ip = [0u8; IPSIZE];
    readn(fd, &mut ip)?;
    Ok(ip)
}

fn socks5_read_port(fd: &TcpStream) -> io::Result<u16> {
    let mut port = [0u8; 2];
    readn(fd, &mut port)?;
    Ok(u16::from_be_bytes(port))
}

fn app_connect(ip: [u8; IPSIZE], port: u16) -> io::Result<TcpStream> {
    let address = format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    let addr = SocketAddrV4::new(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]), port);
    TcpStream::connect(addr)
}

fn socks5_ip_send_response(fd: &TcpStream, ip: [u8; IPSIZE], port: u16) -> io::Result<()> {
    let response = [VERSION, OK, 0, IP, ip[0], ip[1], ip[2], ip[3], (port >> 8) as u8, (port & 0xFF) as u8];
    fd.write_all(&response)?;
    Ok(())
}

fn app_socket_pipe(fd0: &TcpStream, fd1: &TcpStream) {
    let mut buffer_r = vec![0u8; BUFSIZE];
    loop {
        let nread = match fd0.read(&mut buffer_r) {
            Ok(n) if n > 0 => n,
            _ => break,
        };
        fd1.write_all(&buffer_r[..nread]).unwrap();
    }
}

fn worker(fd: TcpStream) {
    let mut inet_fd = TcpStream::new().unwrap();
    let command = socks5_command(&fd).unwrap();
    if command == CONNECT {
        let ip = socks5_ip_read(&fd).unwrap();
        let port = socks5_read_port(&fd).unwrap();
        inet_fd = app_connect(ip, port).unwrap();
        socks5_ip_send_response(&fd, ip, port).unwrap();
    }
    app_socket_pipe(&fd, &inet_fd);
    drop(inet_fd);
}

fn proxy(sock: TcpStream) {
    let mut a = [0u8; 1];
    write(sock.try_clone().unwrap(), "And this is my Child\n").unwrap();
    read(sock.try_clone().unwrap(), &mut a).unwrap();
    worker(sock);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1337").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    proxy(stream);
                });
            },
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }
}