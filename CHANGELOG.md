# 开发日志 (Development Log)

## 2026-W4 (4月1日 - 4月7日)

### 本周概要
- 完成了解释器底层核心模块的搭建，包括运行时值系统、抽象语法树定义和变量环境。所有模块已通过单元测试，为下周的求值器开发奠定基础

### 新增
`value.rs` – 定义运行时值枚举 Value，支持 Int(i64)、Bool(bool)、Null 三种类型
实现方法：is_truthy()（用于条件判断）、as_int()、as_bool()、is_null()，并实现 Display trait 用于打印

`ast.rs` – 定义抽象语法树节点：
- Expr 枚举：包括标识符、数字字面量、二元运算、逻辑非
- Stmt 枚举：包括变量声明、赋值、if/if-else、while、代码块、表达式语句
- Opcode 枚举：算术、比较、逻辑运算符
- Type 枚举：Int、Bool（用于变量声明的类型注解）

`environment.rs` – 实现变量环境：
- 使用 Rc + RefCell 管理嵌套作用域
- 提供 define()、get()、assign() 方法，支持变量定义、查找和修改

### 修改
- 修复了开发中存在的BUG

### 计划
- 实现 evaluator.rs，遍历 AST 并执行语句
- 编写手动构造 AST 的集成测试，验证求值逻辑与环境的交互
- 开始设计 grammar.lalrpop 的语法规则草稿