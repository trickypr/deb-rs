use std::{
    fs,
    io::{Error, ErrorKind},
};

extern crate yaml_rust;
use yaml_rust::{YamlEmitter, YamlLoader};

use crate::extractor::extract;

/**
 * Type doc: https://www.debian.org/doc/debian-policy/ch-controlfields.html#s-binarycontrolfiles
 * YAML format
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    package: String,
    source: Option<String>,
    version: String,
    section: Option<String>,
    priority: Option<String>,
    architecture: String,
    essential: Option<String>,
    depends_et_al: Option<String>,
    install_size: Option<i64>, // There could be a better value for this, however rust-yaml outputs it as i64
    maintainer: String,
    description: String,
    homepage: Option<String>,
    built_using: Option<String>,
}

pub struct Deb {
    path: &'static str,
    extracted_path: Option<String>,
}

impl Deb {
    pub fn new(path: &'static str) -> Self {
        Deb {
            path,
            extracted_path: None,
        }
    }

    /**
     * Extracts the deb file into a set of individual files
     */
    pub fn extract(&mut self) -> Result<&mut Self, Error> {
        self.extracted_path = Some(extract(&self.path).unwrap());
        println!("{:?}", self.extracted_path);
        Ok(self)
    }

    pub fn retrieve_control(&self) -> Result<Control, Error> {
        if let Some(extract_path) = &self.extracted_path {
            let control_raw = fs::read_to_string(format!("{}control/control", extract_path))?;
            let control_contents = YamlLoader::load_from_str(&control_raw).unwrap();
            let control = &control_contents[0];

            let package = control["Package"].as_str().unwrap().to_string();
            let version = control["Version"].as_str().unwrap().to_string();
            let architecture = control["Architecture"].as_str().unwrap().to_string();
            let maintainer = control["Maintainer"].as_str().unwrap().to_string();
            let description = control["Description"].as_str().unwrap().to_string();

            let mut source = None;
            let mut section = None;
            let mut priority = None;
            let mut essential = None;
            let depends_et_al = None; // TODO: implement this
            let mut install_size = None;
            let mut homepage = None;
            let mut built_using = None;

            if !control["Source"].is_badvalue() {
                source = Some(control["Source"].as_str().unwrap().to_string());
            }

            if !control["section"].is_badvalue() {
                section = Some(control["section"].as_str().unwrap().to_string());
            }

            if !control["priority"].is_badvalue() {
                priority = Some(control["priority"].as_str().unwrap().to_string());
            }

            if !control["essential"].is_badvalue() {
                essential = Some(control["essential"].as_str().unwrap().to_string());
            }

            if !control["install_size"].is_badvalue() {
                install_size = Some(control["install_size"].as_i64().unwrap());
            }

            if !control["homepage"].is_badvalue() {
                homepage = Some(control["homepage"].as_str().unwrap().to_string());
            }

            if !control["built_using"].is_badvalue() {
                built_using = Some(control["built_using"].as_str().unwrap().to_string());
            }

            Ok(Control {
                package,
                source,
                version,
                section,
                priority,
                architecture,
                essential,
                depends_et_al,
                install_size,
                maintainer,
                description,
                homepage,
                built_using,
            })
        } else {
            Err(Error::new(
            ErrorKind::Other,
            "This deb file has not been extracted. Please run `extract()` before calling `retrieve_control`",
          ))
        }
    }
}

#[cfg(test)]
mod deb_test {
    use std::io::Error;

    use crate::{deb::Control, Deb};

    #[test]
    fn new() {
        Deb::new("./example/assets/gnome_clocks.deb");
    }

    #[test]
    fn extract() -> Result<(), Error> {
        let mut deb = Deb::new("./example/assets/gnome_clocks.deb");
        deb.extract()?;
        println!("{:?}", deb.extracted_path);
        Ok(())
    }

    #[test]
    fn retrieve_control() -> Result<(), Error> {
        let control = Deb::new("./example/assets/gnome_clocks.deb")
            .extract()?
            .retrieve_control()?;

        assert_eq!(control, Control {
            package: "gnome-clocks".to_string(),
            source: None,
            version: "3.30.1-2".to_string(),
            section: None,
            priority: None,
            architecture: "amd64".to_string(),
            essential: None,
            depends_et_al: None,
            install_size: None,
            maintainer: "Debian GNOME Maintainers <pkg-gnome-maintainers@lists.alioth.debian.org>".to_string(),
            description: "Simple GNOME app with stopwatch, timer, and world clock support GNOME Clocks is a simple application to show the time and date in multiple locations and set alarms or timers. A stopwatch is also included.".to_string(),
            homepage: None,
            built_using: None,
        });

        Ok(())
    }
}
