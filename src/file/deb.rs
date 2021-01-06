use std::{
    fs,
    io::{Error, ErrorKind},
};

extern crate yaml_rust;
use glob::glob;
use yaml_rust::YamlLoader;

use crate::{
    file::{extract, Control, PathItem, Version},
    shared::PackageWithVersion,
};

/**
* This is the struct that should be used to parse `.deb` files.
* It should first be created with <Deb::new> and then extracted with <Deb::extract>. Once
* the file has been extracted you can then retrieve data from it.
*
* **Example**
* ```rust
* use std::io::Error;
* use deb_rs::file::Deb;
*
* fn main() -> Result<(), Error> {
*   let mut deb = Deb::new("./example/assets/gnome_clocks.deb");
*   deb.extract()?;
*
*   deb.version()?; // Returns the version of the structure of the debian package.
*   // NOTE: extract() will fail with versions that are not 2.0 as their structure is different
*
*   deb.retrieve_control()?; // Will return some general information about the contents of the package
*
*    deb.install_tree()?; // Returns an array of files that must be moved for the file package to work
*
*    Ok(())
* }
* ```
*/
pub struct Deb {
    path: &'static str,
    pub extracted_path: Option<String>,
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
        if self.extracted_path.is_none() {
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
            let input: Vec<&str> = control["Depends"].as_str().unwrap().split(',').collect();

            input
                .into_iter()
                .for_each(|dep| depends.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Pre-Depends"].is_badvalue() {
            let input: Vec<&str> = control["Pre-Depends"]
                .as_str()
                .unwrap()
                .split(',')
                .collect();

            input
                .into_iter()
                .for_each(|dep| pre_depends.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Recommends"].is_badvalue() {
            let input: Vec<&str> = control["Recommends"].as_str().unwrap().split(',').collect();

            input
                .into_iter()
                .for_each(|dep| recommends.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Suggests"].is_badvalue() {
            let input: Vec<&str> = control["Suggests"].as_str().unwrap().split(',').collect();

            input
                .into_iter()
                .for_each(|dep| suggests.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Enhances"].is_badvalue() {
            let input: Vec<&str> = control["Enhances"].as_str().unwrap().split(',').collect();

            input
                .into_iter()
                .for_each(|dep| enhances.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Breaks"].is_badvalue() {
            let input: Vec<&str> = control["Breaks"].as_str().unwrap().split(',').collect();

            input
                .into_iter()
                .for_each(|dep| breaks.push(PackageWithVersion::from_str(dep)));
        }

        if !control["Conflicts"].is_badvalue() {
            let input: Vec<&str> = control["Conflicts"].as_str().unwrap().split(',').collect();

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
