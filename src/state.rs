use log::info;
use power_controller::pwrctl::Command;
use power_controller::Cluster;
use serde_json::Value;
use num::*;
use std::{
    thread::{
        sleep
    },
    time::{
        Duration
    }, 
    fmt::{
        self,
        Display, Debug
    }
};

const DEFAULT_LASTING_TIME: Duration = Duration::from_millis(1);

#[derive(Clone, PartialEq)]
pub struct State {
    pub(super) cpu_freq: Option<usize>,
    pub(super) gpu_freq: Option<usize>,
    pub(super) fan_speed: Option<usize>,
    pub(super) lasting_time: Option<Duration>,
}

pub struct StateManager<'a> {
    current_state: State,
    cluster: &'a Cluster
}

impl From<&Value> for State {
    fn from(c: &Value) -> Self {
        let obj = c.as_object().expect("the value in action must be an object");
        State { 
            cpu_freq: match obj.get("CPU_Freq") {
                None => {
                    None
                },
                Some(x) => {
                    Some(x.as_u64().expect("need a number").to_usize().unwrap())
                }
            }, 
            gpu_freq: match obj.get("GPU_Freq") {
                None => {
                    None
                },
                Some(x) => {
                    Some(x.as_u64().expect("need a number").to_usize().unwrap())
                }
            }, 
            fan_speed: 
            match obj.get("Fan_Speed") {
                None => {
                    None
                },
                Some(x) => {
                    Some(x.as_u64().expect("need a number[1-100]").to_usize().unwrap())
                }
            }, 
            lasting_time: 
            match obj.get("Time") {
                None => {
                    None
                },
                Some(x) => {
                    Some(Duration::from_millis(x.as_u64().expect("need a number of milisecond")))
                }
            } 
        }
    }
}
impl State {
    // maybe there are some places can be empty
    pub fn new(cpu_freq: Option<usize>, gpu_freq: Option<usize>, fan_speed: Option<usize>, lasting_time: Option<Duration>) -> State {
        State {
            cpu_freq: cpu_freq,
            gpu_freq: gpu_freq,
            fan_speed: fan_speed,
            lasting_time: lasting_time
        }
    }
    pub fn all_filled(&self) ->bool {
        match self.cpu_freq {
            None => false,
            Some(_) => {
                match self.gpu_freq {
                    None => false,
                    Some(_) => {
                        match self.fan_speed {
                            None => false,
                            Some(_) => true
                        } 
                    }
                }
            }
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cpu_freq = match self.cpu_freq {
            Some(x) => {
                format!("CPU_Freq: {}MHz,",x)
            },
            None => {
                String::new()
            }
        };
        let gpu_freq = match self.gpu_freq {
            Some(x) => {
                format!("GPU_Freq: {}MHz,", x)
            },
            None => {
                String::new()
            }
        };
        let fan_speed = match self.fan_speed {
            Some(x) => {
                format!("Fan_Speed: {}%,", x)
            },
            None => {
                String::new()
            }
        };
        let lasting_time = match self.lasting_time {
            Some(x) => {
                format!("Lasting_time: {:?},", x)
            },
            None => {
                String::new()
            }
        };
        write!(f, "State{{{}{}{}{}}}", cpu_freq, gpu_freq, fan_speed, lasting_time)
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cpu_freq = match self.cpu_freq {
            Some(x) => {
                format!("CPU_Freq: {}MHz,",x)
            },
            None => {
                String::new()
            }
        };
        let gpu_freq = match self.gpu_freq {
            Some(x) => {
                format!("GPU_Freq: {}MHz,", x)
            },
            None => {
                String::new()
            }
        };
        let fan_speed = match self.fan_speed {
            Some(x) => {
                format!("Fan_Speed: {}%,", x)
            },
            None => {
                String::new()
            }
        };
        let lasting_time = match self.lasting_time {
            Some(x) => {
                format!("Lasting_time: {:?},", x)
            },
            None => {
                String::new()
            }
        };
        write!(f, "State{{{}{}{}{}}}", cpu_freq, gpu_freq, fan_speed, lasting_time)
    }
}
impl StateManager<'_> {
    pub fn new(cluster: &Cluster, state: State) -> StateManager {
        StateManager { 
            current_state: state, 
            cluster,
        }
    }
    pub fn set_cpu_freq(&self, target_freq: usize) {
        info!("[state switch]change cpu frequency to {}MHz",target_freq);
        let s = format!("SETFREQ CPU {freq}", freq = target_freq);
        let command = Command::parse(self.cluster, &s);
        match command {
            Ok(c) => {
                self.cluster.run_command(&c);
            },
            Err(msg) => {
                println!("{}", msg);

            }
        };
    } 
    pub fn set_gpu_freq(&self, target_freq: usize) {
        info!("[state switch]change gpu frequency to {}MHz",target_freq);
        let s = format!("SETFREQ GPU {freq}", freq = target_freq);
        let command = Command::parse(self.cluster, &s);
        match command {
            Ok(c) => {
                self.cluster.run_command(&c);
            },
            Err(msg) => {
                println!("{}", msg);

            }
        };

    }
    pub fn set_fan_speed(&self, target_speed: usize) {
        info!("[state switch]change fan speed to {}%",target_speed);
        let s = format!("SETSPEED FAN {speed}", speed = target_speed);
        let command = Command::parse(self.cluster, &s);
        match command {
            Ok(c) => {
                self.cluster.run_command(&c);
            },
            Err(msg) => {
                println!("{}", msg);

            }
        };
    }
    pub fn switch_state(&mut self, mut target_state: State) {


        match target_state.cpu_freq {
            Some(x) => {
                self.set_cpu_freq(x);
                self.current_state.cpu_freq = target_state.cpu_freq.take();
            },
            None =>{}
        };

        match target_state.gpu_freq {
            Some(x) => {
                self.set_gpu_freq(x);
                self.current_state.gpu_freq = target_state.gpu_freq.take();
            },
            None => {}
        };

        match target_state.fan_speed {
            Some(x) => {
                self.set_fan_speed(x);
                self.current_state.fan_speed = target_state.fan_speed.take();
            },
            None => {}
        }

        sleep(match target_state.lasting_time {
            Some(x) => {
                x
            },
            None => {
                DEFAULT_LASTING_TIME
            }
        });

    }

