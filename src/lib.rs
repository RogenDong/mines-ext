// pub use game::*;

mod tracing_rs;

#[jni_macro::jni]
mod game {

    use std::sync::Mutex;

    use jni::{
        objects::JObject,
        sys::{jboolean, jbyteArray, jint, jstring, JNI_FALSE, JNI_TRUE},
        JNIEnv,
    };
    use log::{debug, error, info, trace, warn};
    use mines::{location::Loc, mmap::MineMap};
    use tracing_appender::non_blocking::WorkerGuard;

    const MSG_GET_MAP_FAIL: &str = "地图数据异常！";

    static MM: Mutex<Option<MineMap>> = Mutex::new(None);
    static TL: Mutex<Option<WorkerGuard>> = Mutex::new(None);

    /// 初始化日志
    #[jni]
    fn initLogger(_: JNIEnv, _: JObject) -> jboolean {
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

        // let ok = match env.get_string(&dir) {
        //     Ok(d) => {
        //     }
        //     Err(e) => {
        //         error!("get jni string fail! {e:#?}");
        //         JNI_FALSE
        //     }
        // };
        *TL.lock().unwrap() = Some(crate::tracing_rs::init_tracing());

        logger.tag("mines-jni");
        JNI_TRUE
    }

    #[jni]
    fn init(_: JNIEnv, _: JObject, count: jint, width: jint, height: jint) -> jboolean {
        let Ok(mm) = MineMap::new(count as u16, width as u8, height as u8) else {
            error!("{}", MSG_GET_MAP_FAIL);
            return JNI_FALSE;
        };
        info!("create mines map ok");
        *MM.lock().unwrap() = Some(mm);
        JNI_TRUE
    }

    #[jni]
    fn newGame(_: JNIEnv, _: JObject, x: jint, y: jint) {
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        if x < 0 || y < 0 {
            mm.new_game(None)
        } else {
            mm.new_game(Some(Loc(x as u8, y as u8)))
        }
    }

    #[jni]
    fn resetProgress(_: JNIEnv, _: JObject) {
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        mm.reset_progress()
    }

    #[jni]
    fn switchFlag(_: JNIEnv, _: JObject, x: jint, y: jint) {
        if x < 0 || y < 0 {
            warn!("无效坐标：({x},{y})");
            return;
        }
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        mm.switch_flag(x as usize, y as usize)
    }

    #[jni]
    fn reveal(_: JNIEnv, _: JObject, x: jint, y: jint) -> jint {
        if x < 0 || y < 0 {
            warn!("无效坐标：({x},{y})");
            return 0;
        }
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        mm.reveal(x as usize, y as usize) as jint
    }

    #[jni]
    fn revealAround(_: JNIEnv, _: JObject, x: jint, y: jint) -> jint {
        if x < 0 || y < 0 {
            warn!("无效坐标：({x},{y})");
            return 0;
        }
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        mm.reveal_around(x as usize, y as usize) as jint
    }

    #[jni]
    fn countFlaggedAround(_: JNIEnv, _: JObject, x: jint, y: jint) -> jint {
        if x < 0 || y < 0 {
            warn!("无效坐标：({x},{y})");
            return 0;
        }
        let mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_ref().expect(MSG_GET_MAP_FAIL);
        mm.count_flagged_around(x as usize, y as usize) as jint
    }

    #[jni]
    fn revealAllMines(_: JNIEnv, _: JObject) {
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        mm.reveal_all_mines()
    }

    #[jni]
    fn isAllReveal(_: JNIEnv, _: JObject) -> jboolean {
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        if mm.is_all_reveal() {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    }

    #[jni]
    fn countFlagged(_: JNIEnv, _: JObject) -> jint {
        let mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_ref().expect(MSG_GET_MAP_FAIL);
        mm.count_flagged() as jint
    }

    #[jni]
    fn formatString(env: JNIEnv, _: JObject) -> jstring {
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        env.new_string(mm.format_str()).unwrap().into_raw()
    }

    #[jni]
    fn formatStatusString(env: JNIEnv, _: JObject) -> jstring {
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        env.new_string(mm.format_stat_str()).unwrap().into_raw()
    }

    /// 导出布局数据
    /// # Argument
    /// - `hold_stat` 是否保留状态
    /// # Returns
    /// - `[宽width, 高height, 数据data..]`
    #[jni]
    fn fetch(env: JNIEnv, _: JObject, hold_stat: jboolean) -> jbyteArray {
        let hold = hold_stat == JNI_TRUE;
        let mut mm = MM.lock().expect(MSG_GET_MAP_FAIL);
        let mm = mm.as_mut().expect(MSG_GET_MAP_FAIL);
        if let Ok(arr) = env.byte_array_from_slice(&mm.export(hold)) {
            return **arr;
        }
        **env.new_byte_array(0).unwrap()
    }
}
