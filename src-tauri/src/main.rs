// Tauri 桌面入口。
// 桌面端把工作全部委托给 lib.rs 的 run()，方便后续做移动端或测试时复用同一份代码。

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    mmcode_toolbox_lib::run();
}