    pub fn reset(&self) {
        let cpu_freq = self.current_state.cpu_freq.unwrap();
        let gpu_freq = self.current_state.gpu_freq.unwrap();
        let fan_speed = self.current_state.fan_speed.unwrap();
        self.set_cpu_freq(cpu_freq);
        self.set_gpu_freq(gpu_freq);
        self.set_fan_speed(fan_speed);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // some tests, the interface may can't handle some crushed input, so please don't do that
    #[test]
    fn test_complete_state_from_value() {
        let testv  = r#"
        {
            "GPU_Freq": 390,
            "CPU_Freq": 1000,
            "Fan_Speed": 40,
            "Time": 0
        
        }
        "#;
        let v = serde_json::from_str(testv).unwrap();
        let s = State::from(&v);
        assert_eq!(s, State {
            gpu_freq: Some(390),
            cpu_freq: Some(1000),
            fan_speed: Some(40),
            lasting_time: Some(Duration::from_millis(0))
        });

    }
    #[test]
    fn test_state_from_partial_value() {
        let testv  = r#"
        {
            "GPU_Freq": 765
        }
        "#;
        let v = serde_json::from_str(testv).unwrap();
        let s = State::from(&v);
        assert_eq!(s, State {
            gpu_freq: Some(765),
            cpu_freq: None,
            fan_speed: None,
            lasting_time: None
        });

    }
    #[test]
    fn test_display() {
        let testv  = r#"
        {
            "GPU_Freq": 765
        }
        "#;
        let v = serde_json::from_str(testv).unwrap();
        let s = State::from(&v);
        let res = format!("{}",s);
        assert_eq!("GPU_Freq: 765MHz,", res);
    }
}