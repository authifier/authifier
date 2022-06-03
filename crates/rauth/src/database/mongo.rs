use super::definition::AbstractDatabase;

pub struct MongoDb(mongodb::Database);

impl AbstractDatabase for MongoDb {}
