// // Copyright 2023 Developers of the exdsdevs project.
// //
// // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// // https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// // <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// // option. This file may not be copied, modified, or distributed
// // except according to those terms

use std::{fs::read_to_string, path::Path};

use serde::Deserialize;

use crate::errors::ExdsdevsError;

pub fn read_json_from_file<T: for<'a> Deserialize<'a>, P: AsRef<Path>>(
    file_path: P,
) -> Result<T, ExdsdevsError> {
    let json_string = read_to_string(file_path)?;
    let result = serde_json::from_str(&json_string)?;
    Ok(result)
}
