# Comix 语言解释器

Comix 是一个用 Rust 编写的静态类型、解释型脚本语言，支持整数、浮点数、布尔、字符串类型，具有变量作用域、控制流、函数定义、递归调用等特性。此项目已实现较为完整的语言核心，并提供交互式 REPL、脚本文件执行以及图形界面（GUI）。

## 特性概览

- **基本类型**：整数 (`int`)、浮点数 (`float`)、布尔 (`bool`)、字符串 (`string`)
- **运算符**：算术（`+ - * /`）、比较（`= < > <= >= !=`）、逻辑（`and or not`）、负号（`-`）
- **变量声明与赋值**：`int x, y;` `x := 10;`
- **控制流**：`if`/`else`、`while`、`for`（`for var := start to end do { ... }`）
- **函数**：`func name(params) -> ret_type { ... }`，支持参数、返回值、递归、无返回值函数
- **内置输出**：`print expr;`
- **注释**：单行注释 `// ...`
- **错误报告**：解析和运行时错误包含行列号信息
- **REPL**：多行输入支持（行尾使用 `\` 续行），内置命令 `exit`、`clear`、`dump`（打印当前所有变量）
- **图形界面**：基于 egui 的简易 IDE（代码编辑、运行输出、变量监视）
- **脚本执行**：直接运行 `.comix` 文件

## 如何运行

### 1. 编译

确保已安装 Rust：https://rustup.rs/

```bash
git clone https://github.com/你的用户名/comix.git
cd comix
cargo build --release
```

### 2. 运行 REPL（交互式命令行）

```bash
cargo run
```

然后在 `>` 提示符后输入代码。例如：

```text
> int x;
> x := 42;
42
> x * 2;
84
> string s;
> s := "Hello " + "World";
Hello World
```

**多行输入**：行尾加 `\` 续行，例如：

```text
> func add(int a, int b) -> int { \
...     return a + b; \
... };
> print add(3, 5);
8
```

**REPL 命令**：
- `exit` – 退出
- `clear` – 清除所有变量（重置环境）
- `dump` – 打印当前所有变量及其值

### 3. 运行图形界面（IDE）

```bash
cargo run -- --gui
```

图形界面包含：
- **代码编辑器**：等宽字体，支持多行编辑，自动滚动。
- **Run 按钮**：执行当前编辑器中的代码。
- **输出区域**：显示执行结果、错误信息以及当前变量状态（只读，可滚动）。
- **布局**：编辑器占据大部分空间，输出区域固定在底部（可拖拽调整高度）。

### 4. 运行脚本文件

创建一个文件 `test.comix`，内容：

```text
// 计算阶乘
func factorial(int n) -> int {
    if n <= 1 then {
        return 1;
    } else {
        return n * factorial(n - 1);
    };
};

int result;
result := factorial(5);
print result;   // 输出 120
```

执行：

```bash
cargo run -- test.comix
```

## 语法详细示例

### 变量声明与赋值

```text
int a, b;           // 声明整数变量（默认值为 0）
float pi;           // 浮点数（默认 0.0）
bool flag;          // 布尔（默认 false）
string name;        // 字符串（默认空串）

a := 10;            // 赋值
pi := 3.14159;
flag := true;
name := "Comix";

// 注意：不支持声明时立即赋值，须分开写
```

### 表达式

```text
// 算术
a := 10 + 3 * 2;      // 16
b := (10 - 4) / 2;    // 3
c := -5;              // 负数

// 混合类型
float f;
f := 5 + 2.5;         // 7.5 (整数提升为浮点)

// 比较与逻辑
if a > 5 and not flag then {
    print "condition true";
};

if "hello" = "hello" then { print "equal"; };
if 5 != 3 then { print "not equal"; };
```

### 控制流

```text
// if-else
if x > 0 then {
    print x;
} else {
    print "non-positive";
};

// while 循环
int i;
i := 0;
while i < 5 do {
    print i;
    i := i + 1;
};

// for 循环（包含上界）
for j := 1 to 10 do {
    print j;
};
```

### 函数定义与调用

```text
// 带返回值的函数
func add(int a, int b) -> int {
    return a + b;
};

// 无返回值函数（返回 null）
func greet(string name) -> int {
    print "Hello, " + name;
};

// 递归
func fib(int n) -> int {
    if n <= 1 then {
        return n;
    } else {
        return fib(n-1) + fib(n-2);
    };
};

print fib(6);   // 8
```

### 作用域

```text
int outer;
outer := 10;
{
    int inner;
    inner := 20;
    outer := outer + inner;   // 可以访问外部变量
};
// print inner;   // 错误：inner 已超出作用域
print outer;      // 30
```

## 项目结构

```
src/
├── ast.rs               # 抽象语法树定义（Expr, Stmt, Type, Opcode）
├── value.rs             # 运行时值（Value 枚举及类型转换）
├── environment.rs       # 变量环境（支持嵌套作用域）
├── grammar.lalrpop      # LALRPOP 语法定义文件（词法、语法规则）
├── evaluator.rs         # 解释器执行逻辑（求值、错误处理）
├── main.rs              # 入口：REPL、文件执行、多行输入
└── gui.rs               # 图形界面模块（基于 egui）
```

## 错误处理示例

解析或运行时错误会输出具体位置：

```text
> int x;
> x := y + 1;
运行时错误: 2:7 变量 'y' 未定义
```

## 依赖

- [lalrpop](https://github.com/lalrpop/lalrpop) – 解析器生成器
- [lalrpop-util](https://crates.io/crates/lalrpop-util) – 运行时支持
- [anyhow](https://github.com/dtolnay/anyhow) – 错误处理
- [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) – GUI 框架

## 许可证

MIT 或 Apache-2.0