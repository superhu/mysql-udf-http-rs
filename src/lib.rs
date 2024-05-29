//! This function is the bare minimum to do literally nothing

mod http;


use crate::http::http;
use udf::prelude::*;

struct HttpCall;

#[register]
impl BasicUdf for HttpCall {
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
        let mut request_url: String = String::new();
        if let Some(arg0) = args.get(0) {
            let req_url = arg0.value();
            if req_url.is_string() {
                if let Some(req_url0) = req_url.as_string() {
                    if req_url0.len() > 0 {
                        request_url = req_url0.to_string();
                    }
                }
            }
        };

        let mut request_params: Option<String> = None;
        if let Some(arg1) = args.get(1) {
            let req_params = arg1.value();
            if req_params.is_string() {
                if let Some(req_params0) = req_params.as_string() {
                    if req_params0.len() > 0 {
                        request_params = Some(req_params0.to_string());
                    }
                }
            }
        };

        let mut request_type: String = "GET".into();
        if let Some(arg2) = args.get(2) {
            let req_type = arg2.value();
            if req_type.is_string() {
                if let Some(req_type_str) = req_type.as_string() {
                    if req_type_str.len() > 0 {
                        request_type = req_type_str.to_string();
                    }
                }
            }
        };

        let mut request_header: String = String::new();
        if let Some(arg3) = args.get(3) {
            let req_header = arg3.value();
            if req_header.is_string() {
                if let Some(req_hader_str) = req_header.as_string() {
                    if req_hader_str.len() > 0 {
                        request_header = req_hader_str.to_string();
                    }
                }
            }
        };

        let mut request_body: Option<String> = None;
        if let Some(arg4) = args.get(4) {
            let req_body = arg4.value();
            if req_body.is_string() {
                if let Some(req_body_str) = req_body.as_string() {
                    if req_body_str.len() > 0 {
                        request_body = Some(req_body_str.to_string());
                    }
                }
            }
        };

        let mut request_proxy: Option<String> = None;
        if let Some(arg5) = args.get(5) {
            let req_proxy = arg5.value();
            if req_proxy.is_string() {
                if let Some(req_proxy_str) = req_proxy.as_string() {
                    if req_proxy_str.len() > 0 {
                        request_proxy = Some(req_proxy_str.to_string());
                    }
                }
            }
        };

        let result = http(
            request_url,
            request_params,
            request_type,
            request_header,
            request_body,
            request_proxy,
        );
        let r = match result {
            Ok(data) => data,
            Err(err) => {
                dbg!(err);
                None
            }
        };
        Ok(r)
    }
}
