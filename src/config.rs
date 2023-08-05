use std::{default, str::FromStr};

use crate::{storage, time, GlobalError};

pub struct Config {
    pub print_color: bool,
    pub ls_default_period: time::Period,
}

impl Config {
    const CONFIG_FILE_NAME: &'static str = "config.db";

    /// size of the print color field
    pub const PRINT_COLOR_SIZE: usize = 1;
    /// size of the ls default period field
    const LS_DEFAULT_PERIOD_SIZE: usize = 4;
    /// total size of the config
    const SIZE: usize = Self::PRINT_COLOR_SIZE + Self::LS_DEFAULT_PERIOD_SIZE;

    pub fn load() -> Result<Self, GlobalError> {
        let buffer = storage::load(Self::CONFIG_FILE_NAME)?;
        if buffer.len() == 0 {
            return Ok(Self::default());
        } else if buffer.len() != Self::SIZE {
            return Err(GlobalError::wrong_byte_len("config", buffer.len(), Self::SIZE));
        }
        let print_color = buffer[0] != 0;
        let ls_default_period_bytes = buffer[1..5]
            .try_into()
            .expect("The size was checked before");
        let ls_default_period_in_days = u32::from_le_bytes(ls_default_period_bytes);
        Ok(Self {
            ls_default_period: time::Period::Days(ls_default_period_in_days),
            print_color,
        })
    }

    pub fn save(&self) -> Result<(), GlobalError> {
        let path = storage::build_path(Self::CONFIG_FILE_NAME)?;
        let mut content: Vec<u8>= Vec::new();
        content.push(self.print_color.into());
        content.extend_from_slice(&self.ls_default_period
            .as_days()
            .to_le_bytes());
        std::fs::write(path, content)
            .map_err(|_| GlobalError::fs("save", "config"))?;
        Ok(())
    }

    pub fn try_set(&mut self, key: &str, value: &str) -> Result<(), GlobalError> {
        match key {
            "ls-default-period" => {
                self.ls_default_period = time::Period::from_str(value)?;
                Ok(())
            }
            "print-color" => {
                self.print_color = bool::from_str(value)
                    .map_err(|_| GlobalError::parse("bool", value.to_string(), "one of `true` or `false`"))?;
                Ok(())
            }
            _ => Err(GlobalError::config_invalid_key(key))
        }
    }
}

impl default::Default for Config {
    fn default() -> Self {
        Self {
            print_color: true,
            ls_default_period: time::Period::Weeks(2),
        }
    }
}
