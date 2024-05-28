use std::{collections::HashMap, str::FromStr, time::Duration};

use reqwest::{blocking::{ClientBuilder}, header::{HeaderMap, HeaderName, HeaderValue}, Method, Proxy};
use anyhow::{Context, Result};
use serde_json::Value;

pub fn convert(headers: &HeaderMap<HeaderValue>) -> Value {
    format!("{:?}", headers).into()
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
    let client = client_builder
        .danger_accept_invalid_certs(true)
        .build()
        .context("parse client_builder.build() error")?;

    let mut url = String::new();
    url.push_str(&request_uri);
    if let Some(req_params) = request_params {
        url.push_str("?");
        url.push_str(&req_params);
    }
    let mut header_map = HeaderMap::new();
    if request_header.len() > 0 {
        let headers_value: Value =
            serde_json::from_str(&request_header).unwrap_or_else(|_a| Value::Null);
        if let Some(map) = headers_value.as_object() {
            for (k, v) in map {
                header_map.insert(
                    HeaderName::from_str(k)?,
                    HeaderValue::from_str(v.as_str().unwrap_or_else(|| ""))
                        .context("HeaderValue::from_str error".to_string())?,
                );
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
                request_builder = request_builder.body::<String>(body);
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
    dbg!(&result);
    result_map.insert(
        "result",
        Value::from_str(&result).context("result to Value error")?,
    );
    result_map.insert("headers", header_value);
    let result =
        serde_json::to_string(&result_map).context("serde_json::to_string(&result_map) error")?;
    Ok(Some(result))
}