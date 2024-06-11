//! 配置日志追踪

use log::{debug, error, info, trace, warn};
use time::format_description::{self, FormatItem};
use time::UtcOffset;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    fmt,
    fmt::{time::OffsetTime, writer::MakeWriterExt},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

const LOG_DIR: &str = "/sdcard/Android/data/me.dong.mines/logs";
const F_PFX_NOR: &str = "mmrs.log";
const FMT: &str = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";

/// 配置时区和时间格式
fn get_timer(t_fmt: Vec<FormatItem>) -> OffsetTime<Vec<FormatItem>> {
    match UtcOffset::from_hms(8, 0, 0) {
        Ok(ofs) => OffsetTime::new(ofs, t_fmt),
        Err(e) => {
            error!("配置时区异常！{:#?}", e);
            panic!("配置时区异常！");
        }
    }
}

/// 初始化日志
pub fn init_tracing() -> WorkerGuard {
    debug!("描述格式...");
    let t_fmt1 = match format_description::parse(FMT) {
        Err(e) => {
            error!("time::format_description::parse fail: {e:#?}");
            panic!("init tracing fail")
        }
        Ok(f) => f,
    };

    // 日志文件。日志文件不上色（with_ansi(false)）
    // normal.log: INFO < 等级 < WARN
    debug!("创建非阻塞、脱线程写入器...");
    // dir.push_str(LOG_DIR);
    let (ff, nl_guard) = tracing_appender::non_blocking(rolling::never(LOG_DIR, F_PFX_NOR));

    debug!("创建格式化层...");
    let f_normal = fmt::layer()
        .with_ansi(false)
        .with_writer(ff.with_min_level(Level::WARN).with_max_level(Level::INFO));

    debug!("设置时区...");
    let timer = get_timer(t_fmt1);
    let f_normal = f_normal.with_timer(timer);

    // 注册
    debug!("注册...");
    tracing_subscriber::registry().with(f_normal).init();

    trace!("tracing ready.");
    debug!("tracing ready.");
    info!("tracing ready.");
    warn!("tracing ready.");
    error!("tracing ready.");

    nl_guard
}
