use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

// 运行时环境 用于存储变量
#[derive(Debug, Clone, Default)]
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,   // Option<shared_ptr<#*<Environment>>> + RefCell -> 提供 &mut T 以外的借用可变引用的方式;
}                                               // 即 borrow_mut(), borrow();   ps: Rc<RefCell<T>> 可实现可共享、可修改的引用，但需小心避免循环引用

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

    // 调试输出 打印当前环境 + 所有父环境的变量
    pub fn dump(&self) {
        self.dump_with_indent(0);
    }

    fn dump_with_indent(&self, indent: usize) {
        let indent_str = "  ".repeat(indent);
        println!("{}Environment ({} variables):", indent_str, self.variables.len());
        for (name, value) in &self.variables {
            println!("{}  {} = {}", indent_str, name, value);
        }
        if let Some(parent) = &self.parent {
            println!("{}parent:", indent_str);
            parent.borrow().dump_with_indent(indent + 1);
        }
    }

    pub fn dump_to_string(&self, output: &mut String) {
        self.dump_to_string_with_indent(0, output);
    }

    fn dump_to_string_with_indent(&self, indent: usize, output: &mut String) {
        let indent_str = "  ".repeat(indent);
        output.push_str(&format!("{}Environment ({} variables):\n", indent_str, self.variables.len()));
        for (name, value) in &self.variables {
            output.push_str(&format!("{}  {} = {}\n", indent_str, name, value));
        }
        if let Some(parent) = &self.parent {
            output.push_str(&format!("{}parent:\n", indent_str));
            parent.borrow().dump_to_string_with_indent(indent + 1, output);
        }
    }

    // 获取当前作用域的变量值
    pub fn get_local(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    // 判断变量是否在当前作用域定义
    pub fn defined_local(&self, name: &str) -> bool {
        self.variables.contains_key(name)
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

    #[test]
    fn test_get_local() {
        let env = Environment::new();
        env.borrow_mut().define("x", Value::Int(10));
        // 创建子环境并遮蔽 x
        let child = Environment::new_with_parent(env.clone());
        child.borrow_mut().define("x", Value::Int(20));
        
        assert_eq!(child.borrow().get("x"), Some(Value::Int(20)));          // get 向上查找，取最近
        assert_eq!(child.borrow().get_local("x"), Some(Value::Int(20)));    // get_local 只取当前层
        assert_eq!(child.borrow().get_local("a"), None);                    // 父环境变量不在当前层
    }

    #[test]
    fn test_defined_local() {
        let env = Environment::new();
        env.borrow_mut().define("x", Value::Int(10));
        let child = Environment::new_with_parent(env.clone());
        
        assert!(env.borrow().defined_local("x"));
        assert!(!child.borrow().defined_local("x")); // 子环境未定义 x，虽然父环境有
    }

    // 测试 dump (无panic)
    #[test]
    fn test_dump_does_not_panic() {
        let env = Environment::new();
        env.borrow_mut().define("foo", Value::Int(123));
        let child = Environment::new_with_parent(env.clone());
        child.borrow_mut().define("bar", Value::Bool(true));
        child.borrow().dump();
    }
}