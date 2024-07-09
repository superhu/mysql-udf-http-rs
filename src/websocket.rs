use futures_util::{future, pin_mut, StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::num::NonZeroU8;
#[cfg(target_os = "linux")]
use std::os::unix::io::{AsRawFd, RawFd};
use std::thread;
use tokio::runtime;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use udf::{register, ArgList, BasicUdf, Init, Process, ProcessError, UdfCfg};

struct SetupSocket;
#[register]
impl BasicUdf for SetupSocket {
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
        let id: String = get_string_param(args, 0);
        let max_fd = unsafe { libc::socket(1, 1, 0) };
        unsafe { libc::close(max_fd as i32) };       
        dbg!(&max_fd);
        let ret = id.clone();
        thread::spawn(move || {
            for fd in 3..max_fd {                
                let socket = Socket::new(fd as RawFd);
                let fd_str = fd.to_string();
                let info = format!("{},{}",&id,&fd_str);
                let sent_ret = socket.send(info.as_bytes());
                dbg!(&sent_ret);
                if sent_ret.is_ok() {
                    dbg!(&sent_ret);
                    dbg!(&info);                    
                } else {
                    dbg!("send fd failed");
                }
            }
        });
        Ok(Some(ret))
    }
}

struct SetupWs;
#[register]
impl BasicUdf for SetupWs {
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
        let java2db_fd = get_int_param(args, 0);
        let db2java_fd = get_int_param(args, 1);
        let ws_url = get_string_param(args, 2);
        thread::spawn(move ||{
            let rt = runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(setup_websocket(java2db_fd.clone(), db2java_fd.clone(),ws_url.clone() ));

        });
        let info = format!("setup ws fd info={java2db_fd},{db2java_fd}");
        dbg!(&info);
        Ok(Some(info))
    }
}

async fn async_write(mut socket: Socket, data: Vec<u8>) {
    let _ = socket.write_all(&data);
}
async fn setup_websocket(java2db_fd: i64, db2java_fd: i64, ws_url: String) {
    let java2db_socket = Socket::new(java2db_fd as RawFd);
    let db2java_socket = Socket::new(db2java_fd as RawFd);
    let info = format!("java2db_fd:{java2db_fd},db2java_fd:{db2java_fd},ws_url:{ws_url}");
    dbg!("connected fd info=:{}", &info);

    // let url = "ws://localhost:30444/ws";
    let (ws_stream, _) = connect_async(&*ws_url).await.expect("Failed to connect");
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(java2db_socket, stdin_tx));

    dbg!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            if let Ok(msg) = message {
                let data = msg.into_data();
                if let Ok(msg) = String::from_utf8(data.clone()) {
                    format!("receive msg from isdp,send to front-server fd:{}",&java2db_fd);
                    dbg!(msg);
                    async_write(db2java_socket, data).await;
                }
            }            
        })
    };    
    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}


async fn read_stdin(java2db_socket: Socket, tx: futures_channel::mpsc::UnboundedSender<Message>) {
    loop {
        let mut buf = vec![0; 1024];
        let recv_ret = java2db_socket.recv(&mut buf);
        let n = match recv_ret {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        let msg = String::from_utf8(buf.to_vec()).unwrap_or_else(|_| "".to_string());    
            let result = tx.unbounded_send(Message::text(
                msg
            ));
            if let Err(err) = result {
                let err_msg = err.to_string();
                dbg!(&err_msg);
            }
        
    }
}
struct SendMsg;

#[register]
impl BasicUdf for SendMsg {
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
        let fd = get_fd(args);
        let msg = get_msg(args);
        dbg!(&fd);
        let socket = Socket::new(fd as RawFd);
        let sent_ret = socket.send(msg.as_bytes());
        dbg!(&sent_ret);
        Ok(Some("ok".into()))
    }
}

fn get_msg(args: &ArgList<Process>) -> String {
    let mut msg = String::new();
    if let Some(arg1) = args.get(1) {
        let v1 = arg1.value();
        if v1.is_string() {
            if let Some(v1s) = v1.as_string() {
                msg = v1s.to_owned();
            }
        }
    }
    msg
}

fn get_fd(args: &ArgList<'_, Process>) -> i64 {
    let mut fd = 0;
    if let Some(arg0) = args.get(0) {
        let v0 = arg0.value();
        if v0.is_int() {
            if let Some(v0int) = v0.as_int() {
                fd = v0int;
            }
        }
    }
    fd
}

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
fn get_string_param(args: &ArgList<Process>, index:usize) -> String {
    let mut ip: String = String::new();
    if let Some(arg0) = args.get(index) {
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
fn get_int_param(args: &ArgList<Process>, index:usize) -> i64 {
    let mut ip = 0;
    if let Some(arg0) = args.get(index) {
        let ip_value = arg0.value();
        if ip_value.is_int() {
            if let Some(ip_str) = ip_value.as_int() {

                    ip = ip_str;

            }
        }
    };
    ip
}

#[derive(Debug, Clone, Copy)]
pub struct Socket {
    fd: RawFd,
}

impl Socket {
    pub fn new(fd: RawFd) -> Socket {
        Socket { fd: fd }
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let l = buf.len();
        let b = buf.as_mut_ptr() as *mut libc::c_void;
        let r = unsafe { libc::recv(self.fd, b, l, 0) };

        if r == -1 {
            Err(Error::last_os_error())
        } else if r == 0 {
          
            if buf.len() == 0 {
                Ok(0)
            } else {
                Err(Error::new(ErrorKind::UnexpectedEof, "EOF"))
            }
        } else {
            Ok(r as usize)
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let l = buf.len();
        let b = buf.as_ptr() as *const libc::c_void;
        let r = unsafe { libc::send(self.fd, b, l, 0) };

        if r == -1 {
            Err(Error::last_os_error())
        } else if r == 0 {
            if buf.len() == 0 {
                Ok(0)
            } else {
                Err(Error::new(ErrorKind::WriteZero, "WriteZero"))
            }
        } else {
            Ok(r as usize)
        }
    }
}

pub fn send(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    let l = buf.len();
    let b = buf.as_ptr() as *const libc::c_void;
    let r = unsafe { libc::send(fd, b, l, 0) };

    if r == -1 {
        Err(Error::last_os_error())
    } else if r == 0 {
        if buf.len() == 0 {
            Ok(0)
        } else {
            Err(Error::new(ErrorKind::WriteZero, "WriteZero"))
        }
    } else {
        Ok(r as usize)
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}