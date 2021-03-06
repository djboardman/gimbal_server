
use crate::lang::internal;


#[derive(Debug)]
pub enum DatabaseChange {
  MockDb(DatabaseConfig)
, SqlDb(Vec<String>)
}

impl DatabaseChange {
  pub fn to_string(&self) -> String {
    match self {
      DatabaseChange::MockDb(_) => "I'm just a mock".to_string()
    , DatabaseChange::SqlDb(s) => s.join("\n")
    }
  }

  pub fn commands(&self) -> &Vec<String> {
    match self {
      DatabaseChange::SqlDb(s) => s
    , _ => unreachable!()
    }
  }
}

#[derive(Debug)]
pub enum DatabaseConfig {
  MockDb(MockDbConfig)
, Postgres(String)
}



#[derive(Debug)]
pub struct MockDbConfig {
  pub tables: Vec<Table>
}


#[derive(Debug)]
pub struct Database {
  tables: Vec<Table>
}

impl Database {
  pub fn new(tables: Vec<Table>) -> Database {
    Database{ tables }
  }

  pub fn tables(&self) -> &Vec<Table> {
    &self.tables
  }
}

#[derive(Debug)]
pub struct Table {
  schema: String
, name: String
, columns: Vec<Column>
}


impl Table {
  pub fn new(schema: &str, name: &str, columns: Vec<Column>) -> Table {
    Table{ schema: schema.to_string(), name: name.to_string(), columns }
  }

  pub fn name(&self) -> String {
    self.name.clone()
  }

  pub fn schema(&self) -> String {
    self.schema.clone()
  }

  pub fn columns(&self) -> &Vec<Column> {
    &self.columns
  }
}

#[derive(Debug)]
pub struct Column {
  name: String
, data_type: internal::LeafType
}

impl Column {
  pub fn new(name: &str, data_type: internal::LeafType) -> Column {
    Column{ name: name.to_string(), data_type: data_type }
  }

  pub fn name(&self) -> String {
    self.name.clone()
  }

  pub fn data_type(&self) -> internal::LeafType {
    self.data_type.clone()
  }
}