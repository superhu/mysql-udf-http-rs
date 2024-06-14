use std::{collections::HashMap, str::FromStr, time::Duration};

use reqwest::{blocking::ClientBuilder, header::{HeaderMap, HeaderName, HeaderValue}, Method, Proxy};
use anyhow::{Context, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use reqwest::blocking::multipart;
use serde_json::Value;
use serde::Deserialize;

pub fn convert(headers: &HeaderMap<HeaderValue>) -> Value {
    format!("{:?}", headers).into()
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct Body {
    fileList: Vec<MultipartFile>,
    parameterMap: HashMap<String, String>,
}
#[derive(Deserialize)]
struct MultipartFile {
    key: String,
    value: String,
    #[serde(rename = "originalFilename")]
    original_filename: String,
}
pub fn http(
    request_uri: String,
    request_params: Option<String>,
    request_type: String,
    request_header: String,
    request_body: Option<String>,
    request_proxy: Option<String>,
) -> Result<Option<String>> {
    let mut client_builder = ClientBuilder::new();
    if let Some(req_proxy) = request_proxy {
        if req_proxy.len() > 0 {
            dbg!(&req_proxy);
            let proxy = Proxy::http(&req_proxy).context("parse req_proxy error")?;
            client_builder = client_builder.proxy(proxy);
        }
    }
    let client = client_builder.danger_accept_invalid_certs(true).build()?;
    let mut url = String::new();
    url.push_str(&request_uri);
    if let Some(req_params) = request_params {
        if req_params.len() > 0 {
            url.push_str("?");
            url.push_str(&req_params);
        }
    }
    dbg!(&url);
    let mut header_map = HeaderMap::new();
    let mut is_multipart_request = false;
    let mut content_type = String::new();
    if request_header.len() > 0 {
        let headers_value: Value =
            serde_json::from_str(&request_header).unwrap_or_else(|_a| Value::Null);
        if let Some(map) = headers_value.as_object() {
            for (k, v) in map {
                let value = &v.as_str().unwrap_or_else(|| "");
                if value.starts_with("multipart/form-data") {
                    content_type = String::from(*value);
                    is_multipart_request = true;
                } else {
                    header_map.insert(
                        HeaderName::from_str(k)?,
                        HeaderValue::from_str(value)
                            .context("HeaderValue::from_str error".to_string())?,
                    );
                }
            }
        }
    }
    let mut request_builder = client.request(
        Method::from_str(&request_type).context(format!("at {}", line!()))?,
        url,
    );
    if &request_type != "GET" {
        if let Some(body) = request_body {
            if body.len() > 0 {
                dbg!(&is_multipart_request);
                dbg!(&content_type);
                if is_multipart_request {
                    let body_map: Body = serde_json::from_str(&body).with_context(|| {
                        dbg!(&body);
                        "parse body failed"
                    })?;
                    let mut form = multipart::Form::new();
                    let parameter_map = body_map.parameterMap;
                    for (k, v) in parameter_map {
                        dbg!(&k);
                        dbg!(&v);
                        form = form.text(k, v);
                    }
                    let file_list = body_map.fileList;
                    for file_map in file_list {
                        let file_bytes = BASE64_STANDARD.decode(file_map.value)?;
                        let key = file_map.key;
                        dbg!(&key);
                        let original_filename = file_map.original_filename;
                        dbg!(&original_filename);
                        // let _ = std::fs::write(&original_filename, file_bytes)?;
                        // form = form.file(key, original_filename)?;
                        let mut part = multipart::Part::bytes(file_bytes);
                        part = part.file_name(original_filename);
                        form = form.part(key, part);
                    }
                    dbg!(form.boundary());
                    request_builder = request_builder.multipart(form);
                } else {
                    dbg!("not multipart request");
                    request_builder = request_builder.body::<String>(body);
                }
            }
        }
    }
    if header_map.len() > 0 {
        request_builder = request_builder.headers(header_map);
    }
    let response = request_builder
        .timeout(Duration::from_secs(5))
        .send()
        .context(format!("send request error at:{}", line!()))?;
    let response_headers = response.headers();
    let mut result_map: HashMap<&str, Value> = HashMap::new();
    let header_value = convert(&response_headers);
    if let Some(content_type) = response_headers.get("content-type") {
        let content_type_str = String::from_utf8(Vec::from(content_type.as_bytes()))
            .context("content_type_str parse error")?;
        result_map.insert("content_type", content_type_str.into());
    }
    let status = response.status().as_u16().to_string();
    dbg!(&status);
    let result = response.text().context("response.text() error")?;
    result_map.insert(
        "status",
        Value::from_str(&status).context("status to Value error")?,
    );
    if status == "101" {
        dbg!(&header_value);
    }
    result_map.insert("headers", header_value);
    // dbg!(&result);
    result_map.insert(
        "result",
        Value::from_str(&result)
            .context("result to Value error")
            .unwrap_or_else(|_| Value::Null),
    );
    let result =
        serde_json::to_string(&result_map).context("serde_json::to_string(&result_map) error")?;
    Ok(Some(result))
}