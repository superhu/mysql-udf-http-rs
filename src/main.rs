use crate::http::http;

mod http;
fn main() {
    let result = http("https://httpbin.dev/get".into()
    ,None
        ,"GET".into()
        ,"{\"accept\": \"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\", \"accept-language\": \"zh-CN,zh;q=0.9\", \"cache-control\": \"no-cache\", \"pragma\": \"no-cache\", \"priority\": \"u=0, i\", \"sec-ch-ua\": \"\\\"Google Chrome\\\";v=\\\"125\\\", \\\"Chromium\\\";v=\\\"125\\\", \\\"Not.A/Brand\\\";v=\\\"24\\\"\", \"sec-ch-ua-mobile\": \"?0\", \"sec-ch-ua-platform\": \"\\\"Windows\\\"\", \"sec-fetch-dest\": \"document\", \"sec-fetch-mode\": \"navigate\", \"sec-fetch-site\": \"none\", \"sec-fetch-user\": \"?1\", \"upgrade-insecure-requests\": \"1\"}".into()
        ,None,None);
    println!("{:?}", result);
}
