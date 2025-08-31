use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// 初始化 tracing 日志系统
///
/// # 参数
/// - `app_name`: 应用名称，用作日志文件前缀
/// - `log_dir`: 日志目录路径
/// - `level`: 日志级别（如 "info", "debug", "warn", "error"）
///
/// # 返回
/// 返回 WorkerGuard，必须保持存活直到程序结束，否则日志可能丢失
pub fn init_tracing(app_name: &str, log_dir: &str, level: &str) -> anyhow::Result<WorkerGuard> {
    // 创建日志目录
    std::fs::create_dir_all(log_dir)?;

    // 创建按日期滚动的文件 appender
    let file_appender = tracing_appender::rolling::daily(log_dir, app_name);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 创建环境过滤器
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    // 构建 subscriber
    // 支持同时输出到控制台和文件
    Registry::default()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false), // 文件输出不使用颜色
        )
        .init();

    tracing::info!("Tracing initialized for {}", app_name);
    Ok(guard)
}

/// 初始化带 span 追踪的 tracing 系统
/// 适用于需要分布式追踪的场景
/// FIXME: 这个函数暂时没有使用
#[allow(dead_code)]
pub fn init_tracing_with_spans(
    app_name: &str,
    log_dir: &str,
    level: &str,
) -> anyhow::Result<WorkerGuard> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, app_name);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    Registry::default()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .pretty(), // 控制台使用漂亮格式
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false)
                .json(), // 文件使用 JSON 格式，便于日志分析
        )
        .init();

    tracing::info!("Tracing with spans initialized for {}", app_name);
    Ok(guard)
}

/// 为库 crate 提供的简单初始化
/// 仅在没有全局 subscriber 时使用
pub fn init_default_if_none() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .try_init();
    });
}
