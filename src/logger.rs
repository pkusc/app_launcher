use log::info;
use power_controller::Cluster;
use std::io::Write;
use std::sync::Arc;
use std::fs::File;
use std::time::Duration;
use crate::execute::PROGRESS;
pub static mut POWER :usize = 0;
pub static mut STOP: bool = false;
const THRESHOLD: usize = 1450;

pub struct PowerLogger {
    cluster: Arc<Cluster>,
}

impl PowerLogger {
    pub fn new(cluster: Arc<Cluster>)-> PowerLogger {
        PowerLogger { cluster }
    }
    fn get_power(&self) -> usize{
        self.cluster.collect_power_data(0).total_power
    }
    pub fn run_deamon(&self, parent_id: u32, output_file: String) {
        info!("the parent_id is {parent_id}");
        let mut SAMPLE_FREQ = 10000;
        let mut f = File::create(output_file).unwrap();
        loop {
            unsafe {
                if STOP {
                    break;
                }
            }
            
            let power = self.get_power();
            info!("get the power of {power}");
            unsafe {
                
                if PROGRESS > 0.0 {
                    
                    SAMPLE_FREQ = 0;
                    f.write(format!("{PROGRESS}% {power}\n").as_bytes()).unwrap();
                    POWER = power;
                }
                
            }
            use nix::{
                unistd::Pid,
                sys::signal::{self,Signal}  
            };
            #[allow(deprecated)]
            if power > THRESHOLD {
                signal::kill(Pid::from_raw(parent_id as std::os::unix::raw::pid_t)
                    , Signal::SIGUSR1).unwrap();
            }
            

            std::thread::sleep(Duration::from_millis(SAMPLE_FREQ));
        }
    }
    pub fn start_deamon(cluster: Arc<Cluster>, output_file: &str, parent_id: u32){
        info!("run the power_logger");
        let power_logger = PowerLogger::new(cluster);
        power_logger.run_deamon(parent_id, output_file.to_string());
    }
}
