use crate::StateManager;
use power_controller::Cluster;
use std::{
    time::Duration,
    thread::sleep
};
use log::*;
const DEFAULT_BLOWING_TIME: Duration = Duration::from_millis(30);
const WINDOW_SIZE: usize = 10;
const THRESHOLD: usize = 30;

pub struct Preparer<'a> {
    state_manager: &'a StateManager<'a>,
    cluster: &'a Cluster,
    blowing_time: Option<Duration>
}

impl<'a> Preparer<'a> {
    pub fn new(cluster: &'a Cluster, state_manager: &'a StateManager<'a>, blowing_time: Option<Duration>) -> Preparer<'a>{
        Preparer { 
            state_manager, 
            cluster, 
            blowing_time 
        }
    }
    pub fn fiercely_blowing(&self) {
        
        self.state_manager.set_fan_speed(100);
        sleep(match self.blowing_time {
            Some(x) => {
                x
            },
            None => {
                DEFAULT_BLOWING_TIME
            }
        });
        self.state_manager.reset();
    }
    pub fn wait_for_stability(&self) {
        let mut rec = vec![self.cluster.collect_power_data(0).total_power];
        let (mut min_index, mut max_index)  = (0, 0);
        loop {
            let x = self.cluster.collect_power_data(0).total_power;
            info!("[waiting stability]the newly read power is {}", x);
            if x > rec[max_index] {
                max_index = rec.len();
            }
            if x < rec[min_index] {
                min_index = rec.len();
            }
    
            rec.push(x);
    
            if rec.len() > WINDOW_SIZE {
                let st = rec.len() - WINDOW_SIZE;
                let ed = rec.len();
                if max_index < st {
                    max_index = st;
    
                    for i in (st + 1)..ed {
                        if rec[i] > rec[max_index] {
                            max_index = i;
                        }
                    }
                }
                if min_index < st {
                    min_index = st;
                    for i in (st + 1)..ed {
                        if rec[i] < rec[min_index] {
                            min_index = i;
                        }
                    }
                }
                warn!("the highest power is {}, the lowest is {}, the difference is {}", 
                    rec[max_index], rec[min_index], rec[max_index] - rec[max_index]);
                if rec[max_index] - rec[min_index] <= THRESHOLD {
                    break;
                }
            }
            
        };
        info!("the power variation is stable in the threshold {}", THRESHOLD);
    } 
}


