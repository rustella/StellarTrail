//! 分页领域模型，提供 cursor 风格列表响应的通用结构。

use serde::{Deserialize, Serialize};

/// CursorPage 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

impl<T> CursorPage<T> {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new(items: Vec<T>, next_cursor: Option<String>) -> Self {
        Self { items, next_cursor }
    }
}
