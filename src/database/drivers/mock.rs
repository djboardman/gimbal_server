use std::collections::HashMap;
use std::error;

use crate::database::meta;
use crate::database::integration::{DbDiff, DiffDiagnosis};

static mut TABLES: Vec<meta::Table> = Vec::new();


pub fn execute_changes(db_changes: &meta::DatabaseChange, db_config: &meta::DatabaseConfig) -> Result<(), String> {
  Ok(())
}

pub fn diffs_to_changes(db_diffs: &Vec<DbDiff>, db_config: &meta::DatabaseConfig) -> meta::DatabaseChange {
  //just going to copy all the tables from the diff for the purposes of mock
  let tables: Vec<meta::Table> = db_diffs.iter().map(|d| copy_table(d.db_table())).collect();
  meta::DatabaseChange::MockDb(meta::DatabaseConfig::MockDb(meta::MockDbConfig{ tables }))
}

pub fn db_table_for_ast_table(db_config: &meta::DatabaseConfig, ast_table: &meta::Table) -> Option<meta::Table>  {
  match db_config {
    meta::DatabaseConfig::MockDb(config) => {
      let t_copy = copy_tables(&config.tables);
      Some(t_copy.into_iter().find(|t| t.name() == ast_table.name()))?
    }
  , _ => unreachable!()
  }
}

fn copy_tables(tables: &Vec<meta::Table>) -> Vec<meta::Table> {
  tables.iter().map(|t| copy_table(t)).collect()
}

fn copy_table(table: &meta::Table) -> meta::Table {
  meta::Table::new("schema", &table.name(), copy_columns(table.columns()))
}

fn copy_columns(columns: &Vec<meta::Column>) -> Vec<meta::Column> {
  columns.iter().map(|c| copy_column(c)).collect()
}

fn copy_column(column: &meta::Column) -> meta::Column {
  meta::Column::new(&column.name(), column.data_type())
}

fn tables_for_entities(entities: Vec<String>) -> HashMap<String, Option<&'static meta::Table>> {
  unsafe {
   entities.into_iter().map(|e| (e.clone(), TABLES.iter().find(|t| t.name() == e).clone())).collect()
  }
}

fn tables() -> Vec<meta::Table> {
  unsafe {
    TABLES.iter().map(|t| meta::Table::new(&t.schema(), &t.name(), vec!())).collect::<Vec<meta::Table>>()
  }
}

fn create_missing(table_list: &HashMap<String, Option<&meta::Table>>) -> Result<(), Box<dyn error::Error>> {

  table_list.iter().for_each(|(k, v)| {
    match v {
      None => unsafe{ 
        TABLES.push(meta::Table::new("", &k.clone(), vec!())) 
      }
    , Some(table) => {}
    }
  });

  Ok(())

}