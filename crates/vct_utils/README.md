# V-Craft Utilities

> 参考 [bevy_platform](https://github.com/bevyengine/bevy/blob/main/crates/bevy_platform/README.md) 与 [bevy_utils](https://github.com/bevyengine/bevy/blob/main/crates/bevy_utils/README.md)。

本库提供一些常用容器和工具：

- `OnDrop`: 在 `drop` 时自动调用函数的容器。
- `parallel_queue`: 多线程队列，使用线程局部存储。
- `OnceFlag`: 一次性标志位。
- `DebugName`: 调试环境中存储类型名的类型。
- cell:
    - `sync_cell`: 满足 `sync` 的类 `cell` 容器。
    - `sync_unsafe_cell`: `sync` 版本的 `unsafe_cell`。
- collections:
    - `HashMap`: 基于 hashbrown 库的实现。
    - `HashSet`: 基于 hashbrown 库的实现。
    - `HashTable`: 基于 hashbrown 库的实现。
    - `BTreeMap`: alloc 中的容器
    - `BTreeSet`: alloc 中的容器
    - `BinaryHeap`: alloc 中的容器
    - `LinkedList`: alloc 中的容器
    - `VecDeque`: alloc 中的容器
- `PreHashMap`: 预计算好键的 `HashMap`。
- `TypeIdMap`: 键为 `TypeId` 的 `HashMap`。

库默认启用 `std` 支持，大部分容器至少需要 `alloc` 支持。
