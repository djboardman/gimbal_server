
use std::collections::HashMap;
use std::error;

use crate::lang::{ast, internal};
use crate::database::drivers::mock;
use crate::database::meta;


#[derive(Debug)]
pub struct DbDiff {
  entity_table: AstTable
, diff_diagnosis: Vec<DiffDiagnosis>
}

impl DbDiff {
  pub fn diff_diagnosis(&self) -> &Vec<DiffDiagnosis> {
    &self.diff_diagnosis
  }

  pub fn db_table(&self) -> &meta::Table {
    &self.entity_table.table
  }
}

#[derive(Debug)]
pub struct AstTable {
  entity_name: ast::QualifiedName
, table: meta::Table
}

#[derive(Debug, PartialEq)]
pub enum DiffDiagnosis {
  NoDiff
, TableMissing
, ColumnMissing(String)
, ColumnTypeMismatch(String, internal::LeafType, internal::LeafType)
}

fn ast_to_db(ast: &ast::Application) -> meta::Database {
  let tables = ast.entity_functions().iter().map(|(k, _)| {
    entity_to_table(ast, k)
  }).collect::<Vec<meta::Table>>();
  meta::Database::new(tables)
}

fn entity_to_table(ast: &ast::Application, qn: &ast::QualifiedName) ->  meta::Table {
  let fn_qns = ast.get_entity_functions(qn).expect("entity not found");
  meta::Table::new("schema", &qn.table_name(), functions_to_columns(ast, fn_qns))
}

fn functions_to_columns(ast: &ast::Application, qn: &Vec<ast::QualifiedName>) -> Vec<meta::Column> {
  qn.iter().map(|qn| {
    function_to_column(ast, qn)
  }).collect::<Vec<meta::Column>>()
} 

fn function_to_column(ast: &ast::Application, qn: &ast::QualifiedName) -> meta::Column {
  let a =  ast.get_type(&qn).expect("function not found");
  let c_qn = match a {
    ast::AType::FunctionType(f) => f.codom()
  , _ => unreachable!()
  };
  let c = ast.get_type(&c_qn).expect("codom not found");
  match c {
    ast::AType::LeafType(lt) => meta::Column::new(&qn.name(), (*lt).clone())
  , ast::AType::EntityType(_) => meta::Column::new(&qn.name(), internal::LeafType::Id)
  , ast::AType::FunctionType(_) => {
      meta::Column::new(&(qn.name() + "_param")
                                      , internal::LeafType::String)
    }
  }  
}

pub fn diagnose_db_diffs(ast: &ast::Application, db_config: &meta::DatabaseConfig) -> Vec<DbDiff> {
  ast.entity_functions().keys().map(|e_qn| diagnose_diff(ast, db_config, e_qn)).collect()
}

fn diagnose_diff(ast: &ast::Application, db_config: &meta::DatabaseConfig, entity_qname: &ast::QualifiedName) -> DbDiff {
  let entity_table = entity_to_table(ast, entity_qname);
  let database_table = mock::db_table_for_ast_table(db_config, &entity_table);
  let diff_diagnosis = diagnose_table(&entity_table, database_table);
  let ast_table = AstTable{ entity_name: entity_qname.clone(), table: entity_table };
  DbDiff{ entity_table: ast_table, diff_diagnosis }
}

fn diagnose_table(entity_table: &meta::Table, database_table: Option<meta::Table>) -> Vec<DiffDiagnosis> {
  match database_table {
    None => vec!(DiffDiagnosis::TableMissing)
  , Some(dt) => diagnose_columns(entity_table.columns(), dt.columns())
  }
}

fn diagnose_columns(entity_columns: &Vec<meta::Column>, database_columns: &Vec<meta::Column>) -> Vec<DiffDiagnosis> {
  entity_columns.iter().map(|c| diagnose_column(c, database_columns)).collect()
}

fn diagnose_column(entity_column: &meta::Column, database_columns: &Vec<meta::Column>) -> DiffDiagnosis {
  match database_columns.iter().find(|dc| dc.name() == entity_column.name()) {
    None => DiffDiagnosis::ColumnMissing(entity_column.name())
  , Some(c) => {
      if entity_column.data_type().name() != c.data_type().name() {
        DiffDiagnosis::ColumnTypeMismatch(entity_column.name(), entity_column.data_type(), c.data_type())
      } else {
        DiffDiagnosis::NoDiff
      }
    }
  }
}

