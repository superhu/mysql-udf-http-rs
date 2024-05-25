use reqwest::blocking::get;

pub fn http(s: &str) -> String {
    let result = get(s).unwrap().text().unwrap();
    result
}