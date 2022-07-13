use std::{path::Path, fs::File, io::BufReader, time::Duration, process::Output};

use app_lanucher::{StateManager, State, Preparer, Executor};
use clap::Parser;
use power_controller::Cluster;
use serde_json::Value;

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
    /// save the log for application
    #[clap(short = 'o', long, value_parser, default_value = "./result.log")]
    log_file: String,
    /// only do preparation
    #[clap(short = 'p', long, value_parser, default_value = "false")]
    only_prepare: bool,
    /// blowing time in milisecond
    #[clap(short = 'b', long, value_parser, default_value = "100")]
    blowing_time: u64 
}


fn print_args(a: &Args) {
    println!("blowing_time is {}", a.blowing_time);
    println!("application file is {}", Path::new(&a.application_file).exists());
    println!("cluster file is {}", Path::new(&a.cluster_file).exists());
}
fn extract_application(file_name: &str) -> Value {
    let file : File = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}
fn do_preparation(p: &Preparer) {
    p.fiercely_blowing();
    p.wait_for_stability();
}

fn do_executation(e: &mut Executor) {
    e.run();
}
fn main() {
    let args = Args::parse();

    let cluster = Cluster::from_file(Path::new(&args.cluster_file));
    let app_info = extract_application(args.application_file.as_str());


    let mut state_manager = StateManager::new(&cluster, State::from(&app_info["start_state"]));
    
    let preparer = Preparer::new(&cluster, &state_manager, Some(Duration::from_millis(args.blowing_time)));

    do_preparation(&preparer);

    if args.only_prepare {
        return;
    }

    let application_path = app_info["application_path"].as_str().unwrap();
    let result_path = args.log_file.as_str();

    let mut executor = Executor::new(application_path, result_path, 
            &app_info["strategy"], &mut state_manager);

    do_executation(&mut executor);
}