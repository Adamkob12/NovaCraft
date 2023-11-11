use super::*;
pub(super) use std::cmp;

#[derive(Copy, Clone, Debug)]
pub enum GlobalParameter {
    RenderDistance,
    SLintensity,
    SLmax,
    SLsmoothing,
    SL,
}

impl clap::builder::ValueParserFactory for GlobalParameter {
    type Parser = GlobalParameterParser;
    fn value_parser() -> Self::Parser {
        GlobalParameterParser
    }
}

#[derive(Clone, Debug)]
pub struct GlobalParameterParser;
impl clap::builder::TypedValueParser for GlobalParameterParser {
    type Value = GlobalParameter;

    #[allow(unused)]
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let render_distance = OsStr::from("render-distance");
        let sl_intensity = OsStr::from("sli");
        let sl_max = OsStr::from("slm");
        let sl_smoothing = OsStr::from("sls");
        let sl = OsStr::from("sl");
        if matches!(value.cmp(&render_distance), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::RenderDistance);
        }
        if matches!(value.cmp(&sl_max), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::SLmax);
        }
        if matches!(value.cmp(&sl), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::SL);
        }
        if matches!(value.cmp(&sl_intensity), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::SLintensity);
        }
        if matches!(value.cmp(&sl_smoothing), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::SLsmoothing);
        }

        Err(clap::Error::new(clap::error::ErrorKind::UnknownArgument))
    }
}
