- https://docs.rs/jni/latest/jni/
- https://developer.android.google.cn/ndk/guides/abis?hl=zh-cn
- https://developer.android.google.cn/ndk/guides/abis?hl=zh-cn#gradle
- https://github.com/android/ndk-samples/tree/main/hello-jni/app/src/main
- https://rustmagazine.github.io/rust_magazine_2021/chapter_5/running_rust_on_android.html

`rustup target add aarch64-linux-android`

`cargo ndk -t arm64-v8a build --release`