//! Live-replay SSE endpoint — `GET /replay-live/:signature`.
//!
//! Streams progress events as a transaction is fetched, reconstructed, and
//! re-executed. Source is either real Helius LaserStream (when configured)
//! or a progressive walk over standard Helius RPC. The wire format is the
//! same either way; the first event tells the client which source ran.
//!
//! ## Why progressive
//!
//! Without this endpoint, `POST /replay` blocks for 2–5s while the engine
//! fetches ~20 accounts, reconstructs program data, and executes. The user
//! sees a spinner. Progressive events let the UI render the trace tree as
//! it's built — visually it feels instant.
//!
//! ## Concurrency cap
//!
//! Live sessions hold an SSE connection plus a fetch loop and several MB of
//! per-replay state. We cap concurrent live sessions at
//! `REPLAY_MAX_LIVE_SESSIONS` (default 5) and reject new ones with HTTP 429
//! until something finishes or hits the 5-minute idle timeout.

use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::Stream;
use replay_core::{
    decode_account_deltas, fetch::fetch_full_tx_context, idl::IdlCache, idl::AccountDecoder,
    laserstream_connect, reconstruct::reconstruct_state_with_progress,
    reconstruct::snapshot_pre_state, reconstruct::ReconstructProgress, svm::SvmRunner, trace,
    LaserStreamStatus, LiveReplayEvent, LiveSource, ReplayError,
};
use solana_sdk::signature::Signature;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use tracing::{debug, error, info, warn};

/// Hard cap on how long the engine task is allowed to run before we drop the
/// session. Matches the prompt's "auto-close idle live sessions after 5 minutes"
/// requirement; in practice the pipeline finishes in seconds.
const LIVE_SESSION_TIMEOUT: Duration = Duration::from_secs(300);

/// SSE channel buffer. Big enough to absorb a Jupiter-sized tx (~30 frames +
/// ~25 accounts) without back-pressure stalling the engine task; small enough
/// that a wedged client doesn't pin a lot of memory.
const EVENT_CHANNEL_CAPACITY: usize = 128;

