use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CliPaths {
    pub config_path: PathBuf,
    pub schedules_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl CliPaths {
    pub fn from_env() -> Result<Self, String> {
        Self::from_args(std::env::args().skip(1))
    }

    pub fn from_args<I>(mut args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut paths = Self::defaults();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--config" => {
                    paths.config_path = Self::next_path(&mut args, "--config")?;
                }
                "--schedules" => {
                    paths.schedules_dir = Self::next_path(&mut args, "--schedules")?;
                }
                "--logs" => {
                    paths.logs_dir = Self::next_path(&mut args, "--logs")?;
                }
                _ => return Err(format!("Unknown argument: {arg}")),
            }
        }
        Ok(paths)
    }

    fn next_path<I>(args: &mut I, flag: &str) -> Result<PathBuf, String>
    where
        I: Iterator<Item = String>,
    {
        args.next()
            .map(PathBuf::from)
            .ok_or_else(|| format!("Missing value for {flag}"))
    }

    fn defaults() -> Self {
        Self {
            config_path: PathBuf::from("config.json"),
            schedules_dir: PathBuf::from("schedules"),
            logs_dir: PathBuf::from("logs"),
        }
    }
}
