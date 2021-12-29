
use std::env;
use std::io;
use std::fs;
use std::error::Error;
use std::path::{Path};
use std::collections::HashMap;

use crate::lang::cst;
use crate::lang::ast;

fn code_from_file(file_path_str: &str) -> io::Result<String> {
  fs::read_to_string(file_path_str)
}

pub fn build_from_main_file(main_path_str: &str) -> Result<ast::Application, Box<dyn Error>> {
  let main_code = code_from_file(main_path_str)?;
  let main_path = Path::new(main_path_str).canonicalize()?;
  let working_dir = main_path.parent().expect("could not get working dir");
  env::set_current_dir(working_dir)?;
  Ok(build(&main_code)?)
}

fn build(main_code: &str) -> Result<ast::Application, Box<dyn Error>> {
  let main = cst::parse(&main_code);
  let import_path_strs = main.import_paths();
  let import_csts: Vec<cst::FileRoot> = import_path_strs.iter().map(|p| cst::parse(&code_from_file(&p).expect("couldn't read file"))).collect();
  let mut atypes: HashMap<ast::QualifiedName, ast::AType> = HashMap::new();
  import_csts.iter().for_each(|c| {
    c.entity_types().iter().for_each(|e| {
      let entity_qn = ast::QualifiedName::new(&c.namespace(), &e.name(), None);
      let ae = ast::EntityType::new(entity_qn);
      atypes.insert(ae.qualified_name(), ast::AType::EntityType(ae));
    });

    c.function_types().iter().for_each(|f| {
      let dom_qn= qn_default(&c.namespace(), f.dom());
      let codom_qn = qn_default(&c.namespace(), f.codom());
      let fn_qn = ast::QualifiedName::new(&c.namespace(), &f.name(), Some((&dom_qn.namespace(), &dom_qn.name())));
      let af = ast::FunctionType::new(fn_qn, dom_qn, codom_qn);
      atypes.insert(af.qualified_name(), ast::AType::FunctionType(af));
    });

  });

  let mut domains: HashMap<ast::QualifiedName, Vec<ast::QualifiedName>> = HashMap::new();
  atypes.values().for_each(|e| {
    match e {
      ast::AType::FunctionType(f) => {
        let fq = f.qualified_name();
        let dq = f.dom();
        let eq: ast::QualifiedName = match atypes.get(&dq).expect("entity for domain not found") {
          ast::AType::EntityType(e) => e.qualified_name()
        , _ => unreachable!()
        };
        if domains.contains_key(&eq) {
          let mut d = domains.remove(&eq).unwrap();
          d.push(fq);
          domains.insert(eq, d);
        } else {
          domains.insert(eq, vec!(fq));
        }
      }
    , _ => {}
    }
  });
  
  // = import_csts.iter().map(|c| c.entity_types().iter().map(|e| (c.namespace(), ast::AType::EntityType(ast::EntityType::new(&c.namespace(), &e.name()))))).collect();
  Ok(ast::Application::new(&main.app_name().expect("no application name found"), atypes, domains))
}

fn qn_default(default_namespace: &str, qualified_pair: (Option<String>, String)) -> ast::QualifiedName {

  ast::QualifiedName::new(&qualified_pair.0.unwrap_or(default_namespace.to_string()), &qualified_pair.1, None)
}



#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn main_test() {
    let main_code = r#"
app my_app"#;
    let app = build(main_code);
    
  }

  #[test]
  fn example_test() {
    let main_file_path = "./examples/ast_builder/main.gim";
    let app = build_from_main_file(main_file_path).unwrap();
    assert_eq!(app.name(), "my_app");
    let resource_qn = ast::QualifiedName::new("example", "Resource", None);
    assert_eq!(app.get_type(&resource_qn).unwrap().name(), "Resource");

    let name_fn_qn = ast::QualifiedName::new("example", "name", Some(("example", "Agent")));
    assert_eq!(app.get_type(&name_fn_qn).unwrap().name(), "name");
    
    assert_eq!(app.get_type(&name_fn_qn).unwrap().try_to_function_type().unwrap().dom().namespace(), "example");
    assert_eq!(app.get_type(&name_fn_qn).unwrap().try_to_function_type().unwrap().codom().namespace(), "elsewhere");
    assert_eq!(app.get_type(&name_fn_qn).unwrap().try_to_function_type().unwrap().codom().namespace(), "elsewhere");
    
    let agent_qn = ast::QualifiedName::new("example", "Agent", None);
    assert_eq!(app.get_entity_functions(&agent_qn).unwrap()[0].name(), "name");
  }
}


