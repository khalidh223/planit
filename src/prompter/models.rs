use crate::errors::Result;

pub enum FlowCtrl {
    Continue,
    Finish,
    Abort,
}

pub trait Flow {
    fn render(&mut self) -> Result<()>;
    fn handle_input(&mut self, input: &str) -> Result<FlowCtrl>;
}

#[derive(Debug, Clone)]
pub enum ConfigState {
    ShowTable,   // show the config table and ask Y/N
    SelectId,    // ask for ID
    ShowCurrent, // show desc/current
    AskNewValue, // prompt for new value
    ApplyChange, // write to config
    Done,        // end
}
