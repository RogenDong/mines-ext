# 参考文档
- https://docs.rs/jni/latest/jni/
- https://developer.android.google.cn/ndk/guides/abis?hl=zh-cn
- https://developer.android.google.cn/ndk/guides/abis?hl=zh-cn#gradle
- https://github.com/android/ndk-samples/tree/main/hello-jni/app/src/main
- https://rustmagazine.github.io/rust_magazine_2021/chapter_5/running_rust_on_android.html

### 查看过程宏结果
```
cargo install cargo-expand
cargo expand --test jnim
```
### 生产动态库文件
```
rustup target add aarch64-linux-android
cargo install cargo-ndk
cargo ndk -t arm64-v8a build --release
```