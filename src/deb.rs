use std::{
    fs,
    io::{Error, ErrorKind},
};

extern crate yaml_rust;
use glob::glob;
use yaml_rust::YamlLoader;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathItem {
    real: String,
    move_to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Version {
    V1_0,
    V2_0,
    VUnknown,
}

pub struct Deb {
    path: &'static str,
    extracted_path: Option<String>,
}
/**
 * @todo Add support for prerm and postinst files
 */
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
        Ok(self)
    }

    /**
     * Checks if the deb file has been extracted, throws and error if it has not
     */
    fn extract_check(&self) -> Result<(), Error> {
        if let None = &self.extracted_path {
            return Err(Error::new(
            ErrorKind::Other,
            "This deb file has not been extracted. Please run `extract()` before calling `retrieve_control`",
            ));
        };

        Ok(())
    }

    /**
     * Returns the version of the deb file. Note that the standard has been V2_0 (`2.0`) since debian `0.93`
     */
    pub fn version(&self) -> Result<Version, Error> {
        self.extract_check()?;

        let version = fs::read_to_string(format!(
            "{}debian-binary",
            self.extracted_path.as_ref().unwrap()
        ))?;

        let version = match &(*version) {
            "1.0\n" => Version::V1_0,
            "2.0\n" => Version::V2_0,
            _ => Version::VUnknown,
        };

        Ok(version)
    }

    /**
     * @todo Docs for this function
     */
    pub fn install_tree(&self) -> Result<Vec<PathItem>, Error> {
        let mut install_tree = Vec::new();

        let root = format!("{}data/", self.extracted_path.as_ref().unwrap());

        for entry in glob(&format!("{}**/*", root)).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let path = path.as_path();
                    let path_str = path.to_str().unwrap();
                    let path_rel_to_root: Vec<&str> = path_str.split(&root).collect();
                    let path_rel_to_root = format!("/{}", path_rel_to_root[1]);

                    if path.is_file() {
                        install_tree.push(PathItem {
                            real: path_str.to_string(),
                            move_to: path_rel_to_root,
                        });
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }

        Ok(install_tree)
    }

    /**
     * @todo Docs for this function
     */
    pub fn retrieve_control(&self) -> Result<Control, Error> {
        self.extract_check()?;

        let control_raw = fs::read_to_string(format!(
            "{}control/control",
            self.extracted_path.as_ref().unwrap()
        ))?;
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
    }
}

#[cfg(test)]
mod deb_test {
    use std::io::Error;

    use crate::{
        deb::{Control, Version},
        Deb,
    };

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

    #[test]
    fn version() -> Result<(), Error> {
        let version = Deb::new("./example/assets/gnome_clocks.deb")
            .extract()?
            .version()?;

        assert_eq!(version, Version::V2_0);

        Ok(())
    }

    #[test]
    fn install_tree() -> Result<(), Error> {
        let install_tree = Deb::new("./example/assets/gnome_clocks.deb")
            .extract()?
            .install_tree()?;

        assert_eq!(install_tree.len(), 302);

        Ok(())
    }
}
