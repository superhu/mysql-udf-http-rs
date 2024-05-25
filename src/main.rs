use crate::http::http;

mod http;
fn main() {

    let result = http("https://www.baidu.com");
    println!("{}", result);
}