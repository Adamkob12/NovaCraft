use super::*;
pub(super) use std::cmp;

#[derive(Copy, Clone, Debug)]
pub enum GlobalParameter {
    RenderDistance,
    PbsValue,
    PbsMin,
    PbsSmoothing,
    Pbs,
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
        let pbs_value = OsStr::from("pbs-value");
        let pbs_min = OsStr::from("pbs-min");
        let pbs_smoothing = OsStr::from("pbs-smoothing");
        let pbs = OsStr::from("pbs");
        if matches!(value.cmp(&render_distance), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::RenderDistance);
        }
        if matches!(value.cmp(&pbs_min), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::PbsMin);
        }
        if matches!(value.cmp(&pbs), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::Pbs);
        }
        if matches!(value.cmp(&pbs_value), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::PbsValue);
        }
        if matches!(value.cmp(&pbs_smoothing), cmp::Ordering::Equal) {
            return Ok(GlobalParameter::PbsSmoothing);
        }

        Err(clap::Error::new(clap::error::ErrorKind::UnknownArgument))
    }
}