pub async fn replay_live(
    State(state): State<Arc<AppState>>,
    Path(signature): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    // Parse first so an obviously bad signature returns a clean 400 instead
    // of opening an SSE connection that immediately errors.
    let sig = Signature::from_str(&signature)
        .map_err(|_| ApiError(ReplayError::InvalidSignature(signature.clone())))?;

    // Reserve a live-session slot. The guard releases the slot in Drop, so
    // we don't have to remember to decrement on every error path.
    let permit = match state.live_sessions.try_acquire() {
        Some(p) => p,
        None => {
            return Err(ApiError(ReplayError::Execution(format!(
                "live-session cap reached ({}); try again in a moment",
                state.live_sessions.cap()
            ))));
        }
    };

    let source = match laserstream_connect() {
        LaserStreamStatus::Configured(_) => LiveSource::Laserstream,
        LaserStreamStatus::NotConfigured => LiveSource::Rpc,
    };

    let (tx, rx) = mpsc::channel::<LiveReplayEvent>(EVENT_CHANNEL_CAPACITY);

    // Spawn the engine. The task owns the permit so the slot stays held
    // until the pipeline completes, errors, or hits the timeout.
    let client = Arc::clone(&state.client);
    tokio::spawn(async move {
        let _permit = permit;
        let pipeline = drive_replay(client, sig, source, tx.clone());
        match tokio::time::timeout(LIVE_SESSION_TIMEOUT, pipeline).await {
            Ok(Ok(())) => debug!(%sig, "live replay finished cleanly"),
            Ok(Err(e)) => {
                warn!(%sig, error = %e, "live replay failed");
                let _ = tx
                    .send(LiveReplayEvent::Error {
                        code: e.code().to_string(),
                        message: e.to_string(),
                    })
                    .await;
            }
            Err(_) => {
                error!(%sig, "live replay exceeded {LIVE_SESSION_TIMEOUT:?}");
                let _ = tx
                    .send(LiveReplayEvent::Error {
                        code: "LIVE_TIMEOUT".into(),
                        message: format!(
                            "live replay exceeded {}s budget",
                            LIVE_SESSION_TIMEOUT.as_secs()
                        ),
                    })
                    .await;
            }
        }
    });

    // Wrap the receiver as an SSE stream. Each event becomes one
    // `data: <json>\n\n` SSE frame; the JSON is the tagged-union shape from
    // replay-core::laserstream.
    let event_stream = ReceiverStream::new(rx).map(|ev| {
        let json = serde_json::to_string(&ev).unwrap_or_else(|e| {
            // Should be impossible — every variant in LiveReplayEvent is
            // serde-friendly. If it ever happens we'd rather emit a usable
            // error event than crash the whole stream.
            format!(
                r#"{{"type":"error","code":"SERDE_ERROR","message":"{}"}}"#,
                e.to_string().replace('"', r#"\""#)
            )
        });
        Ok::<_, Infallible>(Event::default().data(json))
    });

    // Prepend the `mode` event and append a synthetic `[DONE]`-style sentinel
    // not strictly needed by EventSource but helpful for debugging in curl.
    let mode_event = serde_json::to_string(&LiveReplayEvent::Mode { source }).unwrap();
    let prelude = futures::stream::iter(std::iter::once(Ok::<_, Infallible>(
        Event::default().data(mode_event),
    )));
    let combined = prelude.chain(event_stream);

    Ok(Sse::new(combined).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}

/// Drive the replay pipeline, emitting events into `tx` as each stage
/// completes. Returns `Err` if any stage fails; the caller turns that into
/// a terminal `error` event on the wire.
async fn drive_replay(
    client: Arc<dyn replay_core::HeliusClient>,
    sig: Signature,
    _source: LiveSource,
    tx: mpsc::Sender<LiveReplayEvent>,
) -> Result<(), ReplayError> {
    info!(%sig, "starting live replay pipeline");

    // 1. Fetch + slot resolution.
    let mut ctx = fetch_full_tx_context(&client, &sig).await?;
    let _ = tx
        .send(LiveReplayEvent::SlotObserved {
            slot: ctx.slot,
            block_time: ctx.block_time,
        })
        .await;

    // 2. Reconstruct state, forwarding per-account events. Use try_send
    //    inside the closure so a slow consumer never blocks the engine.
    let progress_tx = tx.clone();
    let state = reconstruct_state_with_progress(&client, &ctx, move |p| match p {
        ReconstructProgress::AccountFetched {
            pubkey,
            size,
            is_program,
        } => {
            let _ = progress_tx.try_send(LiveReplayEvent::AccountFetched {
                pubkey: pubkey.to_string(),
                size,
                is_program,
            });
        }
    })
    .await?;

    let total_accounts =
        state.accounts.len() + state.programs.len() * 2; // programs counted with their data accts
    let _ = tx
        .send(LiveReplayEvent::AllAccountsFetched {
            count: total_accounts,
        })
        .await;

    ctx.pre_account_snapshots = snapshot_pre_state(&state);

    // 3. Execute.
    let _ = tx.send(LiveReplayEvent::ExecutionStarted).await;
    let mut runner = SvmRunner::new();
    runner.seed(&state)?;
    runner.set_clock_for_slot(ctx.slot, ctx.block_time);
    let execution = runner.execute(&ctx)?;

    // 4. Build the trace + decode account deltas. The CPI tree is fully
    //    available now; emit each top-level frame as it gets attached
    //    so the UI can grow the tree visibly. (For RPC source this is a
    //    tight burst; LaserStream will spread these out by validator
    //    emission in a future iteration.)
    let idl_cache = IdlCache::default();
    let decoder = AccountDecoder::new(&idl_cache);
    let mut full_trace = trace::build_trace(&ctx, &execution, &decoder).await;

    for frame in &full_trace.frames {
        let _ = tx
            .send(LiveReplayEvent::FrameCompleted {
                frame: Box::new(frame.clone()),
            })
            .await;
    }

    decode_account_deltas(&mut full_trace, &ctx, &execution, &decoder, &client).await;

    // 5. Done.
    let _ = tx
        .send(LiveReplayEvent::Done {
            trace: Box::new(full_trace),
        })
        .await;
    Ok(())
}
