extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate ctrlc;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
mod settings;
mod data_collection;
use data_collection::run_data_collection;

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(2);

fn main() {
    if cfg!(debug_assertions) {
        println!("Running as a debug configuration");
    }

    let (data_collection_tx, data_collection_rx) = channel();

    ctrlc::set_handler(move || {
        data_collection_tx.send(ThreadMessage::Terminate).expect("Data collection thread receiver went away");
    }).expect("Error setting Ctrl-C handler");

    thread::Builder::new().name("DataCollectionThread".to_string()).spawn(move || {
        if let Err(error) = run_data_collection(data_collection_rx) {
            println!("[ERR] Data collection panicked: {:?}", error);
        }
        GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
    }).expect("Data collection thread panicked?");

    thread::Builder::new().name("WebServerThread".to_string()).spawn(move || {
        println!("Hello from the web server thread.");
        thread::sleep(Duration::from_secs(2));
        GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
    }).expect("Web server thread panicked?");

    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        thread::sleep(Duration::from_millis(100));
    }

}

pub enum ThreadMessage {
    Terminate
}