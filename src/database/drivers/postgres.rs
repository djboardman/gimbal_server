
use postgres::{Client, NoTls, error::DbError, Error};

use crate::database::meta;
use crate::database::integration::{DbDiff, DiffDiagnosis};
use crate::lang::internal;

pub fn execute_changes(db_changes: &meta::DatabaseChange, db_config: &meta::DatabaseConfig) -> Result<(), String> {
  let (_, errors): (Vec<_>, Vec<_>) = db_changes.commands().iter().map(|c| execute_ddl(&c)).partition(Result::is_ok);
  if errors.len() > 0 {
    Err(errors.iter().map(|e| e.as_ref().unwrap_err().to_string()).collect::<Vec<String>>().join("\n"))
  } else {
    Ok(())
  }
}

fn execute_ddl(command: &str) -> Result<(), Error> {
  let mut client = 
    Client::connect("host=localhost user=postgres dbname=david user=david password=password", NoTls).unwrap();
  client.batch_execute(command)
}

pub fn db_table_for_ast_table(db_config: &meta::DatabaseConfig, ast_table: &meta::Table) -> Option<meta::Table> {
  let mut client = 
    Client::connect("host=localhost user=postgres dbname=david user=david password=password", NoTls).unwrap();
  let result = client.query("SELECT table_name FROM information_schema.tables WHERE table_name = $1",
               &[&ast_table.name().to_lowercase()]).unwrap();
  match result.len() {
    0 => None
  , 1 => Some(meta::Table::new("", result[0].get(0), db_columns_for_table(&db_config, result[0].get(0))))
  , _ => unreachable!()
  }
}

fn db_columns_for_table(db_config: &meta::DatabaseConfig, table_name: &str) -> Vec<meta::Column> {
  let mut client = 
    Client::connect("host=localhost user=postgres dbname=david user=david password=password", NoTls).unwrap();
  let result = client.query("SELECT column_name, udt_name from information_schema.columns where table_name = $1",
    &[&table_name.to_lowercase()]).unwrap();
  result.iter().map(|r| meta::Column::new(r.get(0), data_type_to_leaf_type(r.get(1)))).collect()
  
}

fn data_type_to_leaf_type(data_type: &str) -> internal::LeafType {
  match data_type {
    "varchar" => internal::LeafType::String
  , _ => unreachable!()
  }
}

pub fn diffs_to_changes(db_diffs: &Vec<DbDiff>, db_config: &meta::DatabaseConfig) -> meta::DatabaseChange {
  meta::DatabaseChange::SqlDb(db_diffs.iter().map(|d| diff_to_command(&d.db_table(), d)).flatten().collect())
}

fn diff_to_command(table: &meta::Table, db_diff: &DbDiff) -> Vec<String> {
  db_diff.diff_diagnosis().iter().map(|d| diagnosis_to_ddl(table, d)).collect()
}

fn diagnosis_to_ddl(table: &meta::Table, diagnosis: &DiffDiagnosis) -> String {
  match diagnosis {
    DiffDiagnosis::NoDiff => "".to_string()
  , DiffDiagnosis::TableMissing => table_ddl(table)
  , _ => "".to_string()
  }
}

fn table_ddl(table: &meta::Table) -> String {
  format!("CREATE TABLE {} {}", table.name(), columns_for_create_ddl(table.columns()))
}

fn columns_for_create_ddl(columns: &Vec<meta::Column>) -> String {
  format!("({})", columns.iter().map(|c| column_ddl(c)).collect::<Vec<String>>().join(", "))
}

fn column_ddl(column: &meta::Column) -> String {
  format!("{} {}", column.name(), data_type_ddl(column.data_type()))
}

fn data_type_ddl(data_type: internal::LeafType) -> String {
  match data_type {
    internal::LeafType::String => "varchar(255)".to_string()
  , internal::LeafType::Int => "integer".to_string()
  , internal::LeafType::Float => "double precision".to_string()
  , internal::LeafType::Bool => "boolean".to_string()
  , internal::LeafType::Id => "uuid".to_string()
  }
}


