# V-Craft Cross-Platform Support

> 参考 [bevy_platform](https://github.com/bevyengine/bevy/blob/main/crates/bevy_platform/README.md)。

Rust 标准库提供了三个层级：

- core: 基础的语言核心功能。
- alloc: \(额外包含\)内存分配相关功能与 `String` 等常用容器。
- std: \(额外包含\)文件、线程等操作系统 API。

理想状态下，游戏引擎面向的所有平台都支持 `core` ，兼容 `alloc` （可能需要提供内存分配器）。

`std` 包含操作系统接口，意味着每种平台都需要给出一套实现。
Rust 官方进行了大量工作以扩充多平台支持，但无法完全覆盖主机与嵌入式端。

> [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)

常见的解决方案是提供一个抽象层，包含所需的操作系统接口，并为各平台提供特定的实现。

这是一个庞大的工程，本库虽然定义了基础抽象层，却只能提供基于 `std` 的实现。

好消息是这足以支持 Win、Linux、Android 等多数平台的工作，已经能够满足 `demo` 项目的需求。
