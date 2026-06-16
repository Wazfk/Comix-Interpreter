use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::ast::{Expr, Opcode, Stmt, Type};
use crate::value::Value;
use crate::environment::Environment;

type FuncBody = Vec<Stmt>;
type ParamList = Vec<(String, Type)>;

#[derive(Debug)]
pub enum EvalError {
    Runtime(String),
    Return(Value),
}

impl From<String> for EvalError {
    fn from(s: String) -> Self {
        EvalError::Runtime(s)
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Runtime(msg) => write!(f, "{}", msg),
            EvalError::Return(v) => write!(f, "{}", v),
        }
    }
}

fn get_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (byte_idx, byte) in source.as_bytes().iter().enumerate() {
        if byte_idx == offset {
            break;
        }
        if *byte == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub struct Evaluator {
    environment: Rc<RefCell<Environment>>,
    source: String,
    functions: HashMap<String, (ParamList, Type, FuncBody, usize, usize)>,
}

impl Evaluator {
    pub fn new(environment: Rc<RefCell<Environment>>, source: String) -> Self {
        Self {
            environment,
            source,
            functions: HashMap::new(),
        }
    }

    pub fn set_source(&mut self, source: String) {
        self.source = source;
    }

    pub fn evaluate(&mut self, stmts: &[Stmt]) -> Result<Value, EvalError> {
        let mut result = Value::Null;
        for stmt in stmts {
            result = self.execute(stmt)?;
        }
        Ok(result)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Value, EvalError> {
        match stmt {
            Stmt::Assign(lhs, rhs) => {
                let value = self.eval_expr(rhs)?;
                match lhs.as_ref() {
                    Expr::Id(name, ..) => {
                        self.environment.borrow_mut().assign(name, value.clone())
                            .map_err(EvalError::Runtime)?;
                        Ok(value)
                    }
                    Expr::Index(arr_expr, idx_expr, ..) => {
                        // 先求数组和索引
                        let arr_val = self.eval_expr(arr_expr)?;
                        let idx_val = self.eval_expr(idx_expr)?;
                        let idx = idx_val.as_int()
                            .ok_or_else(|| EvalError::Runtime("索引必须是整数".to_string()))?;
                        // 获取数组的可变引用
                        let mut arr = arr_val.as_array().ok_or_else(|| EvalError::Runtime("索引赋值需要数组".to_string()))?.clone();
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(EvalError::Runtime(format!("索引越界: 长度 {}，索引 {}", arr.len(), idx)));
                        }
                        arr[idx as usize] = value;
                        // 写回环境（如果 arr_expr 是 Id）
                        if let Expr::Id(var_name, ..) = arr_expr.as_ref() {
                            self.environment.borrow_mut().assign(var_name, Value::Array(arr))
                                .map_err(EvalError::Runtime)?;
                            Ok(Value::Null)
                        } else {
                            Err(EvalError::Runtime("索引赋值暂不支持复杂左值".to_string()))
                        }
                    }
                    _ => Err(EvalError::Runtime("赋值左侧必须是变量或索引".to_string())),
                }
            }

            Stmt::VarDecl(vars, var_type) => {
                for var in vars {
                    let default_value = match var_type {
                        Type::Int => Value::Int(0),
                        Type::Float => Value::Float(0.0),
                        Type::Bool => Value::Bool(false),
                        Type::String => Value::String(String::new()),
                        Type::Array(_) => Value::Array(vec![]),
                    };
                    self.environment.borrow_mut().define(var, default_value);
                }
                Ok(Value::Null)
            }

            Stmt::Expr(expr) => self.eval_expr(expr),

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
                let new_env = Environment::new_with_parent(self.environment.clone());
                let mut block_eval = Evaluator::new(new_env, self.source.clone());
                block_eval.functions = self.functions.clone();
                block_eval.evaluate(stmts)
            }

            Stmt::Print(expr) => {
                let value = self.eval_expr(expr)?;
                println!("{}", value);
                Ok(Value::Null)
            }

            Stmt::For(var_name, start_expr, end_expr, body) => {
                let for_env = Environment::new_with_parent(self.environment.clone());
                let start_val = self.eval_expr(start_expr)?;
                let start_int = start_val.as_int()
                    .ok_or_else(|| EvalError::Runtime("for 循环起始值必须是整数".to_string()))?;
                for_env.borrow_mut().define(var_name, Value::Int(start_int));

                let mut child_eval = Evaluator::new(for_env.clone(), self.source.clone());
                child_eval.functions = self.functions.clone();
                
                let cond_expr = Expr::Op(
                    Opcode::LessOrEqual,
                    Box::new(Expr::Id(var_name.clone(), 0, 0)),
                    Box::new((**end_expr).clone()),
                );

                let update_stmt = Stmt::Assign(
                    Box::new(Expr::Id(var_name.clone(), 0, 0)),
                    Box::new(Expr::Op(
                        Opcode::Add,
                        Box::new(Expr::Id(var_name.clone(), 0, 0)),
                        Box::new(Expr::Num(1, 0, 0)),
                    )),
                );

                while child_eval.eval_expr(&cond_expr)?.is_truthy() {
                    child_eval.evaluate(body)?;
                    child_eval.execute(&update_stmt)?;
                }
                Ok(Value::Null)
            }

            Stmt::FuncDef(name, params, ret_type, body, _start, _end) => {
                self.functions.insert(name.clone(), (params.clone(), ret_type.clone(), body.clone(), *_start, *_end));
                Ok(Value::Null)
            }

            Stmt::Return(expr_opt, _start, _end) => {
                let value = match expr_opt {
                    Some(expr) => self.eval_expr(expr)?,
                    None => Value::Null,
                };
                Err(EvalError::Return(value))
            }
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Id(name, start, _) => {
                let value = self.environment.borrow()
                    .get(name)
                    .ok_or_else(|| {
                        let (line, col) = get_line_col(&self.source, *start);
                        EvalError::Runtime(format!("{}:{} 变量 '{}' 未定义", line, col, name))
                    })?;
                Ok(value)
            }

            Expr::Num(n, _, _) => Ok(Value::Int(*n as i64)),

            Expr::FloatLit(f, _, _) => Ok(Value::Float(*f)),

            Expr::StringLit(s, _, _) => Ok(Value::String(s.clone())),

            Expr::BoolLit(b, _, _) => Ok(Value::Bool(*b)),

            Expr::ArrayLit(elems, ..) => {
                let mut values = Vec::new();
                for e in elems {
                    values.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(values))
            }

            Expr::Index(arr_expr, idx_expr, ..) => {
                let arr_val = self.eval_expr(arr_expr)?;
                let idx_val = self.eval_expr(idx_expr)?;
                let arr = arr_val.as_array().ok_or_else(|| EvalError::Runtime("索引操作需要数组".to_string()))?;
                let idx = idx_val.as_int().ok_or_else(|| EvalError::Runtime("索引必须是整数".to_string()))?;
                if idx < 0 || idx as usize >= arr.len() {
                    return Err(EvalError::Runtime(format!("索引越界: 长度 {}，索引 {}", arr.len(), idx)));
                }
                Ok(arr[idx as usize].clone())
            }

            Expr::Op(op, left, right) => {
                match op {
                    Opcode::And => {
                        let left_val = self.eval_expr(left)?;
                        if !left_val.is_truthy() {
                            return Ok(Value::Bool(false));
                        }
                        let right_val = self.eval_expr(right)?;
                        Ok(Value::Bool(right_val.is_truthy()))
                    }
                    Opcode::Or => {
                        let left_val = self.eval_expr(left)?;
                        if left_val.is_truthy() {
                            return Ok(Value::Bool(true));
                        }
                        let right_val = self.eval_expr(right)?;
                        Ok(Value::Bool(right_val.is_truthy()))
                    }
                    _ => {
                        let left_val = self.eval_expr(left)?;
                        let right_val = self.eval_expr(right)?;
                        match op {
                            Opcode::Add => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Int(l_int + r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Float(lf + rf))
                                } else if let Some(l_str) = left_val.as_string() {
                                    let r_str = right_val.as_string()
                                        .map(|s| s.clone())
                                        .or_else(|| right_val.as_int().map(|i| i.to_string()))
                                        .or_else(|| right_val.as_float().map(|f| f.to_string()))
                                        .ok_or_else(|| EvalError::Runtime("加法：右操作数无法转换为字符串".to_string()))?;
                                    Ok(Value::String(format!("{}{}", l_str, r_str)))
                                } else if let Some(r_str) = right_val.as_string() {
                                    let l_str = left_val.as_int()
                                        .map(|i| i.to_string())
                                        .or_else(|| left_val.as_float().map(|f| f.to_string()))
                                        .ok_or_else(|| EvalError::Runtime("加法：左操作数无法转换为字符串".to_string()))?;
                                    Ok(Value::String(format!("{}{}", l_str, r_str)))
                                } else {
                                    Err(EvalError::Runtime("加法需要数字或字符串操作数".to_string()))
                                }
                            }
                            Opcode::Sub => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Int(l_int - r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Float(lf - rf))
                                } else {
                                    Err(EvalError::Runtime("减法需要数字操作数".to_string()))
                                }
                            }
                            Opcode::Mul => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Int(l_int * r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Float(lf * rf))
                                } else {
                                    Err(EvalError::Runtime("乘法需要数字操作数".to_string()))
                                }
                            }
                            Opcode::Div => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    if r_int == 0 {
                                        return Err(EvalError::Runtime("除数不能为零".to_string()));
                                    }
                                    Ok(Value::Int(l_int / r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    if rf == 0.0 {
                                        return Err(EvalError::Runtime("除数不能为零".to_string()));
                                    }
                                    Ok(Value::Float(lf / rf))
                                } else {
                                    Err(EvalError::Runtime("除法需要数字操作数".to_string()))
                                }
                            }
                            Opcode::GreaterThan => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Bool(l_int > r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Bool(lf > rf))
                                } else {
                                    Err(EvalError::Runtime("比较需要数字操作数".to_string()))
                                }
                            }
                            Opcode::GreaterOrEqual => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Bool(l_int >= r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Bool(lf >= rf))
                                } else {
                                    Err(EvalError::Runtime("比较需要数字操作数".to_string()))
                                }
                            }
                            Opcode::LessThan => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Bool(l_int < r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Bool(lf < rf))
                                } else {
                                    Err(EvalError::Runtime("比较需要数字操作数".to_string()))
                                }
                            }
                            Opcode::LessOrEqual => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Bool(l_int <= r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Bool(lf <= rf))
                                } else {
                                    Err(EvalError::Runtime("比较需要数字操作数".to_string()))
                                }
                            }
                            Opcode::Equal => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Bool(l_int == r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Bool(lf == rf))
                                } else if let (Some(l_bool), Some(r_bool)) = (left_val.as_bool(), right_val.as_bool()) {
                                    Ok(Value::Bool(l_bool == r_bool))
                                } else if let (Some(l_str), Some(r_str)) = (left_val.as_string(), right_val.as_string()) {
                                    Ok(Value::Bool(l_str == r_str))
                                } else {
                                    Err(EvalError::Runtime("相等比较需要相同类型的操作数".to_string()))
                                }
                            }
                            Opcode::NotEqual => {
                                if let (Some(l_int), Some(r_int)) = (left_val.as_int(), right_val.as_int()) {
                                    Ok(Value::Bool(l_int != r_int))
                                } else if let (Some(lf), Some(rf)) = (left_val.as_float(), right_val.as_float()) {
                                    Ok(Value::Bool(lf != rf))
                                } else if let (Some(l_bool), Some(r_bool)) = (left_val.as_bool(), right_val.as_bool()) {
                                    Ok(Value::Bool(l_bool != r_bool))
                                } else if let (Some(l_str), Some(r_str)) = (left_val.as_string(), right_val.as_string()) {
                                    Ok(Value::Bool(l_str != r_str))
                                } else {
                                    Err(EvalError::Runtime("不等比较需要相同类型的操作数".to_string()))
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }

            Expr::Not(expr) => {
                let val = self.eval_expr(expr)?;
                let b = val.as_bool().ok_or_else(|| EvalError::Runtime("逻辑非需要布尔操作数".to_string()))?;
                Ok(Value::Bool(!b))
            }

            Expr::Call(func_name, args, _start, _end) => {
                // 内置函数 len
                if func_name == "len" {
                    if args.len() != 1 {
                        return Err(EvalError::Runtime("len 需要 1 个参数".to_string()));
                    }
                    let arg = self.eval_expr(&args[0])?;
                    match arg {
                        Value::Array(v) => Ok(Value::Int(v.len() as i64)),
                        Value::String(s) => Ok(Value::Int(s.len() as i64)),
                        _ => Err(EvalError::Runtime("len 只支持数组或字符串".to_string())),
                    }
                } else {
                    // 用户自定义函数
                    let (params, _ret_type, body, ..) = self.functions.get(func_name)
                        .ok_or_else(|| EvalError::Runtime(format!("函数 '{}' 未定义", func_name)))?;
            
                    if args.len() != params.len() {
                        return Err(EvalError::Runtime(format!("函数 '{}' 需要 {} 个参数，提供了 {} 个",
                            func_name, params.len(), args.len())));
                    }
            
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg)?);
                    }
            
                    let func_env = Environment::new_with_parent(self.environment.clone());
                    {
                        let mut env_ref = func_env.borrow_mut();
                        for ((param_name, _param_type), arg_val) in params.iter().zip(arg_values) {
                            env_ref.define(param_name, arg_val);
                        }
                    }
            
                    // 如果之前实现了输出重定向，此处传递 self.output.clone()
                    let mut child_eval = Evaluator::new(func_env, self.source.clone() /* , self.output.clone() */);
                    child_eval.functions = self.functions.clone();
            
                    match child_eval.evaluate(body) {
                        Ok(v) => Ok(v),
                        Err(EvalError::Return(v)) => Ok(v),
                        Err(e) => Err(e),
                    }
                }
            }

            Expr::Neg(expr, _start, _end) => {
                let val = self.eval_expr(expr)?;
                if let Some(i) = val.as_int() {
                    Ok(Value::Int(-i))
                } else if let Some(f) = val.as_float() {
                    Ok(Value::Float(-f))
                } else {
                    Err(EvalError::Runtime("负数只能应用于数字".to_string()))
                }
            }
        }
    }

    pub fn dump_env(&self) {
        self.environment.borrow().dump();
    }

    pub fn dump_env_to_string(&self, output: &mut String) {
        self.environment.borrow().dump_to_string(output);
    }
}