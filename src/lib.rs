use std::sync::Mutex;

use jni::{objects::{JObject}, sys::{jboolean, jstring, JNI_TRUE}, JNIEnv};
use mines::{location::Loc, mmap::MineMap};

static MM: Mutex<Option<MineMap>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn Java_me_dong_mines0522_Rsjni_init<'a>(
    _: JNIEnv<'a>,
    _: JObject<'a>,
    count: i32,
    width: i32,
    height: i32,
) -> jboolean {
    let Ok(mm) = MineMap::new(
        count as u16,
        width as u8,
        height as u8,
    ) else {
        panic!("init game fail !")
    };
    *MM.lock().unwrap() = Some(mm);
    JNI_TRUE
}

#[no_mangle]
pub extern "C" fn Java_me_dong_mines0522_Rsjni_msg<'a>(
    env: JNIEnv<'a>,
    _: JObject<'a>,
) -> jstring {
    // let inn = MM.lock().unwrap().as_mut();
    let output = match MM.lock().unwrap().as_mut() {
        Some(mm) => {
            mm.new_game(Some(Loc(1, 1)));
            env.new_string(mm.format_str()).unwrap()
        }
        _ => env.new_string(String::new()).unwrap()
    };
    output.into_raw()
}
