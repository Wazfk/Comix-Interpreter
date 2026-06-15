#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    GreaterThan,
    GreaterOrEqual,
    Equal,
    NotEqual,
    LessOrEqual,
    LessThan,
    Add,
    Sub,
    Mul,
    Div, // Pow
    And,
    Or,
    // Xor
}

#[derive(Debug, Clone)]
pub enum Expr {
    Id(String, usize, usize),                   // 标识
    Num(u32, usize, usize),                     // 数字（整数）
    FloatLit(f64, usize, usize),                // 浮点数
    StringLit(String, usize, usize),            // 字符串字面量
    BoolLit(bool, usize, usize),                // 布尔字面量
    Op(Opcode, Box<Expr>, Box<Expr>),           // 二元运算
    Not(Box<Expr>),                             // 一元运算
    Neg(Box<Expr>, usize, usize),               // 负号
    Call(String, Vec<Expr>, usize, usize),      // 函数调用
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If(Box<Expr>, Vec<Stmt>),                      // if
    IfElse(Box<Expr>, Vec<Stmt>, Vec<Stmt>),       // if-else
    While(Box<Expr>, Vec<Stmt>),                   // while
    For(String, Box<Expr>, Box<Expr>, Vec<Stmt>),  // for
    Assign(String, Box<Expr>),                     // 赋值
    VarDecl(Vec<String>, Type),                    // 变量声明
    Block(Vec<Stmt>),                              // 语句块
    Expr(Box<Expr>),                               // 表达式语句
    Print(Box<Expr>),                              // 输出语句

    FuncDef(String, Vec<(String, Type)>, Type, Vec<Stmt>, usize, usize),    // 函数定义
    Return(Option<Box<Expr>>, usize, usize),    // return语句
}