use std::sync::Mutex;

use jni::{
    objects::JObject,
    sys::{jboolean, jstring, JNI_FALSE, JNI_TRUE},
    JNIEnv,
};
use jni_macro::jni;
use log::{debug, error, info, trace, warn};
use mines::{location::Loc, mmap::MineMap};

static MM: Mutex<Option<MineMap>> = Mutex::new(None);

fn log_init() {
    let logger = android_logd_logger::builder()
        .parse_filters("debug")
        .tag("jni-log-init")
        .prepend_module(true)
        .init();

    trace!("trace message");
    debug!("debug message");
    info!("info message");
    warn!("warn message");
    error!("error message");

    logger.tag("mines-jni");
}

#[jni("me.dong.mines0531.Rsjni")]
fn init(_: JNIEnv, _: JObject, count: i32, width: i32, height: i32) -> jboolean {
    log_init();
    let Ok(mm) = MineMap::new(count as u16, width as u8, height as u8) else {
        error!("create mines map fail !");
        return JNI_FALSE;
    };
    info!("create mines map ok");
    *MM.lock().unwrap() = Some(mm);
    JNI_TRUE
}

#[jni("me.dong.mines0531.Rsjni")]
fn msg(env: JNIEnv, _: JObject) -> jstring {
    let output = match MM.lock().unwrap().as_mut() {
        Some(mm) => {
            mm.new_game(Some(Loc(1, 1)));
            env.new_string(mm.format_str()).unwrap()
        }
        _ => env.new_string(String::new()).unwrap(),
    };
    output.into_raw()
}
