
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

const LEAF_TYPE_STRINGS: &'static [&'static str] = &["String", "Int", "Float", "Bool"];
const INTERNAL_NAMESPACE: &str = "_internal_";

#[derive(Debug)]
pub enum AstError {
  NoSuchAType(String)
, DupAType(String)
, DupModule(String)
, NoSuchModule(String)
, DupDType(String)
}

impl std::error::Error for AstError { }

impl fmt::Display for AstError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AstError::NoSuchAType(name) => write!(f, "{}", format!("I couldn't find an attribute called {}.", name))
    , AstError::DupAType(name) => write!(f, "{}", format!("I've already got an attribute called {} but you've tried to define it again.", name))
    , AstError::DupModule(name) => write!(f, "{}", format!("I've already got a module called {} but you've tried to define it again.", name))
    , AstError::NoSuchModule(name) => write!(f, "{}", format!("I've couldn't find a module called {}.", name))
    , AstError::DupDType(name) => write!(f, "{}", format!("I've already got a datatype called {} but you've tried to define it again.", name))
    }
  }
}


#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct QualifiedName {
  namespace: String
, type_scope: Option<(String, String)>
, name: String
}

impl QualifiedName {
  pub fn new(namespace: &str, name: &str, type_scope: Option<(&str, &str)> ) -> QualifiedName {
    QualifiedName{ namespace: namespace.to_string(), name: name.to_string(), type_scope: type_scope.map(|s| (s.0.to_string(), s.1.to_string())) } 
  }

  pub fn name(&self) -> String {
    self.name.clone()
  }

  pub fn namespace(&self) -> String {
    self.namespace.clone()
  }

}

#[derive(Debug)]
pub struct Application {
  name: String
, types: HashMap<QualifiedName, AType>
, entity_functions: HashMap<QualifiedName, Vec<QualifiedName>>
}

impl Application {
  pub fn new(name: &str, atypes: HashMap<QualifiedName, AType>, entity_functions: HashMap<QualifiedName, Vec<QualifiedName>>) -> Application {
    Application { name: name.to_string(), types: atypes, entity_functions }
  }
  pub fn name(&self) -> String {
    self.name.clone()
  }

  pub fn get_type(&self, qualified_name: &QualifiedName) -> Option<&AType> {
    self.types.get(qualified_name)
  }

  pub fn get_entity_functions(&self, qualified_name: &QualifiedName) -> Option<&Vec<QualifiedName>> {
    self.entity_functions.get(qualified_name)
  }
}


#[derive(Debug)]
pub struct EntityType {
  qualified_name: QualifiedName
, function_types: Vec<QualifiedName>
}

impl EntityType {
  pub fn new(qualified_name: QualifiedName) -> EntityType {
    EntityType{ qualified_name, function_types: vec!() }
  }

  pub fn qualified_name(&self) -> QualifiedName {
    self.qualified_name.clone()
  }

  pub fn name(&self) -> String {
    self.qualified_name.name.clone()
  }
}

pub fn add_function_to_entity_type(entity: EntityType, qualified_name: QualifiedName) -> EntityType {
  let mut new_entity = EntityType::new(entity.qualified_name());
  new_entity.function_types = entity.function_types;
  new_entity.function_types.push(qualified_name);
  new_entity
}

#[derive(Debug)]
pub struct FunctionType {
  qualified_name: QualifiedName
, dom: QualifiedName
, codom: QualifiedName 
}

impl FunctionType {
  pub fn new(qualified_name: QualifiedName, dom: QualifiedName, codom: QualifiedName) -> FunctionType {
    FunctionType{ qualified_name, dom, codom }
  }

  pub fn qualified_name(&self) -> QualifiedName {
    self.qualified_name.clone()
  }

  pub fn name(&self) -> String {
    self.qualified_name.name.clone()
  }

  pub fn dom(&self) -> QualifiedName {
    self.dom.clone()
  }

  pub fn codom(&self) -> QualifiedName {
    self.codom.clone()
  }
}



#[derive(Debug)]
enum LeafType {
  String
, Int
, Float
, Bool
}

#[derive(Debug)]
pub enum AType {
  FunctionType(FunctionType)
, EntityType(EntityType)
, LeafType(LeafType)
}

impl AType {
  pub fn name(&self) -> String {
    match self {
      AType::EntityType(e) => e.name()
    , AType::FunctionType(f) => f.name()
    , _ => "".to_string()
    }
  }

  pub fn try_to_function_type(&self) -> Option<&FunctionType> {
    match self {
      AType::FunctionType(f) => Some(f)
    , _ => None
    }
  }
}


