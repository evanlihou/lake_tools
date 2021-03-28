use config::{ConfigError, Config, File, Environment};

/// Settings managing the data collection process
#[derive(Debug, Deserialize)]
pub struct DataCollectionSettings {
    /// How many samples to take from the sensor for each data collection. These samples will be
    /// averaged together, and that average will be used as the data point
    pub samples_per_collection: u8,
    /// A calibration value to account for any constant delay, such as long wires and propagation delay
    pub calibration_microsec: f64,
    /// Whether to simulate a sensor or use an actual ultrasonic sensor attached via GPIO
    pub simulate_sensor: bool,
    /// Number of milliseconds to wait in between taking data collections
    pub millisec_between_readings: u64,
    /// Filename of the SQLite database to use to store data
    pub db_filename: String
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub data_collection: DataCollectionSettings
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();
        settings.merge(File::with_name("Settings")).expect("Could not merge in settings file");
        settings.merge(Environment::with_prefix("app")).expect("Could not merge in environment variables");

        settings.try_into()
    }
}