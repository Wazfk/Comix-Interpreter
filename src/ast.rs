#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    GreaterThan,
    Equal,
    LessThan,
    Add,
    Sub,
    Mul,
    // Div, Pow
    And,
    Or,
    // Xor
}

#[derive(Debug, Clone)]
pub enum Expr {
    Id(String, usize, usize),           // 标识
    Num(u32),                           // 数字
    Op(Opcode, Box<Expr>, Box<Expr>),   // 二元运算
    Not(Box<Expr>),                     // 一元运算
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If(Box<Expr>, Vec<Stmt>),                      // if
    IfElse(Box<Expr>, Vec<Stmt>, Vec<Stmt>),       // if-else
    While(Box<Expr>, Vec<Stmt>),                   // while
    // For(Box<Expr>, Box<Expr>, Box<Expr>, VEc<STmt>)
    Assign(String, Box<Expr>),                     // 赋值
    VarDecl(Vec<String>, Type),                    // 变量声明 (ps: 此处设置影响报错，但目前为了简化暂不添加位置信息)
    Block(Vec<Stmt>),                              // 语句块
    Expr(Box<Expr>),                               // 表达式语句
}