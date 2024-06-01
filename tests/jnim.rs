use jni::sys::jboolean;
use jni::{objects::JObject, JNIEnv};
use jni_macro::jni;

#[test]
fn tss() {
    #[jni]
    fn dong(_: JNIEnv, _: JObject) -> jboolean {
        1
    }
    #[jni_macro::jni("me.dong.mines0522")]
    mod m0 {
        use jni::{objects::JObject, JNIEnv};
        #[jni_macro::jni]
        fn go(_: JNIEnv, _: JObject) {}
        #[jni("Rsjni")]
        fn init(_: JNIEnv, _: JObject) {}
    }
    mod m1 {
        use jni::{objects::JClass, JNIEnv};
        use jni_macro::jni;
        #[jni("CCC")]
        fn dong(_: JNIEnv, _: JClass) {}
    }
    #[jni("me.dong.DDD")]
    mod m2 {
        use jni::{objects::JObject, JNIEnv};
        #[jni]
        fn go(_: JNIEnv, _: JObject) {}
        #[jni("Rsjni")]
        fn init(_: JNIEnv, _: JObject) {}
    }

    println!("123")
}
