
use std::collections::{BTreeSet, HashMap};

use pest::{Parser, Span};
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "./lang/gimbal_lang.pest"]
pub struct DbParser;

pub fn parse(code: &str) -> FileRoot {
  let pairs = DbParser::parse(Rule::file, code).unwrap();

  let code_pair = code_rule(pairs).expect("no code found");
  let cst_nodes = code_pair.into_inner().map(|p| cst_node(p)).collect::<Vec<CodeNode>>();
  let mut imports: HashMap<String, Import> = HashMap::new();
  let mut entity_types: HashMap<String, EntityType> = HashMap::new();
  let mut function_types: HashMap<String, FunctionType> = HashMap::new();
  let mut app_def: Option<AppDef> = None;
  let mut namespace: Namespace = Namespace{ name: "".to_string() };
  let mut used_namespaces: HashMap<String, Namespace> = HashMap::new();
  cst_nodes.into_iter().for_each(|n| {
    match n {
      CodeNode::Import(i) => {
        if !imports.contains_key(&i.path) {
          imports.insert(i.path.clone(), i);
        }
      }
    , CodeNode::AppDef(a) => {
      app_def = Some(a);
      }
    , CodeNode::EntityType(e) => {
        if !entity_types.contains_key(&e.name) {
          entity_types.insert(e.name.clone(), e);
        }
      }
    , CodeNode::FunctionType(f) => {
        if !function_types.contains_key(&f.name) {
          function_types.insert(f.name.clone(), f);
        }
      }
    , CodeNode::Namespace(n) => {
        namespace = n;
      }
    , CodeNode::UsedNamespace(n) => {
        used_namespaces.insert(n.name.clone(), n);
      } 
    }
  });


  FileRoot{ imports, app_def, entity_types, function_types, namespace, used_namespaces }


}

fn code_rule(mut pairs: Pairs<Rule>) -> Option<Pair<Rule>> {
  pairs.find(|p| p.as_rule() == Rule::code )
}

fn cst_node(pair: Pair<Rule>) -> CodeNode {
  match pair.as_rule() {
    Rule::import => CodeNode::Import(import_from_pairs(pair.into_inner()).expect("cst/pst mismatch on import"))
  , Rule::app_def => CodeNode::AppDef(app_def_from_pairs(pair.into_inner()).expect("cst/pst mismatch on app def"))
  , Rule::struct_type => CodeNode::EntityType(entity_type_from_pairs(pair.into_inner()).expect("cst/pst mismatch on entity"))
  , Rule::function_type => CodeNode::FunctionType(function_type_from_pairs(pair.into_inner()).expect("cst/pst mismatch on function"))
  , Rule::namespace => CodeNode::Namespace(Namespace{ name: pair.as_str().to_string()})
  , Rule::use_namespace => CodeNode::UsedNamespace(Namespace {name: pair.into_inner().as_str().to_string()})
  , _ => unreachable!()
  }
}


fn import_from_pairs(mut pairs: Pairs<Rule>) -> Option<Import> {
  let path = pairs.next()?.as_str().replace("\"", "").to_string();
  Some(Import{ path })
}

fn app_def_from_pairs(mut pairs: Pairs<Rule>) -> Option<AppDef> {
  let name = pairs.next()?.as_str().to_string();
  Some(AppDef{ name })
}

fn entity_type_from_pairs(mut pairs: Pairs<Rule>) -> Option<EntityType> {
  let duration = pairs.next()?.as_str().to_string();
  let name = pairs.next()?.as_str().to_string();
  Some(EntityType{ name, duration })
}

fn function_type_from_pairs(mut pairs: Pairs<Rule>) -> Option<FunctionType> {
  let name = pairs.next().unwrap().as_str().to_string();

  let dom = qualified_name(pairs.next().expect("dom not found").into_inner());
  let codom = qualified_name(pairs.next().expect("codom not found").into_inner());

  //let dom = pairs.next().unwrap().as_str().to_string();
  //let codom = pairs.next().unwrap().as_str().to_string();

  Some(FunctionType{ name, dom, codom})
}

fn qualified_name(mut pairs: Pairs<Rule>) -> (Option<String>, String) {
  let mut ns: Option<String> = None;
  let mut n: Option<String> = None;
  pairs.for_each(|p| {
    match p.as_rule() {
      Rule::namespace => ns = Some(p.as_str().to_string())
    , Rule::type_name => n = Some(p.as_str().to_string())
    , _ => unreachable!()
    }
  });
  (ns, n.expect("cst/pest mismatch for dom or codom"))
}

#[derive(Debug)]
struct Import {
  path: String
}

#[derive(Debug)]
struct AppDef {
  name: String
}

#[derive(Debug)]
struct Namespace {
  name: String
}

#[derive(Debug)]
pub struct FunctionType {
  name: String
, dom: (Option<String>, String)
, codom: (Option<String>, String)
}

impl FunctionType {
  pub fn new(name: &str, dom: (Option<String>, String), codom: (Option<String>, String)) -> FunctionType {
    FunctionType{ name: name.to_string(), dom: dom, codom: codom }
  }

  pub fn name(&self) -> String {
    self.name.clone()
  }

  pub fn dom(&self) -> (Option<String>, String) {
    self.dom.clone()
  } 

  pub fn codom(&self) -> (Option<String>, String) {
    self.codom.clone()
  }
}

#[derive(Debug)]
pub struct EntityType {
  name: String
, duration: String
}

impl EntityType {
  fn new(name: &str, duration: &str) -> EntityType {
    EntityType{ name: name.to_string(), duration: duration.to_string() }
  }

  pub fn name(&self) -> String {
    self.name.to_string()
  }
}

#[derive(Debug)]
enum CodeNode {
  FunctionType(FunctionType)
, AppDef(AppDef)
, EntityType(EntityType)
, Import(Import)
, Namespace(Namespace)
, UsedNamespace(Namespace)
}

#[derive(Debug)]
pub struct FileRoot {
  app_def: Option<AppDef>
, imports: HashMap<String, Import>
, entity_types: HashMap<String, EntityType>
, function_types: HashMap<String, FunctionType>
, namespace: Namespace
, used_namespaces: HashMap<String, Namespace>
}

impl FileRoot {
  pub fn app_name(&self) -> Option<String> {
    Some(self.app_def.as_ref()?.name.clone())
  }

  pub fn is_main_file(&self) -> bool {
    match &self.app_def {
      Some(_) => true
    , None => false
    }
  }

  pub fn import_paths(&self) -> Vec<String> {
    self.imports.values().map(|i| i.path.clone()).collect()
  }

  pub fn entity_types(&self) -> Vec<&EntityType> {
    self.entity_types.values().collect()
  }

  pub fn function_types(&self) -> Vec<&FunctionType> {
    self.function_types.values().collect()
  }

  pub fn namespace(&self) -> String {
    self.namespace.name.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid_code_test() {
    let valid_code = r#"
  use std
  namespace mine where
  
  struct persists Person
  name:: Person -> String"#;
    let cst = parse(valid_code);
    assert_eq!(cst.namespace.name, "mine");
    assert_eq!(cst.used_namespaces.get("std").unwrap().name, "std");
    assert_eq!(cst.entity_types.get("Person").unwrap().name, "Person");
    assert_eq!(cst.function_types.get("name").unwrap().name, "name");
  }

  #[test]
  fn app_file_test() {
    let valid_code = r#"
import "../another_source"
app my_app"#;
    let cst = parse(valid_code);
    assert_eq!(cst.app_def.unwrap().name, "my_app");
  }
}
