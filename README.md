# orion-async

Eliminate this constraint for performance - the local variables of asynchronous functions must implement Send Trait

消除异步函数内部的本地变量必须支持Send Trait的约束, 提升性能.

正常情况下，下面的代码无法工作.

```rust
use std::rc::Rc;

#[tokio::main]
async fn main() {
    tokio::spawn(foo()).await;
}

async fn foo() {
    let id = Rc::new(100);
    tokio::spawn(bar(*id)).await; 
}

async fn bar(id: i32) {
    println!("bar( {} )", id);
}

```
以上代码编译会报告错误:

```bash
error: future cannot be sent between threads safely
   --> src/main.rs:5:18
    |
5   |     tokio::spawn(foo()).await;
    |                  ^^^^^ future returned by `foo` is not `Send`
    |
    = help: within `impl Future<Output = ()>`, the trait `Send` is not implemented for `Rc<i32>`
```

仅仅在异步函数内部使用的Rc不会出现任何并发访问的场景。理想情况下，应该只需要异步函数的输入参数支持Send Trait即可，而不应该对异步函数内部使用变量类型有任何约束。

如果用Arc来替代Rc，牺牲的是性能。orion_async提供此场景下兼顾安全和性能的解决方案, 只需要如下为异步函数foo增加orion_async::future过程宏定义即可.

```rust
use std::rc::Rc;

#[tokio::main]
async fn main() {
    tokio::spawn(foo()).await;
}

#[orion_async::future(body_send = true)]
async fn foo() {
    let id = Rc::new(100);
    tokio::spawn(bar(*id)).await; 
}

async fn bar(id: i32) {
    println!("bar( {} )", id);
}

```
# 使用方法

此过程宏只能作用于异步函数，且只能消除异步函数内部变量可使用不支持Send Trait的数据类型的约束，不会改变函数输入参数的任何约束。

| 宏名称 | 属性名 | 属性值类型 | 缺省值 |
| --- | --- | ---| --- |
| future ||||
|| body_send | bool | false|

