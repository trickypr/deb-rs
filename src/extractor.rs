use std::{
    fs::create_dir_all,
    io::{Error, ErrorKind},
    process::Command,
};

use cmd_lib::run_cmd;
use uuid::Uuid;

pub fn extract(archive_path: &str) -> Result<String, Error> {
    // Generates a unique id to stop this from conflicting
    let output = format!("/tmp/libdeb/{}/", Uuid::new_v4());
    println!("{}", output);

    if cfg!(target_os = "windows") {
        // We don't support windows
        return Err(Error::new(
            ErrorKind::Other,
            "The target_os windows is not supported. Please only use on linux",
        ));
    } else {
        // create_dir_all(&output)?;
        // create_dir_all(format!("{}data/", &output))?;
        // create_dir_all(format!("{}control/", &output))?;

        let data_archive = format!("{}data.tar.xz", &output);
        let data_extract = format!("{}data/", &output);

        let control_archive = format!("{}control.tar.xz", &output);
        let control_extract = format!("{}control/", &output);

        if run_cmd! {
            mkdir ${output};
            mkdir ${data_extract};
            mkdir ${control_extract};

            ar -x ${archive_path} --output=${output};
            tar -xf ${data_archive} -C ${data_extract};
            tar -xf ${control_archive} -C ${control_extract};
        }
        .is_err()
        {
            return Err(Error::new(ErrorKind::Other, "Error extracting files"));
        }
    }

    Ok(output)
}
