pub mod models;
#[cfg(test)]
mod tests;

use std::fs;
use std::ops::Index;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter as EnumIterDerive, EnumString};

use crate::config::models::{
    ConfigItem, FileLoggingConfigItem, RangeConfigItem, StartDateConfigItem,
    TaskOverflowPolicyConfigItem, TaskSchedulingOrderConfigItem,
};
use crate::core::types::{TaskOverflowPolicy, TaskSchedulingOrder, TimeRange};
use crate::errors::{Error, Result};
use crate::extensions::enums::valid_csv;
use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIterDerive, EnumString, Display, AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfigKey {
    Range,
    TaskOverflowPolicy,
    TaskSchedulingOrder,
    ScheduleStartDate,
    FileLoggingEnabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub range: RangeConfigItem,
    pub task_overflow_policy: TaskOverflowPolicyConfigItem,
    pub task_scheduling_order: TaskSchedulingOrderConfigItem,
    #[serde(default)]
    pub schedule_start_date: StartDateConfigItem,
    #[serde(default)]
    pub file_logging_enabled: FileLoggingConfigItem,
}

#[derive(Debug, Clone)]
pub struct Config {
    path: PathBuf,
    data: ConfigFile,
    pub last_change: Option<(String, String, String)>,
}

#[derive(Debug, Clone)]
pub struct ConfigRows(Vec<(String, String, String)>);

impl ConfigRows {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &(String, String, String)> {
        self.0.iter()
    }
    pub fn get(&self, index: usize) -> Option<&(String, String, String)> {
        self.0.get(index)
    }
}
impl Index<usize> for ConfigRows {
    type Output = (String, String, String);
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Config {
    pub fn load_default() -> Result<Self> {
        Self::load_from("config.json")
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(Error::Parse(format!(
                "Configuration file '{}' not found.",
                path.display()
            )));
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| Error::Parse(format!("Failed to read {}: {}", path.display(), e)))?;
        let data: ConfigFile = serde_json::from_str(&text)
            .map_err(|e| Error::Parse(format!("Invalid JSON in '{}': {}", path.display(), e)))?;
        Ok(Self {
            path,
            data,
            last_change: None,
        })
    }

    pub fn view(&self) -> &ConfigFile {
        &self.data
    }

    pub fn range(&self) -> &TimeRange {
        self.data.range.get_value()
    }
    pub fn task_overflow_policy(&self) -> &TaskOverflowPolicy {
        self.data.task_overflow_policy.get_value()
    }
    pub fn task_scheduling_order(&self) -> &TaskSchedulingOrder {
        self.data.task_scheduling_order.get_value()
    }
    pub fn schedule_start_date(&self) -> &Option<NaiveDate> {
        self.data.schedule_start_date.get_value()
    }
    pub fn file_logging_enabled(&self) -> bool {
        self.data.file_logging_enabled.get_value().0
    }

    pub fn rows(&self) -> ConfigRows {
        let mut rows = Vec::new();
        for key in ConfigKey::iter() {
            match key {
                ConfigKey::Range => rows.push((
                    key.to_string(),
                    self.data.range.description().to_string(),
                    self.data.range.get_value().to_string(),
                )),
                ConfigKey::TaskOverflowPolicy => rows.push((
                    key.to_string(),
                    self.data.task_overflow_policy.description().to_string(),
                    self.data.task_overflow_policy.get_value().to_string(),
                )),
                ConfigKey::TaskSchedulingOrder => rows.push((
                    key.to_string(),
                    self.data.task_scheduling_order.description().to_string(),
                    self.data.task_scheduling_order.get_value().to_string(),
                )),
                ConfigKey::ScheduleStartDate => rows.push((
                    key.to_string(),
                    self.data.schedule_start_date.description().to_string(),
                    self.data
                        .schedule_start_date
                        .get_value()
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                )),
                ConfigKey::FileLoggingEnabled => rows.push((
                    key.to_string(),
                    self.data.file_logging_enabled.description().to_string(),
                    self.data.file_logging_enabled.get_value().to_string(),
                )),
            }
        }
        ConfigRows(rows)
    }

