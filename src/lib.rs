//! This function is the bare minimum to do literally nothing

use udf::prelude::*;
use reqwest::blocking::*;
struct EmptyCall;

#[register]
impl BasicUdf for EmptyCall {
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
        let result = get(s).unwrap().text().unwrap();
        Ok(Some(result))
    }
}

