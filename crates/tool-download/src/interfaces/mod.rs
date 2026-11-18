pub use http::HttpTransport;
pub use http::HttpTransport as CatalogTransport;
pub use xt_transfer::DownloadFuture;
pub use xt_transfer::DownloadObserver;
pub use xt_transfer::FileDownloader;
pub use xt_transfer::RequestAuthorization;

mod credentials;
mod discovery;
mod http;
mod provider;
mod source;
mod storage;
pub use credentials::Authentication;
pub use credentials::AuthenticationProvider;
pub use discovery::Discovery;
pub use provider::SourceProvider;
pub use source::DataSource;
pub use storage::StoragePolicy;
