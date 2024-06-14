use std::num::NonZeroU8;
use std::io::{self, Error, ErrorKind, Read, Write};
use std::mem;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;
use udf::{ArgList, BasicUdf, Init, Process, ProcessError, register, UdfCfg};

struct Websocket;

#[register]
impl BasicUdf for Websocket {
    type Returns<'a> = Option<String>;

    fn init(_cfg: &UdfCfg<Init>, _args: &ArgList<Init>) -> Result<Self, String> {
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let ip = Self::get_ip(args);
        let max_fd = unsafe { libc::socket(1, 1, 0) };
        unsafe { libc::close(max_fd as i32) };
        // https://users.rust-lang.org/t/help-understanding-libc-call/17308
        let mut storage: libc::sockaddr_storage = unsafe { mem::zeroed() };
        let socket_addr: *mut libc::sockaddr = &mut storage as *mut _ as *mut _;
        // let socket_addr: *mut libc::sockaddr = std::ptr::null_mut() as *mut libc::sockaddr;
        let addr_size0 = std::mem::size_of::<libc::sockaddr>();
        let addr_size: *mut libc::c_uint = addr_size0 as *mut libc::c_uint;

        dbg!(&max_fd);
        for fd in 3..max_fd {
            dbg!(&fd);
            // On success, zero is returned.  On error, -1 is returned
            let ret = unsafe { libc::getpeername(fd, socket_addr, addr_size) };
            dbg!(ret);
            if ret == 0 { // let client_addr: libc::sockaddr = unsafe { std::mem::transmute(socket_addr) };
                let sa_data = (unsafe { *socket_addr }).sa_data;
                let sa_ptr = sa_data.as_ptr();
                let sa_cstr = unsafe { std::ffi::CStr::from_ptr(sa_ptr) };
                dbg!(sa_cstr);
                if let Ok(ip) = sa_cstr.to_str() {
                    let my_ip = Self::get_ip(&args);
                    dbg!(&ip);
                    dbg!(&my_ip);
                    dbg!(&fd);
                    if ip == my_ip {
                        let socket = Socket::new(fd as RawFd);
                        let _ = socket.send(b"123456789\n");
                        // let mut tcp_stream = unsafe { TcpStream::from_raw_fd(fd as RawFd) };
                        // let _ = tcp_stream.write(b"123456789\n");
                        // let _ = tcp_stream.flush();
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_secs(3));
        Ok(Some(ip))
    }
}

impl Websocket {
    fn get_ip(args: &ArgList<Process>) -> String {
        let mut ip: String = String::new();
        if let Some(arg0) = args.get(0) {
            let ip_value = arg0.value();
            if ip_value.is_string() {
                if let Some(ip_str) = ip_value.as_string() {
                    if ip_str.len() > 0 {
                        ip = ip_str.to_string();
                    }
                }
            }
        };
        ip
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Socket {
    fd: RawFd,
}

impl Socket {
    pub fn new(fd: RawFd) -> Socket { Socket { fd: fd } }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let l = buf.len();
        let b = buf.as_mut_ptr() as *mut libc::c_void;
        let r = unsafe { libc::recv(self.fd, b, l, 0) };

        // These calls return the number of bytes received, or -1 if an error
        // occurred.  In the event of an error, errno is set to indicate the
        // error.
        if r == -1 {
            Err(Error::last_os_error())
        } else if r == 0 {
            // When a stream socket peer has performed an orderly shutdown, the
            // return value will be 0 (the traditional "end-of-file" return).
            //
            // The value 0 may also be returned if the requested number of bytes
            // to receive from a stream socket was 0.
            if buf.len() == 0 { Ok(0) } else { Err(Error::new(ErrorKind::UnexpectedEof, "EOF")) }
        } else {
            Ok(r as usize)
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let l = buf.len();
        let b = buf.as_ptr() as *const libc::c_void;
        let r = unsafe { libc::send(self.fd, b, l, 0) };

        // On success, these calls return the number of bytes sent.
        // On error, -1 is returned, and errno is set appropriately.
        if r == -1 {
            Err(Error::last_os_error())
        } else if r == 0 {
            if buf.len() == 0 { Ok(0) } else { Err(Error::new(ErrorKind::WriteZero, "WriteZero")) }
        } else {
            Ok(r as usize)
        }
    }
}

impl AsRawFd for Socket { fn as_raw_fd(&self) -> RawFd { self.fd } }

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.recv(buf) }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.send(buf) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}