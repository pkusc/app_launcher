use crate::{State, StateManager};
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::process::{ChildStdout, Command, Stdio};
use log::info;
use serde_json::Value;


pub struct Action {
    pub hint: String,
    tune_set: Vec<State>
}
pub struct Executor<'a> {
    notice: Vec<Action>,
    notice_index: usize,
    state_manager: &'a mut StateManager<'a>,
    executable_file: String,
    result_file:String
}   

impl From<&Value> for Action {
    fn from(raw_data: &Value) -> Self{
        let obj = raw_data.as_object()
            .expect("can not convert the value to an object in action initialization");
        Action { 
            hint: obj["hint"].as_str().expect("hint is not a string").to_string(), 
            tune_set: obj["action"]
                        .as_array()
                        .expect("action must be an array of state")
                        .iter()
                        .map(|c| {State::from(c)})
                        .collect::<_>()  
        }
    }
}
impl Action {
    
    pub fn act(&self, state_manager: &mut StateManager) {
        for s in &self.tune_set {
            state_manager.switch_state(s.clone())
        }
    }
}


impl<'a> Executor<'a> {
    pub fn new<P:'a + AsRef<Path>>(executable_file: P, result_file: P, raw_action_set: &'a Value, state_manager: &'a mut StateManager<'a>) 
    -> Executor<'a> {
        let arr = raw_action_set.as_array().expect("need to input an action array");
        let notice: Vec<Action> = arr.iter()
                        .map(|c| {Action::from(c)})
                        .collect::<_>();

        Executor { 
            notice, 
            notice_index: 0, 
            state_manager, 
            executable_file: executable_file.as_ref().to_str().unwrap().to_string(),
            result_file: result_file.as_ref().to_str().unwrap().to_string()
        }
        
    }
    fn get_buffer(&self) -> Result<BufReader<ChildStdout>, std::io::Error>{
        let mut child = match Command::new(&self.executable_file)
                        .stdout(Stdio::piped())
                        .spawn()
                        {
                            Ok(c) => c,
                            Err(e) => {
                                info!("{}",e);
                                return Err(e);
                            }
                        };
        let stdout = child.stdout.take().unwrap();
        Ok(BufReader::new(stdout))

    }
    pub fn run(&mut self) {
        let mut buffer = self.get_buffer().unwrap();
        let mut s = String::new();
        let l = self.notice.len();
        
        loop {
            match buffer.read_line(&mut s) {
                Ok(x) => {
                    if self.notice_index < l {
                        if s.contains(self.notice[self.notice_index].hint.as_str()) {
                            self.notice[self.notice_index].act(self.state_manager);
                            self.notice_index += 1;
                        }
                    }
                    print!("{}",s); 
                },
                Err(e) => {
                    print!("{}",e);
                    break;
                }
            };
        }
    }
}