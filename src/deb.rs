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
pub struct Control {
    package: &'static str,
    source: Option<&'static str>,
    version: &'static str,
    section: Option<&'static str>,
    priority: Option<&'static str>,
    architecture: &'static str,
    essential: Option<&'static str>,
    depends_et_al: Option<&'static str>,
    install_size: Option<u32>,
    maintainer: &'static str,
    description: &'static str,
    homepage: Option<&'static str>,
    built_using: Option<&'static str>,
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
            let control = fs::read_to_string(format!("{}control/control", extract_path))?;
            let control_contains = YamlLoader::load_from_str(&control).unwrap();
            println!("{:#?}", control_contains);

            Ok(Control {
                package: "",
                source: None,
                version: "",
                section: None,
                priority: None,
                architecture: "",
                essential: None,
                depends_et_al: None,
                install_size: None,
                maintainer: "",
                description: "",
                homepage: None,
                built_using: None,
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

    use crate::Deb;

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
        Deb::new("./example/assets/gnome_clocks.deb")
            .extract()?
            .retrieve_control()?;

        panic!();

        Ok(())
    }
}
