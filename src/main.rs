use std::{path::Path, fs::File, io::BufReader, time::Duration, process::exit};

use app_lanucher::{StateManager, State, Preparer, Executor};
use clap::Parser;
use log::{info, LevelFilter};
use power_controller::Cluster;
use serde_json::Value;
use simplelog::*;

/// launcher for specific HPC application
/// write power adjustment strategy in milisecond grain
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// the cluster file for running application
    #[clap(short = 'c', long, value_parser, default_value = "./config-example/pkusc.json")]
    cluster_file: String,
    /// the application file with application name and running strategy
    #[clap(short = 'a', long, value_parser, default_value = "./config-example/hpl.json")]
    application_file: String,
    /// only do preparation
    #[clap(short = 'p', long, value_parser, default_value = "false")]
    only_prepare: bool,
    /// blowing time in milisecond
    #[clap(short = 'b', long, value_parser, default_value = "10000")]
    blowing_time: u64, 
    /// set debug level
    #[clap(long = "debug", value_parser, default_value = "false")]
    debug_level: bool,
    /// only check setting
    #[clap(long = "sc", value_parser, default_value = "false")]
    setting_check: bool
}


fn print_args_for_debug(a: &Args) {
    info!("blowing_time is {}", a.blowing_time);
    info!("application file is: {}, does it exist? {}", &a.application_file, Path::new(&a.application_file).exists());
    info!("cluster file is: {}, does it exist? {}", &a.application_file, Path::new(&a.cluster_file).exists());
    let app_info = extract_application(a.application_file.as_str());

    info!("the application to launch is {:?}", app_info["application_path"]);
    info!("the start state is {:?}", app_info["start_state"]);
    info!("the strategy is {:?}", app_info["strategy"]);

}
fn extract_application(file_name: &str) -> Value {
    let file : File = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}
fn do_preparation(p: &Preparer) {
    info!("preparedness begins");
    p.fiercely_blowing();
    info!("blowing ends");
    p.wait_for_stability();
    info!("power is stable");
    info!("preparedness ends");
}

fn do_executation(e: &mut Executor) {
    e.run();
}
fn main() {
    let args = Args::parse();
    
    if args.debug_level {
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("debug.log").unwrap())
            ]
        ).unwrap();
        
    }

    if args.setting_check {
        print_args_for_debug(&args);
        exit(1);
    }
    let cluster = Cluster::from_file(Path::new(&args.cluster_file));
    let app_info = extract_application(args.application_file.as_str());

    let mut state_manager = StateManager::new(&cluster, State::from(&app_info["start_state"]));
    
    let preparer = Preparer::new(&cluster, &state_manager, Some(Duration::from_millis(args.blowing_time)));


    do_preparation(&preparer);

    if args.only_prepare {
        exit(1);
    }

    let application_path = app_info["application_path"].as_str().unwrap();

    let mut executor = Executor::new(application_path, 
            &app_info["strategy"], &mut state_manager);

    do_executation(&mut executor);
}