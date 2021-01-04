// TODO: Refactor this entire file. 300 lines is too long

use std::{
    fs,
    io::{Error, ErrorKind},
};

extern crate yaml_rust;
use glob::glob;
use yaml_rust::YamlLoader;

use crate::extractor::extract;

/**
 * Type doc: <https://www.debian.org/doc/debian-policy/ch-controlfields.html#s-binarycontrolfiles>
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
    install_size: Option<i64>, // There could be a better value for this, however rust-yaml outputs it as i64
    maintainer: String,
    description: String,
    homepage: Option<String>,
    built_using: Option<String>,
    // Depends et al: <https://www.debian.org/doc/debian-policy/ch-relationships.html#s-binarydeps>
    depends: Vec<PackageWithVersion>,
    pre_depends: Vec<PackageWithVersion>,
    recommends: Vec<PackageWithVersion>,
    suggests: Vec<PackageWithVersion>,
    enhances: Vec<PackageWithVersion>,
    breaks: Vec<PackageWithVersion>,
    conflicts: Vec<PackageWithVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageWithVersion {
    name: String,
    version: String,
    binding: VersionBinding,
}

impl PackageWithVersion {
    pub fn from_str(contents: &str) -> Self {
        let split: Vec<&str> = contents.split(")").collect::<Vec<&str>>()[0]
            .split(" (")
            .collect();

        if split.len() == 2 {
            let name = split[0].to_string();
            let mut version = split[1].to_string();
            let mut version_binding = String::new();
            loop {
                let first = version.chars().nth(0).unwrap();

                if !(first == '=' || first == '>' || first == '<' || first == ' ') {
                    break;
                } else {
                    version_binding.push(first);
                    version.remove(0);
                }
            }

            PackageWithVersion {
                name,
                version,
                binding: VersionBinding::from_str(&version_binding),
            }
        } else {
            let name = split[0].to_string();

            PackageWithVersion {
                name,
                version: String::new(),
                binding: VersionBinding::Any,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionBinding {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    Any,
    Unknown,
}

impl VersionBinding {
    fn from_str(s: &str) -> Self {
        let s = s.split(" ").collect::<Vec<&str>>()[0];

        match s {
            ">" => VersionBinding::GreaterThan,
            "<" => VersionBinding::LessThan,
            ">=" => VersionBinding::GreaterThanOrEqual,
            "<=" => VersionBinding::LessThanOrEqual,
            "=" => VersionBinding::Equal,
            _ => VersionBinding::Unknown,
        }
    }
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
        let mut install_size = None;
        let mut homepage = None;
        let mut built_using = None;

        if !control["Source"].is_badvalue() {
            source = Some(control["Source"].as_str().unwrap().to_string());
        }

        if !control["Section"].is_badvalue() {
            section = Some(control["Section"].as_str().unwrap().to_string());
        }

        if !control["Priority"].is_badvalue() {
            priority = Some(control["Priority"].as_str().unwrap().to_string());
        }

        if !control["Essential"].is_badvalue() {
            essential = Some(control["Essential"].as_str().unwrap().to_string());
        }

        if !control["Installed-Size"].is_badvalue() {
            install_size = Some(control["Installed-Size"].as_i64().unwrap());
        }

        if !control["Homepage"].is_badvalue() {
            homepage = Some(control["Homepage"].as_str().unwrap().to_string());
        }

        if !control["Built-Using"].is_badvalue() {
            built_using = Some(control["Built-Using"].as_str().unwrap().to_string());
        }

        // Depends et al
        let mut depends = Vec::new();
        let mut pre_depends = Vec::new();
        let mut recommends = Vec::new();
        let mut suggests = Vec::new();
        let mut enhances = Vec::new();
        let mut breaks = Vec::new();
        let mut conflicts = Vec::new();

        if !control["Depends"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| depends.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Pre-Depends"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| pre_depends.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Recommends"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| recommends.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Suggests"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| suggests.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Enhances"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| enhances.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Breaks"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| breaks.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Conflicts"].is_badvalue() {
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(",").collect();

            input
                .into_iter()
                .for_each(|dep| conflicts.push(PackageWithVersion::from_str(dep)));
        }

        Ok(Control {
            package,
            source,
            version,
            section,
            priority,
            architecture,
            essential,
            install_size,
            maintainer,
            description,
            homepage,
            built_using,
            depends,
            pre_depends,
            recommends,
            suggests,
            breaks,
            enhances,
            conflicts,
        })
    }
}

#[cfg(test)]
mod deb_test {
    use std::io::Error;

    use crate::{deb::Version, Deb};

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

        assert_eq!(control.package, "gnome-clocks".to_string());
        assert_eq!(control.source, None);
        assert_eq!(control.version, "3.30.1-2".to_string());
        assert_eq!(control.section, Some("gnome".to_string()));
        assert_eq!(control.priority, Some("optional".to_string()));
        assert_eq!(control.architecture, "amd64".to_string());
        assert_eq!(control.essential, None);
        assert_eq!(control.install_size, Some(1735));
        assert_eq!(
            control.maintainer,
            "Debian GNOME Maintainers <pkg-gnome-maintainers@lists.alioth.debian.org>".to_string()
        );
        assert_eq!(control.description, "Simple GNOME app with stopwatch, timer, and world clock support GNOME Clocks is a simple application to show the time and date in multiple locations and set alarms or timers. A stopwatch is also included.".to_string());
        assert_eq!(
            control.homepage,
            Some("https://wiki.gnome.org/Apps/Clocks".to_string())
        );
        assert_eq!(control.built_using, None);
        assert_eq!(control.depends.len(), 12);
        assert_eq!(control.pre_depends.len(), 0);
        assert_eq!(control.recommends.len(), 0);
        assert_eq!(control.suggests.len(), 0);
        assert_eq!(control.enhances.len(), 0);
        assert_eq!(control.breaks.len(), 0);
        assert_eq!(control.conflicts.len(), 0);

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
