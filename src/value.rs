use std::fmt;
#[derive(Debug, Clone, PartialEq)]

pub enum Value {
    Int(i64),
    Bool(bool),
    Null,
}

impl Value {
    // 判断值是否为真 用于if/while条件判断
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Null => false,
        }
    }
    
    // 尝试将值转换为整数
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }
    
    // 尝试将值转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    // 检查值是否为空
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

// 
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}



// 单元测试代码
#[cfg(test)]
mod tests {
    use super::*;   // 导入父模块的所有项（Value 等）

    #[test]
    fn test_is_truthy() {
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(42).is_truthy());
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Null.is_truthy());
    }

    #[test]
    fn test_as_int() {
        assert_eq!(Value::Int(123).as_int(), Some(123));
        assert_eq!(Value::Bool(true).as_int(), None);
        assert_eq!(Value::Null.as_int(), None);
    }
}