# Iron Doom

Iron Doom is a Doom source port developed entirely in Rust, focusing on code readability and preserving the essence of
the original game. Built on the Chocolate Doom codebase, the project aims to stay true to the classic experience while
providing a modern, stable implementation. By addressing and fixing many of the bugs in the original engine, Iron Doom
offers a more robust version of Vanilla Doom, all while maintaining its authentic feel.

![Iron Doom](https://github.com/user-attachments/assets/8dca0ae1-cc86-4fd1-bcf8-6c8a47ce854c)

## Project Architecture

Rust has some peculiarities that make it difficult to implement certain patterns commonly found in C/C++. Currently, the ECS model is the most developed and widely adopted by the Rust game development community, and it’s the model used in Iron Doom. The ECS implementation is provided by a module of the [Bevy engine](https://github.com/bevyengine/bevy), which, for those unfamiliar, is the most popular game engine in Rust at the moment.

## Demo Compatibility

Maintaining compatibility with old demos is a considerable challenge. While technically possible, changes made to improve code readability are likely to cause synchronization issues with demos. 

## Supported Platforms

| platform | is supported? |
|----------|---------------|
| Windows  | yes |
| Linux    | yes |
| MacOS    | yes |
