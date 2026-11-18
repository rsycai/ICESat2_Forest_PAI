# xt-config

独立配置 crate，负责多格式文件加载、配置树展开、进程环境变量导出及子配置实例化。不依赖 xt-core 或 tracing。

工作区中添加依赖：

```toml
[dependencies]
xt-config.workspace = true
```

声明配置结构体：

```rust
use xt_config::EnvLoad;

#[derive(Debug, EnvLoad)]
#[env(prefix = "XT", section = "download")]
struct DownloadConfig {
    #[env(default = 4)]
    concurrency: usize,
}
```

Windows 启动时调用 `xt_config::envload!("config")?`，随后通过 `DownloadConfig::new()?` 自动读取 XT__DOWNLOAD__CONCURRENCY。已有环境变量跳过，缺失字段采用默认值，非法值返回错误。

根模块导出 EnvLoad、envload!、init_config、EnvConfig、ExportReport 和公共 Error/Result。非 Windows 平台的环境写入仍需调用方满足 unsafe 安全约束。

完整示例：`cargo run -p xt-config --example export_config -- examples/config`。

配置 crate 只依赖公共错误 crate xt-error、过程宏 crate xt-macros 和配置解析库。日志配置位于 xt-core，由日志模块自行定义和校验。

## 自动收集子配置

推荐在程序启动时直接调用 `xt_config::envload!("config.yaml")?`。
宏现在同时完成文件读取、环境变量导入、子配置加载校验和全局静态快照安装，
不再需要额外调用 AppConfig::from_path 或 new：

```rust
xt_config::envload!("config.yaml")?;
let logging = xt_config::get::<LogConfig>()?;
// 需要总配置入口时，取得同一个静态句柄。
let app = xt_config::AppConfig::global()?;
let same = app.get::<LogConfig>()?;
```

`envload!` 仍返回 ExportReport；成功后重复加载返回已初始化错误。
如果只需要导出环境变量而不初始化全局快照，使用 `init_config(path, "XT")`。

Windows 上可以直接读取文件并初始化全局配置，无需单独调用 envload!：

```rust
use xt_config::AppConfig;

let app = AppConfig::from_path("config.yaml")?;
// 也支持目录：AppConfig::from_path("config")?
let logging = app.get::<LogConfig>()?;
// 其他模块直接获取同一个配置快照。
let same = xt_config::get::<LogConfig>()?;
```

默认前缀为 XT；自定义前缀使用 `AppConfig::from_path_with_prefix(path, "MYAPP")`。
文件中的值先导入进程环境，已有环境变量仍优先，然后加载并校验全部子配置。
成功后再次调用文件入口返回“已初始化”错误，不重新读取或改动环境变量；
`AppConfig::new()` 仍可复用已有快照。文件读取失败可重试；子配置校验失败时
不安装快照，但已导入的环境变量仍保留。非 Windows 平台的文件入口为 unsafe，
沿用 init_config 的环境访问安全要求。

如果新增子配置时不希望修改总配置，使用 `Config` 和 `SubConfig`：

```rust
use xt_config::{AppConfig, EnvLoad, SubConfig};

#[derive(Debug, Clone, EnvLoad, SubConfig)]
#[env(prefix = "XT", section = "download")]
pub struct DownloadConfig {
    /// 并发下载数量，环境变量为 XT__DOWNLOAD__CONCURRENCY。
    #[env(default = 4)]
    pub concurrency: usize,
}

# fn main() -> xt_config::Result<()> {
// Windows：先调用 xt_config::envload!("config")? 导入文件配置。
let app = AppConfig::new()?;
let download = app.get::<DownloadConfig>()?;
assert_eq!(download.concurrency, 4);

// 其他模块可以直接按类型获取，无需引用 AppConfig。
let same = xt_config::get::<DownloadConfig>()?;
assert!(std::ptr::eq(download, same));
# Ok(())
# }
```

`SubConfig` 使用 inventory 登记加载函数；登记时不会读取环境变量。首次调用
`AppConfig::new()` 时，统一加载并校验已链接进当前程序的所有子配置。全部成功才
安装全局只读快照；失败不安装，修复配置后可以重试。重复初始化复用已有快照。
子配置须实现 `Send + Sync + 'static`，通常直接与 `EnvLoad` 一起派生即可。
同一规范化前缀和分组只能注册一个类型；不同前缀可使用相同分组。

新增子配置只需在对应模块添加结构体和派生声明，无需注册列表。模块仍须通过
`mod` 声明、所属 crate 仍须作为依赖链接到程序；磁盘上未参与编译的文件不会被发现。
总配置通过 `get::<T>()` 访问，不生成实际字段，也不支持 `app.download`。
同一进程中的多个 `Config` 标记结构体共享这一个全局快照。

`AppConfig` 已内置于 xt-config，通常直接导入即可，无需在应用中重复定义。
如果希望使用自己的总配置名称，仍可对空结构体派生 `xt_config::Config`。

`DownloadConfig::new()` 会先确保全局配置已加载，再读取环境变量生成新实例；`app.get::<DownloadConfig>()` 和
`xt_config::get::<DownloadConfig>()` 返回启动时保存的引用。需要副本时对引用调用
`.clone()`。请先完成文件导入再初始化快照；之后的环境变量变化不更新快照。
自定义 `new()` 或 `validate()` 不应重入总配置初始化，也不应在初始化时读取全局配置。

需要独立快照而非全局配置时，可以使用 `ConfigRegistry::load()?.get::<T>()`。
如果将 xt-config 依赖改名，子配置沿用 `#[env(crate_path = "别名", ...)]`，
总配置使用 `#[config(crate_path = "别名")]`。

## 默认位置与按需加载

Windows 和 Linux 均支持自动加载。未调用 envload! 时，子配置的 `new()`、
`xt_config::get::<T>()` 和 `AppConfig::global()` 会从当前工作目录的 `config.yaml`
初始化全局配置。可以通过进程环境变量 `XT_CONFIG_PATH` 指定其他文件或目录。
自动加载的环境名称前缀为 XT，优先级为真实环境变量（包括兼容别名）、文件值、字段默认值。

如果默认路径不存在，会向 stderr 输出一行“未找到配置文件，使用默认值运行”，
随后用已有环境变量和字段默认值安装全局快照，不再报“全局配置未加载”。
成功安装后不重复查找或提示。配置文件存在但格式错误、字段值无效或校验失败仍返回错误。
显式调用 envload! 或 from_path 时，传入路径不存在仍返回错误。

```rust
// 无需先调用 envload!；缺少默认文件也可以按默认值运行。
let logging = LogConfig::new()?;
let shared = xt_config::get::<LogConfig>()?;
let app = xt_config::AppConfig::global()?;
// 只读取真实环境变量，不查找文件，也不初始化全局配置。
let env_only = LogConfig::load_from_env()?;
```

自动加载将文件映射保存于进程内存，不修改系统环境变量，因此 Linux 上可以安全地
在首次访问时执行。显式 envload! 仍保留原有的环境变量导出行为，在非 Windows 平台
需要满足其 unsafe 环境访问条件。

EnvLoad 的 `new()` 不要求 Clone，它在确保加载后使用环境变量及内存文件源构造新实例；
需要稳定的启动快照请用 `get::<T>()`。注册器加载期间会跳过自动查找，避免递归初始化。