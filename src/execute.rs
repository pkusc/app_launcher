use crate::{State, StateManager};
use std::fmt::{self,Display};
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::process::{ChildStdout, Command, Stdio};
use log::info;
use serde_json::Value;

#[derive(PartialEq, Debug)]
pub struct Action {
    pub hint: String,
    tune_set: Vec<State>
}
pub struct Executor<'a> {
    notice: Vec<Action>,
    notice_index: usize,
    state_manager: &'a mut StateManager<'a>,
    executable_file: String,
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
        info!("[action]{} is acted", &self);
        for s in &self.tune_set {
            state_manager.switch_state(s.clone())
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.

        write!(f, "(hint: {}, action_set: {:?})", self.hint,self.tune_set)
    }
}
impl<'a> Executor<'a> {
    pub fn new<P:'a + AsRef<Path>>(executable_file: P, raw_action_set: &'a Value, state_manager: &'a mut StateManager<'a>) 
    -> Executor<'a> {
        let arr = raw_action_set.as_array().expect("need to input an action array");
        let notice: Vec<Action> = arr.iter()
                        .map(|c| {Action::from(c)})
                        .collect::<_>();

        Executor { 
            notice, 
            notice_index: 0, 
            state_manager, 
            executable_file: executable_file.as_ref().to_str().unwrap().to_string()
        }
        
    }
    fn get_buffer(&self) -> Result<BufReader<ChildStdout>, std::io::Error>{
        let mut child = match Command::new(&self.executable_file)
                        .arg("2>&1")
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
        info!("[execution]set buffer");
        let mut buffer = self.get_buffer().unwrap();
        info!("[execution]executable file is running");
        let mut s = String::new();
        let l = self.notice.len();
        
        loop {
            match buffer.read_line(&mut s) {
                Ok(_x) => {
                    if self.notice_index < l {
                        if s.contains(self.notice[self.notice_index].hint.as_str()) {
                            info!("[execution]hint:{} is matched", self.notice[self.notice_index].hint);
                            self.notice[self.notice_index].act(self.state_manager);
                            
                            self.notice_index += 1;
                        }
                    }
                    info!("[running] get a line");
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

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;
    #[test]
    fn test_action_generation_1() {
        let raw = r#"
        {
            "hint": "POL",
            "action": [
                {
                    "GPU_Freq": 585,
                    "Time": 5
                },
                {
                    "GPU_Freq": 675,
                    "Time": 5
                },
                {
                    "GPU_Freq": 765,
                    "Time":0
                }
            ]
        }
        "#;
        let v = serde_json::from_str(raw).unwrap();
        let a = Action::from(&v);
        assert_eq!(a, Action {
            hint: "POL".to_string(),
            tune_set: vec![
                State {
                    gpu_freq: Some(585),
                    cpu_freq: None,
                    fan_speed: None,
                    lasting_time: Some(Duration::from_millis(5)),
                },
                State {
                    gpu_freq: Some(675),
                    cpu_freq: None,
                    fan_speed: None,
                    lasting_time: Some(Duration::from_millis(5))
                },
                State {
                    gpu_freq: Some(765),
                    cpu_freq: None,
                    fan_speed: None,
                    lasting_time: Some(Duration::from_millis(0)),
                }
            ]
        });

    }

    #[test]
    fn test_display() {
        let raw = r#"
        {
            "hint": "POL",
            "action": [
                {
                    "GPU_Freq": 585,
                    "Time": 5
                },
                {
                    "GPU_Freq": 675,
                    "Time": 5
                },
                {
                    "GPU_Freq": 765,
                    "Time":0
                }
            ]
        }
        "#;
        let v = serde_json::from_str(raw).unwrap();
        let a = Action::from(&v);
        let s = format!("{}",a);
        assert_eq!("(hint: POL, action_set: [State{GPU_Freq: 585MHz,Lasting_time: 5ms,}, State{GPU_Freq: 675MHz,Lasting_time: 5ms,}, State{GPU_Freq: 765MHz,Lasting_time: 0ns,}])"
            , s);
    }
}