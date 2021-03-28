use crate::settings::{Settings, DataCollectionSettings};
use std::thread;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::{Duration};
use time::OffsetDateTime;
use std::vec::Vec;
use rusqlite::{params, Connection, Result};
use crate::ThreadMessage;

#[derive(Debug)]
pub enum DataCollectionError {
    /// The thread encountered a failure error that would have otherwise caused a panic
    FatalError(String),
    /// There was an error initializing or querying the database
    DatabaseError(rusqlite::Error),
}

pub fn run_data_collection(mpsc_receiver: Receiver<ThreadMessage>) -> Result<(), DataCollectionError> {
    let settings = Settings::new().map_err(|e| DataCollectionError::FatalError(e.to_string()))?.data_collection;

    let conn = Connection::open(&settings.db_filename).map_err(|e| DataCollectionError::DatabaseError(e))?;
    
    println!("Starting data collection...");

    conn.execute("CREATE TABLE IF NOT EXISTS collections (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp DATETIME NOT NULL DEFAULT(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
        reading REAL NOT NULL);", params![]).map_err(|e| DataCollectionError::DatabaseError(e))?;

    // Create a simple streaming channel
    let (tx, rx) = channel();

    // Make another thread to handle the queue and handle each reading
    thread::spawn(move || {
        loop {
            let (reading, time): (f64, Option<OffsetDateTime>) = rx.recv().expect("Error happened");
            if reading == f64::MAX { break ;}
            if let Err(e) = conn.execute("INSERT INTO collections (timestamp, reading) VALUES (?, ?)", params![time, reading]) {
                println!("[ERR] Saving to DB failed: {:?}", e);
                break;
            }
        }
    });

    let mut iters: u16 = 100;
    loop {
        match take_collection(&settings) {
            Err(e) => return Err(e),
            Ok(reading) => tx.send((reading, Some(OffsetDateTime::now_utc()))).map_err(|e| DataCollectionError::FatalError(e.to_string()))?
        }

        iters -= 1;
        if iters <= 0 { 
            tx.send((f64::MAX, None)).expect("Done command to DB write queue failed.");
            break;
        }

        match mpsc_receiver.try_recv() {
            Ok(message) => match message {
                ThreadMessage::Terminate => {
                    println!("Terminating data collection thread...");
                    tx.send((f64::MAX, None)).expect("Done command to DB write queue failed.");
                    break;
                }
            },
            Err(error) => match error {
                TryRecvError::Empty => (),
                TryRecvError::Disconnected => {
                    eprintln!("[ERR] Data Collection Thread Message sender went away")
                }
            }
        }
        
        thread::sleep(Duration::from_millis(settings.millisec_between_readings));
    }

    Ok(())
}

fn take_collection(settings: &DataCollectionSettings) -> Result<f64, DataCollectionError> {
    let num_samples = settings.samples_per_collection;
    let simulate_sensor = settings.simulate_sensor;
    if !simulate_sensor {
        return Err(DataCollectionError::FatalError("Working with real sensors not yet supported".to_string()));
    }

    let mut sample_results: Vec<f64> = Vec::with_capacity(num_samples as usize);
    
    for _ in 0..num_samples {
        // Take sample
        sample_results.push(4.07);
    }

    let average = sample_results.iter().sum::<f64>() / sample_results.len() as f64;

    Ok(average - (settings.calibration_microsec / 1000.0))
}