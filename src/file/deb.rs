use std::{
    fs,
    io::{Error, ErrorKind},
};

use debcontrol::{parse_str, Paragraph};
use glob::glob;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    file::{extract, Control, PathItem, Version},
    shared::{paragraph_contains, PackageWithVersion},
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    fn get_control_string(&self, control: &Paragraph, query: &str) -> String {
        paragraph_contains(control.clone(), query.to_string())
            .unwrap()
            .value
    }

    fn str_option_to_number(&self, option: Option<String>) -> Option<u64> {
        if let Some(option) = option {
            Some(option.parse().unwrap())
        } else {
            None
        }
    }

    fn get_control_option_str(&self, control: &Paragraph, query: &str) -> Option<String> {
        let item = paragraph_contains(control.clone(), query.to_string());

        if let Some(item) = item {
            Some(item.value)
        } else {
            None
        }
    }

    fn get_package_name(&self, control: &Paragraph, query: &str) -> Vec<PackageWithVersion> {
        let item = paragraph_contains(control.clone(), query.to_string());

        let mut deps = Vec::new();

        if let Some(item) = item {
            let input: Vec<&str> = item.value.split(',').collect();

            input
                .into_iter()
                .for_each(|dep| deps.push(PackageWithVersion::from_str(dep)));
        }

        deps
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
        let control = parse_str(&control_raw).unwrap()[0].clone();

        let package = self.get_control_string(&control, "Package");
        let version = self.get_control_string(&control, "Version");
        let architecture = self.get_control_string(&control, "Architecture");
        let maintainer = self.get_control_string(&control, "Maintainer");
        let description = self
            .get_control_string(&control, "Description")
            .replace('\n', " ");

        let source = self.get_control_option_str(&control, "Source");
        let section = self.get_control_option_str(&control, "Section");
        let priority = self.get_control_option_str(&control, "Priority");
        let essential = self.get_control_option_str(&control, "Essential");
        let install_size = self.get_control_option_str(&control, "Installed-Size");
        let homepage = self.get_control_option_str(&control, "Homepage");
        let built_using = self.get_control_option_str(&control, "Built-Using");

        let install_size = self.str_option_to_number(install_size);

        // Depends et al
        let depends = self.get_package_name(&control, "Depends");
        let pre_depends = self.get_package_name(&control, "Pre-Depends");
        let recommends = self.get_package_name(&control, "Recommends");
        let suggests = self.get_package_name(&control, "Suggests");
        let enhances = self.get_package_name(&control, "Enhances");
        let breaks = self.get_package_name(&control, "Breaks");
        let conflicts = self.get_package_name(&control, "Conflicts");

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
