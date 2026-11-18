use crate::hya::checkpoint;
use crate::hya::connector::DownloadConnector;
use crate::hya::errors::transfer_error;
use crate::hya::request::Transfer;
use crate::runtime::errors::cancelled;
use hya_core::{Capability, Scheduler, Source};
use hya_net::http::fetch_object;
use hya_net::polite::{Pace, RateLimiter};
use hya_net::sink::SparseSink;
use hya_net::transfer::run_transfer_cancellable;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use xt_error::{DownloadError, Error, ErrorKind, Result};

pub(crate) async fn download(
    job: Transfer,
    cancel: CancellationToken,
    progress: watch::Sender<(u64, Option<u64>)>,
) -> Result<()> {
    let (connector, mut target) =
        DownloadConnector::new(&job.url, &job.settings, &job.client).map_err(transfer_error)?;
    for (name, value) in &job.headers {
        let value = value
            .to_str()
            .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Authentication header is not valid text"))?;
        target.headers.push(format!("{name}: {value}"));
    }
    target.agent = Some(concat!("xtools/", env!("CARGO_PKG_VERSION")).into());
    let output = job
        .path
        .to_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidArgument, "Hya requires valid Unicode file paths"))?;
    let pace = job
        .settings
        .speed_limit
        .map(|rate| Pace::shared(Arc::new(RateLimiter::new(rate))))
        .unwrap_or_else(Pace::unlimited);
    let stop = Arc::new(AtomicBool::new(false));
    let rejected = connector.rejected.clone();
    let connector = Arc::new(connector);
    if job.ranges
        && let Some(size) = job.size
    {
        let workers = job.settings.workers_for(Some(size), true) as usize;
        let mut scheduler = Scheduler::new(
            size,
            vec![Source {
                caps: Capability::Full,
                ..Default::default()
            }],
            &[workers],
        )
        .with_stall_timeout(job.settings.timeout.as_secs_f64());
        let mut manifest = checkpoint::restore(
            &job.path,
            size,
            job.resume,
            &job.asset_key,
            job.validator.as_deref(),
            &mut scheduler,
        )?;
        let sink = Arc::new(SparseSink::create(output, size)?);
        let mut held = scheduler.held_ranges();
        let mut checkpoint_at = Instant::now();
        let mut checkpoint_error = None;
        let mut observe = |scheduler: &Scheduler, bytes: u64| {
            held = scheduler.held_ranges();
            let _ = progress.send((bytes, Some(size)));
            if checkpoint_at.elapsed() >= Duration::from_secs(1) {
                if let Err(error) = checkpoint::save(&job.path, &held, &mut manifest) {
                    checkpoint_error = Some(error);
                    stop.store(true, Ordering::Relaxed);
                }
                checkpoint_at = Instant::now();
            }
        };
        let counts = [workers];
        let mut download = Box::pin(run_transfer_cancellable(
            connector,
            vec![target],
            &counts,
            size,
            sink,
            scheduler,
            50,
            &mut observe,
            pace,
            Some(stop.clone()),
        ));
        let mut stopping = false;
        let result = loop {
            tokio::select! {
                result = &mut download => break result.map(|_| ()).map_err(transfer_error),
                _ = cancel.cancelled(), if !stopping => { stop.store(true, Ordering::Relaxed); stopping = true; },
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if rejected.load(Ordering::Relaxed) {
                        stop.store(true, Ordering::Relaxed);
                        break Err(Error::new(ErrorKind::InvalidArgument, "Download redirect exceeds authorized origins"));
                    }

                }
            }
        };
        drop(download);
        checkpoint::save(&job.path, &held, &mut manifest)?;
        if let Some(error) = checkpoint_error {
            return Err(error.into());
        }
        if cancel.is_cancelled() {
            return Err(cancelled());
        }
        result
    } else {
        let mut last_bytes = 0;
        let mut last_progress = Instant::now();
        let idle_timeout = job.settings.timeout
            + job
                .settings
                .speed_limit
                .map(|rate| Duration::from_secs(65536 / rate + 1))
                .unwrap_or_default();

        let written = AtomicU64::new(0);
        let pool = hya_net::Connector::pool(connector.as_ref());
        let mut download = Box::pin(fetch_object(
            connector.as_ref(),
            &target,
            output,
            &written,
            Some(&stop),
            &pace,
            pool.as_ref(),
        ));
        loop {
            tokio::select! {
                result = &mut download => break result.map(|_| ()).map_err(transfer_error),
                _ = cancel.cancelled() => { stop.store(true, Ordering::Relaxed); break Err(cancelled()); },
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    let bytes = written.load(Ordering::Relaxed);
                    let _ = progress.send((bytes, job.size));
                    if bytes != last_bytes { last_bytes = bytes; last_progress = Instant::now(); }
                    if last_progress.elapsed() >= idle_timeout {
                        break Err(Error::new(ErrorKind::Download(DownloadError::Timeout), "Hya download stalled"));
                    }
                }
            }
        }
    }
}
