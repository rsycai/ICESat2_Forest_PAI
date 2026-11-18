pub mod cli;
pub mod config;
pub mod domain;
pub mod interfaces;
pub mod pipeline;
pub mod providers;
pub mod services;
pub use crate::domain::{
    BoundingBox, DataGroup, DiscoveryResult, DownloadRange, Granule, GroupIdentity, PageToken,
    PipelineSource, RunOptions, SearchPage, SearchQuery, ShapefileCrs, SourceAdapter,
    SourceContext, TimeRange,
};
pub use crate::services::transport::{
    HttpRequest as CatalogRequest, HttpResponse as CatalogResponse,
};
pub use cli::params::{self, DownloadMode, DownloadParams};
pub use cli::{Cli, run};
pub use interfaces::{
    Authentication, AuthenticationProvider, DataSource, Discovery,
    HttpTransport as CatalogTransport, SourceProvider, StoragePolicy,
};
#[doc(hidden)]
pub use inventory as __inventory;
pub use xt_error::{Error, ErrorKind, Result};
pub use xt_transfer::{
    Asset, Checksum, DownloadFuture, DownloadObserver, DownloadOptions, DownloadOutcome,
    DownloadTask, ExistingFilePolicy, FileDownloader, RequestAuthorization,
};
