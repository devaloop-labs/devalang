use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Type of variable binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    /// let: Block-scoped, can be reassigned
    Let,
    /// var: Function/global-scoped, can be reassigned
    Var,
    /// const: Block-scoped, cannot be reassigned
    Const,
}

/// Variable binding with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub value: Value,
    pub binding_type: BindingType,
    pub is_initialized: bool,
}

impl Binding {
    pub fn new(value: Value, binding_type: BindingType) -> Self {
        Self {
            value,
            binding_type,
            is_initialized: true,
        }
    }

    pub fn can_reassign(&self) -> bool {
        matches!(self.binding_type, BindingType::Let | BindingType::Var)
    }
}

/// Variable table with hierarchical scoping
#[derive(Debug, Clone, PartialEq)]
pub struct VariableTable {
    /// Variables in current scope
    bindings: HashMap<String, Binding>,
    /// Parent scope (if any)
    parent: Option<Box<VariableTable>>,
}

impl Default for VariableTable {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableTable {
    /// Create new empty variable table
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create new variable table with parent scope
    pub fn with_parent(parent: VariableTable) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Set variable with default binding type (Let)
    pub fn set(&mut self, name: String, value: Value) {
        self.set_with_type(name, value, BindingType::Let);
    }

    /// Set variable with specific binding type
    pub fn set_with_type(&mut self, name: String, value: Value, binding_type: BindingType) {
        // For 'var', check if it exists in parent scopes and update there
        if binding_type == BindingType::Var {
            if self.has_var_in_chain(&name) {
                self.update_var_in_chain(name, value);
                return;
            }
        }

        self.bindings
            .insert(name, Binding::new(value, binding_type));
    }

    /// Update existing variable (respecting const)
    pub fn update(&mut self, name: &str, value: Value) -> Result<(), String> {
        // Try to update in current scope
        if let Some(binding) = self.bindings.get_mut(name) {
            if !binding.can_reassign() {
                return Err(format!("Cannot reassign const variable '{}'", name));
            }
            binding.value = value;
            return Ok(());
        }

        // Try to update in parent scope
        if let Some(parent) = &mut self.parent {
            return parent.update(name, value);
        }

        Err(format!("Variable '{}' not found", name))
    }

    /// Get variable value (searches up the scope chain)
    pub fn get(&self, name: &str) -> Option<&Value> {
        if let Some(binding) = self.bindings.get(name) {
            return Some(&binding.value);
        }

        // Search in parent scopes
        let mut current = &self.parent;
        while let Some(boxed) = current {
            if let Some(binding) = boxed.bindings.get(name) {
                return Some(&binding.value);
            }
            current = &boxed.parent;
        }

        None
    }

    /// Get binding (with metadata)
    pub fn get_binding(&self, name: &str) -> Option<&Binding> {
        if let Some(binding) = self.bindings.get(name) {
            return Some(binding);
        }

        let mut current = &self.parent;
        while let Some(boxed) = current {
            if let Some(binding) = boxed.bindings.get(name) {
                return Some(binding);
            }
            current = &boxed.parent;
        }

        None
    }

    /// Check if variable exists in current scope (not parent)
    pub fn has_local(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Check if variable exists in any scope
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Remove variable from current scope
    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.bindings.remove(name).map(|b| b.value)
    }

    /// Get all variables in current scope (for debugging)
    pub fn local_variables(&self) -> Vec<String> {
        self.bindings.keys().cloned().collect()
    }

    /// Check if 'var' exists in this or parent scopes
    fn has_var_in_chain(&self, name: &str) -> bool {
        if let Some(binding) = self.bindings.get(name) {
            return binding.binding_type == BindingType::Var;
        }

        if let Some(parent) = &self.parent {
            return parent.has_var_in_chain(name);
        }

        false
    }

    /// Update 'var' in the scope where it was defined
    fn update_var_in_chain(&mut self, name: String, value: Value) {
        if let Some(binding) = self.bindings.get_mut(&name) {
            if binding.binding_type == BindingType::Var {
                binding.value = value;
                return;
            }
        }

        if let Some(parent) = &mut self.parent {
            parent.update_var_in_chain(name, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_let() {
        let mut table = VariableTable::new();
        table.set_with_type("x".to_string(), Value::Number(42.0), BindingType::Let);
        assert_eq!(table.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_const_cannot_reassign() {
        let mut table = VariableTable::new();
        table.set_with_type("x".to_string(), Value::Number(42.0), BindingType::Const);

        let result = table.update("x", Value::Number(100.0));
        assert!(result.is_err());
        assert_eq!(table.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_let_can_reassign() {
        let mut table = VariableTable::new();
        table.set_with_type("x".to_string(), Value::Number(42.0), BindingType::Let);

        let result = table.update("x", Value::Number(100.0));
        assert!(result.is_ok());
        assert_eq!(table.get("x"), Some(&Value::Number(100.0)));
    }

    #[test]
    fn test_scoped_access() {
        let mut parent = VariableTable::new();
        parent.set("global".to_string(), Value::Number(1.0));

        let mut child = VariableTable::with_parent(parent);
        child.set("local".to_string(), Value::Number(2.0));

        assert_eq!(child.get("local"), Some(&Value::Number(2.0)));
        assert_eq!(child.get("global"), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_var_hoisting() {
        let mut parent = VariableTable::new();
        parent.set_with_type("x".to_string(), Value::Number(1.0), BindingType::Var);

        let mut child = VariableTable::with_parent(parent);
        child.set_with_type("x".to_string(), Value::Number(2.0), BindingType::Var);

        // Verify child has access to the updated var
        assert_eq!(child.get("x"), Some(&Value::Number(2.0)));

        // Extract parent and verify it was updated
        if let Some(parent_box) = child.parent {
            assert_eq!(parent_box.get("x"), Some(&Value::Number(2.0)));
        } else {
            panic!("Parent should exist");
        }
    }

    #[test]
    fn test_shadowing() {
        let mut parent = VariableTable::new();
        parent.set("x".to_string(), Value::Number(1.0));

        let mut child = VariableTable::with_parent(parent.clone());
        child.set("x".to_string(), Value::Number(2.0));

        assert_eq!(child.get("x"), Some(&Value::Number(2.0)));
        assert_eq!(parent.get("x"), Some(&Value::Number(1.0)));
    }
}
