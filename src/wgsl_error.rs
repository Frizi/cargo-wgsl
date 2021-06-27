use naga::{front::wgsl::ParseError, valid::ValidationError};

#[derive(Debug)]
pub enum WgslError {
    ValidationErr(ValidationError),
    ParserErr {
        error: String,
        line: usize,
        pos: usize,
    },
    IoErr(std::io::Error),
}

impl From<std::io::Error> for WgslError {
    fn from(err: std::io::Error) -> Self {
        Self::IoErr(err)
    }
}

impl WgslError {
    pub fn from_parse_err(err: ParseError, src: &str, source_map: &crate::pp::SourceMap) -> Self {
        let (line, pos) = err.location(src);
        let error = err.emit_to_string(src);
        Self::ParserErr {
            error,
            line: source_map.map_line(line),
            pos,
        }
    }
}
