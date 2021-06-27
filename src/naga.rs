use naga::{
    front::wgsl,
    valid::{Capabilities, ValidationFlags},
};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::wgsl_error::WgslError;

pub struct Naga {
    validator: naga::valid::Validator,
}

impl Naga {
    pub fn new() -> Self {
        Self {
            validator: naga::valid::Validator::new(ValidationFlags::all(), Capabilities::all()),
        }
    }

    pub fn validate_wgsl(&mut self, path: &Path) -> Result<(), WgslError> {
        let (shader, src_map) = crate::pp::load_shader_preprocessed(path)?;
        let module = wgsl::parse_str(&shader)
            .map_err(|err| WgslError::from_parse_err(err, &shader, &src_map))?;

        if let Err(err) = self.validator.validate(&module) {
            Err(WgslError::ValidationErr(err))
        } else {
            Ok(())
        }
    }

    pub fn get_wgsl_tree(&mut self, path: &Path) -> Result<WgslTree, WgslError> {
        let (shader, src_map) = crate::pp::load_shader_preprocessed(path)?;
        let module = wgsl::parse_str(&shader)
            .map_err(|err| WgslError::from_parse_err(err, &shader, &src_map))?;

        let mut types = Vec::new();
        let mut global_variables = Vec::new();
        let mut functions = Vec::new();

        for (_, ty) in module.types.iter() {
            if let Some(name) = &ty.name {
                types.push(name.clone())
            }
        }

        for (_, var) in module.global_variables.iter() {
            if let Some(name) = &var.name {
                global_variables.push(name.clone())
            }
        }

        for (_, f) in module.functions.iter() {
            if let Some(name) = &f.name {
                functions.push(name.clone())
            }
        }

        Ok(WgslTree {
            types,
            global_variables,
            functions,
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WgslTree {
    types: Vec<String>,
    global_variables: Vec<String>,
    functions: Vec<String>,
}
