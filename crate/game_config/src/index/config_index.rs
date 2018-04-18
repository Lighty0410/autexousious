use std::collections::HashMap;

use object_config::ObjectType;

use index::config_record::ConfigRecord;

/// Index of all configuration records.
#[derive(Debug, Default, PartialEq)]
pub struct ConfigIndex {
    /// List of objects in the assets directory.
    pub objects: HashMap<ObjectType, Vec<ConfigRecord>>,
}
