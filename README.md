# Comix 语言解释器

Comix 是一个用 Rust 编写的简单解释型语言，支持变量、算术运算、逻辑运算、条件判断和循环。
此项目正处于开发阶段，该文档将随开发进度实时更新。

<!-- ## 如何运行

### 1. 编译

确保已安装 Rust：https://rustup.rs/

```bash
git clone https://github.com/你的用户名/comix.git
cd comix
cargo build --release
```

### 2. 运行 REPL（交互式环境）

```bash
cargo run
```

然后在 `>` 提示符后输入代码，例如：

```text
> var x: int
> x = 42
42
> x * 2
84
```

### 3. 运行脚本文件

创建一个文件 `test.cmx`，内容：

```text
var a: int
a = 5 + 3
if a > 5 {
    print a
}
```

执行：

```bash
cargo run -- test.cmx
``` -->

## 语法示例

```text
var x: int
x = 10
var y: bool
y = x > 5
while y {
    x = x - 1
}
```

## 项目结构 (开发中)

- `src/lib.rs` – 组织文件，仅用于开发阶段
- `src/ast.rs` – 抽象语法树定义
- `src/value.rs` – 运行时值（整数、布尔、空）
- `src/environment.rs` – 变量作用域环境
- `src/grammar.lalrpop` - lalrpop解析器语法定义文件

- `src/evaluator.rs` – 解释执行逻辑（待开发）
- `src/main.rs` – REPL 和文件入口（待开发）

## 依赖

- [lalrpop](https://github.com/lalrpop/lalrpop) – 解析器生成器
<!-- - [anyhow](https://github.com/dtolnay/anyhow) – 错误处理 -->

## 许可证

MIT 或 Apache-2.0