    pub fn set_by_index(&mut self, index: usize, new_value: &str) -> Result<()> {
        let key = ConfigKey::iter()
            .nth(index)
            .ok_or_else(|| Error::Parse(format!("Invalid ID: {index}")))?;
        self.set_key(key, new_value)
    }

    pub fn set_key(&mut self, key: ConfigKey, new_value: &str) -> Result<()> {
        let (old, res) = match key {
            ConfigKey::Range => {
                let old = self.data.range.get_value().to_string();
                let res = self.edit(|cfg| cfg.range.set_value(new_value));
                (old, res)
            }
            ConfigKey::TaskOverflowPolicy => {
                let old = self.data.task_overflow_policy.get_value().to_string();
                let res = self.edit(|cfg| cfg.task_overflow_policy.set_value(new_value));
                (old, res)
            }
            ConfigKey::TaskSchedulingOrder => {
                let old = self.data.task_scheduling_order.get_value().to_string();
                let res = self.edit(|cfg| cfg.task_scheduling_order.set_value(new_value));
                (old, res)
            }
            ConfigKey::ScheduleStartDate => {
                let old = self
                    .data
                    .schedule_start_date
                    .get_value()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let res = self.edit(|cfg| cfg.schedule_start_date.set_value(new_value));
                (old, res)
            }
            ConfigKey::FileLoggingEnabled => {
                let old = self.data.file_logging_enabled.get_value().to_string();
                let res = self.edit(|cfg| cfg.file_logging_enabled.set_value(new_value));
                (old, res)
            }
        };

        if res.is_ok() {
            let new_val = match key {
                ConfigKey::Range => self.data.range.get_value().to_string(),
                ConfigKey::TaskOverflowPolicy => {
                    self.data.task_overflow_policy.get_value().to_string()
                }
                ConfigKey::TaskSchedulingOrder => {
                    self.data.task_scheduling_order.get_value().to_string()
                }
                ConfigKey::ScheduleStartDate => self
                    .data
                    .schedule_start_date
                    .get_value()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                ConfigKey::FileLoggingEnabled => {
                    self.data.file_logging_enabled.get_value().to_string()
                }
            };
            // stash for caller to log. We store last change for external logging.
            self.last_change = Some((key.to_string(), old, new_val));
        }

        res
    }

    pub fn take_last_change(&mut self) -> Option<(String, String, String)> {
        self.last_change.take()
    }

    pub fn set(&mut self, key_str: &str, new_value: &str) -> Result<()> {
        use std::str::FromStr;
        let key = ConfigKey::from_str(key_str).map_err(|_| {
            Error::Parse(format!(
                "Unknown configuration key '{}'. Valid keys: {}",
                key_str,
                valid_csv::<ConfigKey>()
            ))
        })?;
        self.set_key(key, new_value)
    }

    pub fn set_many<I, K, V>(&mut self, pairs: I) -> Result<()>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.edit(|cfg| {
            for (k, v) in pairs {
                use std::str::FromStr;
                let key = ConfigKey::from_str(k.as_ref()).map_err(|_| {
                    Error::Parse(format!(
                        "Unknown configuration key '{}'. Valid keys: {}",
                        k.as_ref(),
                        valid_csv::<ConfigKey>()
                    ))
                })?;

                match key {
                    ConfigKey::Range => cfg.range.set_value(v.as_ref())?,
                    ConfigKey::TaskOverflowPolicy => {
                        cfg.task_overflow_policy.set_value(v.as_ref())?
                    }
                    ConfigKey::TaskSchedulingOrder => {
                        cfg.task_scheduling_order.set_value(v.as_ref())?
                    }
                    ConfigKey::ScheduleStartDate => {
                        cfg.schedule_start_date.set_value(v.as_ref())?
                    }
                    ConfigKey::FileLoggingEnabled => {
                        cfg.file_logging_enabled.set_value(v.as_ref())?
                    }
                }
            }
            Ok(())
        })
    }

    fn edit<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ConfigFile) -> Result<()>,
    {
        f(&mut self.data)?;
        self.save()
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)
            .map_err(|e| Error::Parse(format!("Failed to encode config: {}", e)))?;
        fs::write(&self.path, json)
            .map_err(|e| Error::Parse(format!("Failed to write {}: {}", self.path.display(), e)))
    }
}
