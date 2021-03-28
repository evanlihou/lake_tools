use config::{ConfigError, Config, File, Environment};

#[derive(Debug, Deserialize)]
pub struct DataCollectionSettings {
    pub samples_per_collection: u8,
    pub calibration_microsec: f64,
    pub simulate_sensor: bool,
    pub millisec_between_readings: u64,
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