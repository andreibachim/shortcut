use anyhow::anyhow;
use std::path::{Path, PathBuf};

pub fn set_icon(
    image: &gtk::Image,
    icon_path: Option<&str>,
    use_placeholder: bool,
) -> anyhow::Result<()> {
    if let Some(icon_path) = icon_path {
        if PathBuf::from(icon_path).is_absolute() {
            if Path::new(icon_path).exists() {
                image.set_from_file(Some(icon_path));
            } else if use_placeholder {
                set_placeholder_icon(image);
            } else {
                return Err(anyhow!("No icon found at given path"));
            }
        } else {
            let themed_icon = gtk::gio::ThemedIcon::from_names(&[icon_path]);
            if gtk::IconTheme::default().has_gicon(&themed_icon) {
                image.set_gicon(Some(&themed_icon));
            } else {
                return Err(anyhow!("The relative path does not point to any icon"));
            }
        }
    } else if use_placeholder {
        set_placeholder_icon(image);
    } else {
        return Err(anyhow!("Given path is 'None'"));
    };
    Ok(())
}

fn set_placeholder_icon(image: &gtk::Image) {
    let themed_icon = gtk::gio::ThemedIcon::from_names(&["application-x-executable"]);
    image.set_gicon(Some(&themed_icon));
}
