use crate::core::types::{Bool, Date, TaskOverflowPolicy, TaskSchedulingOrder, TimeRange};
use crate::errors::Error;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub trait ConfigItem<T> {
    fn get_value(&self) -> &T;
    fn set_value(&mut self, new_value: &str) -> Result<(), Error>;
    fn description(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StartDateConfigItem {
    pub value: Option<NaiveDate>,
    pub description: String,
}
impl ConfigItem<Option<NaiveDate>> for StartDateConfigItem {
    fn get_value(&self) -> &Option<NaiveDate> {
        &self.value
    }
    fn set_value(&mut self, new_value: &str) -> Result<(), Error> {
        if new_value.trim().is_empty() {
            self.value = None;
            return Ok(());
        }
        let parsed = Date::try_from_str(new_value)?;
        self.value = Some(parsed.0);
        Ok(())
    }
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeConfigItem {
    pub value: TimeRange,
    pub description: String,
}
impl ConfigItem<TimeRange> for RangeConfigItem {
    fn get_value(&self) -> &TimeRange {
        &self.value
    }
    fn set_value(&mut self, new_value: &str) -> Result<(), Error> {
        Ok(self.value = TimeRange::try_from_str(new_value)?)
    }
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOverflowPolicyConfigItem {
    pub value: TaskOverflowPolicy,
    pub description: String,
}
impl ConfigItem<TaskOverflowPolicy> for TaskOverflowPolicyConfigItem {
    fn get_value(&self) -> &TaskOverflowPolicy {
        &self.value
    }
    fn set_value(&mut self, new_value: &str) -> Result<(), Error> {
        Ok(self.value = TaskOverflowPolicy::try_from(new_value)?)
    }
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedulingOrderConfigItem {
    pub value: TaskSchedulingOrder,
    pub description: String,
}
impl ConfigItem<TaskSchedulingOrder> for TaskSchedulingOrderConfigItem {
    fn get_value(&self) -> &TaskSchedulingOrder {
        &self.value
    }
    fn set_value(&mut self, new_value: &str) -> Result<(), Error> {
        Ok(self.value = TaskSchedulingOrder::try_from(new_value)?)
    }
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLoggingConfigItem {
    pub value: Bool,
    pub description: String,
}

impl Default for FileLoggingConfigItem {
    fn default() -> Self {
        Self {
            value: Bool(true),
            description: "Enable writing log messages to file.".into(),
        }
    }
}

impl ConfigItem<Bool> for FileLoggingConfigItem {
    fn get_value(&self) -> &Bool {
        &self.value
    }
    fn set_value(&mut self, new_value: &str) -> Result<(), Error> {
        Ok(self.value = Bool::try_from_str(new_value)?)
    }
    fn description(&self) -> &str {
        &self.description
    }
}
