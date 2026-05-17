use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::{Expr, Opcode, Stmt, Type};
use crate::value::Value;
use crate::environment::Environment;

/// 解释器，负责执行AST
pub struct Evaluator {
    environment: Rc<RefCell<Environment>>,
}

impl Evaluator {
    /// 创建新的解释器
    pub fn new(environment: Rc<RefCell<Environment>>) -> Self {
        Self { environment }
    }
    
    /// 执行语句列表
    // pub fn evaluate(&mut self, stmts: &[Stmt]) -> Result<Value, String> {
    //     let mut result = Value::Null;
        
    //     for stmt in stmts {
    //         result = self.execute(stmt)?;
    //     }
        
    //     Ok(result)
    // }
    
    pub fn evaluate(&mut self, stmts: &[Stmt]) -> Result<Value, String> {
        let mut result = Value::Null;
        
        for stmt in stmts {
            result = self.execute(stmt)?;
            // 如果语句是表达式语句，打印结果
            if let Stmt::Expr(_) = stmt {
                if !result.is_null() {
                    println!("{}", result);
                }
            }
        }
        
        Ok(result)
    }

    /// 执行单条语句
    fn execute(&mut self, stmt: &Stmt) -> Result<Value, String> {
        match stmt {
            Stmt::Assign(var, expr) => {
                let value = self.eval_expr(expr)?;
                self.environment.borrow_mut().assign(var, value.clone())?;
                Ok(value)
            }
            
            Stmt::VarDecl(vars, var_type) => {
                for var in vars {
                    // 根据类型设置默认值
                    let default_value = match var_type {
                        Type::Int => Value::Int(0),
                        Type::Bool => Value::Bool(false),
                    };
                    self.environment.borrow_mut().define(var, default_value);
                }
                Ok(Value::Null)
            }
            
            Stmt::Expr(expr) => {
                self.eval_expr(expr)
            }
            
            Stmt::If(cond, then_branch) => {
                if self.eval_expr(cond)?.is_truthy() {
                    self.evaluate(then_branch)?;
                }
                Ok(Value::Null)
            }
            
            Stmt::IfElse(cond, then_branch, else_branch) => {
                if self.eval_expr(cond)?.is_truthy() {
                    self.evaluate(then_branch)?;
                } else {
                    self.evaluate(else_branch)?;
                }
                Ok(Value::Null)
            }
            
            Stmt::While(cond, body) => {
                while self.eval_expr(cond)?.is_truthy() {
                    self.evaluate(body)?;
                }
                Ok(Value::Null)
            }
            
            Stmt::Block(stmts) => {
                // 为块语句创建新的作用域
                let new_env = Environment::new_with_parent(self.environment.clone());
                let mut block_evaluator = Evaluator::new(new_env);
                block_evaluator.evaluate(stmts)
            }
        }
    }
    
    /// 计算表达式的值
    fn eval_expr(&self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Id(name, _, _) => {
                self.environment.borrow()
                    .get(name)
                    .ok_or_else(|| format!("变量 '{}' 未定义", name))
            }
            
            Expr::Num(n) => Ok(Value::Int(*n as i64)),
            
            Expr::Op(op, left, right) => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                
                match op {
                    Opcode::Add => {
                        let left_int = left_val.as_int().ok_or("加法需要整数操作数")?;
                        let right_int = right_val.as_int().ok_or("加法需要整数操作数")?;
                        Ok(Value::Int(left_int + right_int))
                    }
                    
                    Opcode::Sub => {
                        let left_int = left_val.as_int().ok_or("减法需要整数操作数")?;
                        let right_int = right_val.as_int().ok_or("减法需要整数操作数")?;
                        Ok(Value::Int(left_int - right_int))
                    }
                    
                    Opcode::Mul => {
                        let left_int = left_val.as_int().ok_or("乘法需要整数操作数")?;
                        let right_int = right_val.as_int().ok_or("乘法需要整数操作数")?;
                        Ok(Value::Int(left_int * right_int))
                    }
                    
                    Opcode::GreaterThan => {
                        let left_int = left_val.as_int().ok_or("比较需要整数操作数")?;
                        let right_int = right_val.as_int().ok_or("比较需要整数操作数")?;
                        Ok(Value::Bool(left_int > right_int))
                    }
                    
                    Opcode::LessThan => {
                        let left_int = left_val.as_int().ok_or("比较需要整数操作数")?;
                        let right_int = right_val.as_int().ok_or("比较需要整数操作数")?;
                        Ok(Value::Bool(left_int < right_int))
                    }
                    
                    Opcode::Equal => {
                        // 支持整数和布尔值的比较
                        if let (Some(left_int), Some(right_int)) = (left_val.as_int(), right_val.as_int()) {
                            Ok(Value::Bool(left_int == right_int))
                        } else if let (Some(left_bool), Some(right_bool)) = (left_val.as_bool(), right_val.as_bool()) {
                            Ok(Value::Bool(left_bool == right_bool))
                        } else {
                            Err("相等比较需要相同类型的操作数".to_string())
                        }
                    }
                    
                    Opcode::And => {
                        let left_bool = left_val.as_bool().ok_or("逻辑与需要布尔操作数")?;
                        let right_bool = right_val.as_bool().ok_or("逻辑与需要布尔操作数")?;
                        Ok(Value::Bool(left_bool && right_bool))
                    }
                    
                    Opcode::Or => {
                        let left_bool = left_val.as_bool().ok_or("逻辑或需要布尔操作数")?;
                        let right_bool = right_val.as_bool().ok_or("逻辑或需要布尔操作数")?;
                        Ok(Value::Bool(left_bool || right_bool))
                    }
                }
            }
            
            Expr::Not(expr) => {
                let val = self.eval_expr(expr)?;
                let bool_val = val.as_bool().ok_or("逻辑非需要布尔操作数")?;
                Ok(Value::Bool(!bool_val))
            }
        }
    }
}