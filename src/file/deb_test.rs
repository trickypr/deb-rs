use std::io::Error;

use crate::file::{Deb, Version};

#[test]
fn new() {
    Deb::new("./example/assets/gnome_clocks.deb");
}

#[test]
fn extract() -> Result<(), Error> {
    let mut deb = Deb::new("./example/assets/gnome_clocks.deb");
    deb.extract()?;
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
fn secondary_control() -> Result<(), Error> {
    let control = Deb::new("./example/assets/exa.deb")
        .extract()?
        .retrieve_control()?;

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
