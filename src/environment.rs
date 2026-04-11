use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

// 运行时环境 用于存储变量
#[derive(Debug, Clone, Default)]
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    // 新建全局环境
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: HashMap::new(),
            parent: None,
        }))
    }
    
    // 新建子环境
    pub fn new_with_parent(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }))
    }
    
    // 定义变量
    pub fn define(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }
    
    // 获取变量值
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }
    
    // 变量赋值及修改
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            Err(format!("变量 '{}' 未定义", name))
        }
    }
}



// 单元测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;   // environment 依赖 value

    #[test]
    fn test_define_and_get() {
        let env = Environment::new();
        env.borrow_mut().define("x", Value::Int(10));
        assert_eq!(env.borrow().get("x"), Some(Value::Int(10)));
        assert_eq!(env.borrow().get("y"), None);
    }

    #[test]
    fn test_nested_scope() {
        let parent = Environment::new();
        parent.borrow_mut().define("a", Value::Int(1));
        let child = Environment::new_with_parent(parent.clone());
        child.borrow_mut().define("b", Value::Int(2));
        
        // 子环境可以访问父环境的变量
        assert_eq!(child.borrow().get("a"), Some(Value::Int(1)));
        // 父环境不能访问子环境的变量
        assert_eq!(parent.borrow().get("b"), None);
    }

    #[test]
    fn test_assign() {
        let env = Environment::new();
        env.borrow_mut().define("x", Value::Int(0));
        env.borrow_mut().assign("x", Value::Int(42)).unwrap();
        assert_eq!(env.borrow().get("x"), Some(Value::Int(42)));
        
        // 对未定义的变量赋值应报错
        let result = env.borrow_mut().assign("y", Value::Int(99));
        assert!(result.is_err());
    }
}