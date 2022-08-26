use log::info;
use power_controller::Cluster;
use std::sync::Arc;
pub static mut POWER :usize = 0;

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
    pub fn run_deamon(&self, parent_id: u32) {
        info!("the parent_id is {parent_id}");
        loop {
            let power = self.get_power();
            info!("get the power of {power}");
            unsafe {
                POWER = power;
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
        }
    }

}