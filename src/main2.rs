use std::fs;
use std::fs::File;
use std::io::Read;
use std::os::windows::io::AsRawSocket;
use std::time::Duration;
use crate::http::http;
use anyhow::Result;
use reqwest::blocking::multipart;
use reqwest::Client;
use tokio::runtime::Runtime;
use futures_util::{
    SinkExt,
    StreamExt,
    TryStreamExt,
};
use reqwest_websocket::{
    Message,
    RequestBuilderExt,
};
use libc::socket;
use tokio::net::TcpSocket;

mod http;

#[tokio::main]
async fn main() -> Result<()> {
    // let result = http("https://httpbin.dev/get".into()
    // ,None
    //     ,"GET".into()
    //     ,"{\"accept\": \"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\", \"accept-language\": \"zh-CN,zh;q=0.9\", \"cache-control\": \"no-cache\", \"pragma\": \"no-cache\", \"priority\": \"u=0, i\", \"sec-ch-ua\": \"\\\"Google Chrome\\\";v=\\\"125\\\", \\\"Chromium\\\";v=\\\"125\\\", \\\"Not.A/Brand\\\";v=\\\"24\\\"\", \"sec-ch-ua-mobile\": \"?0\", \"sec-ch-ua-platform\": \"\\\"Windows\\\"\", \"sec-fetch-dest\": \"document\", \"sec-fetch-mode\": \"navigate\", \"sec-fetch-site\": \"none\", \"sec-fetch-user\": \"?1\", \"upgrade-insecure-requests\": \"1\"}".into()
    //     ,None,None);
    // println!("{:?}", result);

    // let client = reqwest::blocking::ClientBuilder::new()
    //     .danger_accept_invalid_certs(true)
    //     .build()?;
    //
    // let form = multipart::Form::new()
    //     .text("username", "seanmonstar")
    //     .text("age", "1")
    //     ;
    //
    // let bytes = std::fs::read("D:/Pictures/11.png")?;
    // let part = multipart::Part::bytes(bytes)
    //     .file_name("11.png")
    //     // .mime_str("text/plain")
    //     // ?
    //     ;
    // let form = form.part("upload", part);
    //
    // let response = client.post("http://172.27.80.1:8888/upload")
    //     .multipart(form)
    //     .timeout(Duration::from_secs(4))
    //     .send()
    //     .unwrap();
    // let result1 = response.text()?;
    // println!("{}", result1);

    let fd = unsafe { libc::socket(1,1,0) };

    let socket = TcpSocket::new_v4()?;

    let websocket = Client::default()
        .get("wss://echo.websocket.org/")
        .upgrade()
        .send()
        .await?
        .into_websocket()
        .await?;

    let (mut tx, mut rx) = websocket.split();

    tokio::spawn(async move {
        for i in 1..11 {
            tx.send(Message::Text(format!("Hello, World! #{i}")))
                .await
                .unwrap();
        }
    });

    while let Some(message) = rx.try_next().await? {
        match message {
            Message::Text(text) => println!("received: {text}"),
            _ => {}
        }
    }
    Ok(())
}

