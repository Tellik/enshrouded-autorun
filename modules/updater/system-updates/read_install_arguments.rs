use std::path::PathBuf;

pub(crate) struct Arguments {
    pub(crate) root: PathBuf,
    pub(crate) staged: PathBuf,
}

impl Arguments {
    pub(crate) fn read() -> Result<Self, String> {
        let values: Vec<String> = std::env::args().collect();
        let value = |name: &str| {
            values
                .windows(2)
                .find(|pair| pair[0] == name)
                .map(|pair| pair[1].clone())
        };
        Ok(Self {
            root: PathBuf::from(value("--root").ok_or("missing --root")?),
            staged: PathBuf::from(value("--staged").ok_or("missing --staged")?),
        })
    }
}
