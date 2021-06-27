use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::wgsl_error::WgslError;

fn advance(slice: &mut &str, pattern: &str) -> bool {
    *slice = slice.trim_start();
    if let Some(rest) = slice.strip_prefix(pattern) {
        *slice = rest;
        true
    } else {
        false
    }
}

fn include_syntax_error(line_number: usize, message: &str) -> WgslError {
    WgslError::ParserErr {
        error: message.into(),
        line: line_number,
        pos: 0,
    }
}

fn include_io_error(line_number: usize, err: std::io::Error) -> WgslError {
    WgslError::ParserErr {
        error: format!("{}", err),
        line: line_number,
        pos: 10,
    }
}

#[derive(Default)]
pub struct SourceMap {
    map: Vec<(std::ops::Range<usize>, usize)>,
}

impl SourceMap {
    pub fn map_line(&self, input_line: usize) -> usize {
        self.map
            .iter()
            .rev()
            .skip_while(|(range, _)| range.start > input_line)
            .next()
            .map_or(input_line, |(range, map_to)| {
                if input_line >= range.end {
                    input_line - range.end + map_to
                } else {
                    *map_to
                }
            })
    }
}

pub fn load_shader_preprocessed(path: &Path) -> Result<(String, SourceMap), WgslError> {
    let mut set = HashSet::new();
    load_shader_preprocessed_recursive(path, &mut set)
}

fn load_shader_preprocessed_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<(String, SourceMap), WgslError> {
    let path = path.canonicalize()?;
    if visited.contains(&path) {
        return Ok((String::new(), SourceMap::default()));
    }
    visited.insert(path.clone());

    let source = std::fs::read_to_string(&path)?;
    let mut output = String::new();
    let mut output_lines = 0;
    let mut source_map = SourceMap::default();
    for (line_number, line) in source.lines().enumerate() {
        let mut tok = line.trim();
        if advance(&mut tok, "#include") {
            if !advance(&mut tok, "\"") {
                return Err(include_syntax_error(
                    line_number,
                    "expected '\"' after #include",
                ));
            }

            let (rel_path, rest) = tok.split_once("\"").ok_or_else(|| {
                include_syntax_error(line_number, "expected '\"' at the end of path")
            })?;
            tok = rest;

            if !(advance(&mut tok, ";") && tok.is_empty()) {
                return Err(include_syntax_error(
                    line_number,
                    "expected ';' after #include directive",
                ));
            }

            let full_path = path
                .parent()
                .map_or(PathBuf::from(rel_path), |parent| parent.join(rel_path));

            let (included_source, _) = load_shader_preprocessed_recursive(&full_path, visited)
                .map_err(|err| match err {
                    WgslError::IoErr(e) => include_io_error(line_number + 1, e),
                    WgslError::ParserErr { error, pos, .. } => WgslError::ParserErr {
                        error,
                        line: line_number + 1,
                        pos,
                    },
                    err => err,
                })?;
            let inserted_lines = included_source.lines().count();
            output.push_str(&included_source);
            output.push('\n');
            source_map
                .map
                .push((output_lines..output_lines + inserted_lines, line_number));
            output_lines += inserted_lines + 1;
        } else {
            output.push_str(line);
            output.push('\n');
            output_lines += 1;
        }
    }
    Ok((output, source_map))
}
