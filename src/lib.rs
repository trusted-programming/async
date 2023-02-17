//!
//! # orion-async: 消除异步函数内部的本地变量必须支持Send Trait的约束，提升性能.
//! # orion-async: Eliminate this constraint - the local variables of asynchronous functions must implement Send Trait
//! 
//! 异步运行时框架的调度接口都会要求Future支持Send Trait,但现有编译器的实现中,
//! 如果异步函数内部使用了不支持Send Trait的变量类型,则其生成Future不支持Send.
//!
//! 理想情况下异步函数生成的Future是否支持Send Trait应该仅有输入参数类型来决定,
//! 在编译器未支持的情况下，可使用orion-async来解决，可以写更高效的异步代码.
//!
//! # Examples
//!
//! ```rust
//! use std::rc::Rc;
//! use std::future::Future;
//!
//! #[orion_async::future(body_send=true)]
//! async fn foo() {
//!     let val = Rc::new(100);
//!     bar(*val).await;
//! }
//! async fn bar(val: i32) {
//! }
//! fn test_send_future<T: Future + Send>(t: T) {
//! }
//!
//! test_send_future(foo());
//!
//! ```
//!  

use std::future::{ Future };
use std::task::{ Context, Poll };
use std::pin::Pin;

pub use orion_async_macros::{
    future
};

pub struct SendFuture<T: Future> {
    future: T,
}

unsafe impl<T: Future> Send for SendFuture<T> {}

impl<T: Future> SendFuture<T> {
    pub unsafe fn new(future: T) -> Self {
        Self { future: future }
    }
}

impl<T: Future> Future for SendFuture<T> {
    type Output = T::Output;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        Future::poll(unsafe { Pin::new_unchecked(&mut Pin::into_inner_unchecked(self).future) }, ctx)
    }
}