fn diffs_to_script(db_diffs: &Vec<DbDiff>, db_config: &meta::DatabaseConfig) -> meta::DatabaseChange {
  mock::diffs_to_changes(db_diffs, db_config)
}

fn tables(db: &meta::DatabaseConfig) -> Vec<meta::Table> {
  mock::tables()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lang::ast_builder;

  #[test]
  fn test_table_missing() {
    let code = r#"
app database

namespace db where

entity persists Agent

name:: Agent -> String"#;

    let ast = ast_builder::build(code).unwrap();
    let ast_db = ast_to_db(&ast);
    let mock_db_config = meta::DatabaseConfig::MockDb(meta::MockDbConfig{ tables: Vec::new() });
    let db_diff = diagnose_db_diffs(&ast, &mock_db_config);
    assert_eq!(ast_db.tables()[0].name(),  "db_Agent");
    //println!("{:?}", db_diff);
    assert!(matches!(db_diff[0].diff_diagnosis[0], DiffDiagnosis::TableMissing));
  }

  #[test]
  fn test_nothing_missing() {
    let code = r#"
app database

namespace db where

entity persists Agent

name:: Agent -> String"#;

    let ast = ast_builder::build(code).unwrap();
    let ast_db = ast_to_db(&ast);
    let mock_column = meta::Column::new("name", internal::LeafType::String);
    let mock_table = meta::Table::new("schema", "db_Agent", vec!(mock_column));
    let mock_db_config = meta::DatabaseConfig::MockDb(meta::MockDbConfig{ tables: vec!(mock_table) });
    let db_diff = diagnose_db_diffs(&ast, &mock_db_config);
    assert_eq!(ast_db.tables()[0].name(),  "db_Agent");
    //println!("{:?}", db_diff);
    assert!(matches!(db_diff[0].diff_diagnosis[0], DiffDiagnosis::NoDiff));
  }

  #[test]
  fn test_column_missing() {
    let code = r#"
app database

namespace db where

entity persists Agent

name:: Agent -> String"#;

    let ast = ast_builder::build(code).unwrap();
    let ast_db = ast_to_db(&ast);
    let _entity_column_name = "name".to_string();
    let _mock_column_name = "another_column".to_string();
    let mock_column = meta::Column::new(&_mock_column_name, internal::LeafType::String);
    let mock_table = meta::Table::new("schema", "db_Agent", vec!(mock_column));
    let mock_db_config = meta::DatabaseConfig::MockDb(meta::MockDbConfig{ tables: vec!(mock_table) });
    let db_diff = diagnose_db_diffs(&ast, &mock_db_config);
    assert_eq!(ast_db.tables()[0].name(),  "db_Agent");
    assert!(matches!(&db_diff[0].diff_diagnosis[0], DiffDiagnosis::ColumnMissing(_entity_column_name)));
  }

  #[test]
  fn test_column_type_mismatch() {
    let code = r#"
app database

namespace db where

entity persists Agent

name:: Agent -> String"#;

    let ast = ast_builder::build(code).unwrap();
    let ast_db = ast_to_db(&ast);
    let _entity_column_name = "name".to_string();
    let mock_column = meta::Column::new(&_entity_column_name, internal::LeafType::Int);
    let mock_table = meta::Table::new("schema", "db_Agent", vec!(mock_column));
    let mock_db_config = meta::DatabaseConfig::MockDb(meta::MockDbConfig{ tables: vec!(mock_table) });
    let db_diff = diagnose_db_diffs(&ast, &mock_db_config);
    let db_change = diffs_to_script(&db_diff, &mock_db_config);
    assert_eq!(ast_db.tables()[0].name(),  "db_Agent");
    assert!(matches!(&db_diff[0].diff_diagnosis[0], DiffDiagnosis::ColumnTypeMismatch(_entity_column_name, internal::LeafType::String, internal::LeafType::Int)));
  }
}