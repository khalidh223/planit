use crate::errors::Result;
use crate::prompter::models::{Flow, FlowCtrl};
use crate::prompter::prompter::Prompter;
use std::cell::Cell;
use std::io::Cursor;
use std::rc::Rc;

struct ScriptFlow {
    renders: Rc<Cell<u32>>,
    inputs: Rc<Cell<u32>>,
    script: Vec<FlowCtrl>,
}

impl ScriptFlow {
    fn new(renders: Rc<Cell<u32>>, inputs: Rc<Cell<u32>>, script: Vec<FlowCtrl>) -> Self {
        Self {
            renders,
            inputs,
            script,
        }
    }
}

impl Flow for ScriptFlow {
    fn render(&mut self) -> Result<()> {
        self.renders.set(self.renders.get() + 1);
        Ok(())
    }

    fn handle_input(&mut self, _: &str) -> Result<FlowCtrl> {
        self.inputs.set(self.inputs.get() + 1);
        let next = self.script.remove(0);
        Ok(next)
    }
}

#[test]
fn prompter_finishes_on_flow_finish() {
    let p = Prompter::new();
    let renders = Rc::new(Cell::new(0));
    let inputs = Rc::new(Cell::new(0));
    let flow = ScriptFlow::new(renders.clone(), inputs.clone(), vec![FlowCtrl::Finish]);
    let reader = Cursor::new(b"line\n");

    p.run_with_reader(flow, false, reader).unwrap();

    assert_eq!(renders.get(), 1);
    assert_eq!(inputs.get(), 1);
}

#[test]
fn prompter_handles_continue_then_finish() {
    let p = Prompter::new();
    let renders = Rc::new(Cell::new(0));
    let inputs = Rc::new(Cell::new(0));
    let flow = ScriptFlow::new(
        renders.clone(),
        inputs.clone(),
        vec![FlowCtrl::Continue, FlowCtrl::Finish],
    );
    let reader = Cursor::new(b"first\nsecond\n");

    p.run_with_reader(flow, false, reader).unwrap();

    assert_eq!(renders.get(), 2);
    assert_eq!(inputs.get(), 2);
}

#[test]
fn prompter_exits_on_explicit_exit_input() {
    let p = Prompter::new();
    let renders = Rc::new(Cell::new(0));
    let inputs = Rc::new(Cell::new(0));
    let flow = ScriptFlow::new(renders.clone(), inputs.clone(), vec![FlowCtrl::Finish]);
    let reader = Cursor::new(b"exit\n");

    p.run_with_reader(flow, true, reader).unwrap();

    assert_eq!(renders.get(), 1);
    assert_eq!(inputs.get(), 0);
}

#[test]
fn prompter_exits_on_eof() {
    let p = Prompter::new();
    let renders = Rc::new(Cell::new(0));
    let inputs = Rc::new(Cell::new(0));
    let flow = ScriptFlow::new(renders.clone(), inputs.clone(), vec![FlowCtrl::Finish]);
    let reader = Cursor::new(b"");

    p.run_with_reader(flow, false, reader).unwrap();

    assert_eq!(renders.get(), 1);
    assert_eq!(inputs.get(), 0);
}
