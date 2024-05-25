//! This function is the bare minimum to do literally nothing

mod http;

use udf::prelude::*;
use crate::http::http;

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
        let arg0 = args.get(0).unwrap().value();

        let s = arg0.as_string().unwrap().to_string();
        let result = http(&s);
        Ok(Some(result))
    }
}



