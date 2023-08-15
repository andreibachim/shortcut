use std::{error::Error, fmt::Write};

#[derive(Default, Debug)]
pub struct Desktop {
    r#type: String,
    pub name: String,
    pub exec: String,
    pub icon: String,
}

impl Desktop {
    pub fn new() -> Self {
        Self {
            r#type: "Application".to_owned(),
            ..Default::default()
        }
    }

    pub fn get_output(&self) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();

        writeln!(&mut output, "[Desktop Entry]")?;
        writeln!(&mut output, "Type={}", self.r#type)?;
        writeln!(&mut output, "Name={}", self.name)?;
        writeln!(&mut output, "Exec={}", self.exec)?;
        writeln!(&mut output, "Icon={}", self.icon)?;

        Ok(output)
    }
}
