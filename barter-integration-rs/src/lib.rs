//! # Barter-Integration
//! High-performance, low-level framework for composing flexible web integrations.
//!
//! Utilised by other Barter trading ecosystem crates to build robust financial exchange integrations,
//! primarily for public data collection & trade execution. It is:
//! * **Low-Level**: Translates raw data streams communicated over the web into any desired data model using arbitrary data transformations.
//! * **Flexible**: Compatible with any protocol (WebSocket, FIX, Http, etc.), any input/output model, and any user defined transformations.
//!
//! ## Core abstractions:
//! - **RestClient** providing configurable signed Http communication between client & server.
//! - **ExchangeStream** providing configurable communication over any asynchronous stream protocols (WebSocket, FIX, etc.).
//!
//! Both core abstractions provide the robust glue you need to conveniently translate between server & client data models.

#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms
)]

use crate::{
    error::SocketError,
    protocol::{flat_files, flat_files::BacktestMode, StreamParser},
};
use futures::Stream;
use pin_project::pin_project;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
// use tokio_tungstenite::tungstenite::Message;

/// Foundational data structures that define the building blocks used by the rest of the `Barter`
/// ecosystem.
///
/// eg/ `Market`, `Exchange`, `Instrument`, `Symbol`, etc.
pub mod model;

/// All [`Error`](std::error::Error)s generated in Barter-Integration.
pub mod error;

/// Contains `StreamParser` implementations for transforming communication protocol specific
/// messages into a generic output data structure.
pub mod protocol;

/// Contains the flexible `Metric` type used for representing real-time metrics generically.
pub mod metric;

/// Utilities to assist deserialisation.
pub mod de;

/// [`Validator`]s are capable of determining if their internal state is satisfactory to fulfill
/// some use case defined by the implementor.
pub trait Validator {
    /// Check if `Self` is valid for some use case.
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized;
}

/// [`Transformer`]s are capable of transforming any `Input` into an iterator of
/// `Result<Self::Output, Self::Error>`s.
pub trait Transformer {
    type Error;
    type Input: for<'de> Deserialize<'de>;
    type Output;
    type OutputIter: IntoIterator<Item = Result<Self::Output, Self::Error>>;
    fn transform(&mut self, input: Self::Input) -> Self::OutputIter;
}

/// An [`ExchangeStream`] is a communication protocol agnostic [`Stream`]. It polls protocol
/// messages from the inner [`Stream`], and transforms them into the desired output data structure.
#[derive(Debug)]
#[pin_project]
pub struct ExchangeStream<Protocol, InnerStream, StreamTransformer>
where
    Protocol: StreamParser,
    InnerStream: Stream,
    StreamTransformer: Transformer,
{
    #[pin]
    pub stream: InnerStream,
    pub transformer: StreamTransformer,
    pub buffer: VecDeque<Result<StreamTransformer::Output, StreamTransformer::Error>>,
    pub protocol_marker: PhantomData<Protocol>,
    pub backtest_mode: BacktestMode,
}

impl<Protocol, InnerStream, StreamTransformer> Stream
    for ExchangeStream<Protocol, InnerStream, StreamTransformer>
where
    Protocol: StreamParser,
    InnerStream: Stream<Item = Result<Protocol::Message, Protocol::Error>> + Unpin,
    StreamTransformer: Transformer,
    StreamTransformer::Error: From<SocketError>,
{
    type Item = Result<StreamTransformer::Output, StreamTransformer::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Flush Self::Item buffer if it is not currently empty
            if let Some(output) = self.buffer.pop_front() {
                return Poll::Ready(Some(output));
            }

            // Poll inner `Stream` for next the next input protocol message
            let input = match self.as_mut().project().stream.poll_next(cx) {
                Poll::Ready(Some(input)) => input,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            // TODO: is it optimal to clone the string here?
            let raw_msg = Protocol::get_msg_contents(&input).map(|s| s.clone());

            // Parse input protocol message into `ExchangeMessage`
            let exchange_message = match Protocol::parse::<StreamTransformer::Input>(input) {
                // `StreamParser` successfully deserialised `ExchangeMessage`
                Some(Ok(exchange_message)) => exchange_message,

                // If `StreamParser` returns an Err pass it downstream
                Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),

                // If `StreamParser` returns None it's a safe-to-skip message
                None => continue,
            };

            if self.backtest_mode == BacktestMode::ToFile {
                match raw_msg {
                    Some(text) => {
                        let rec_time = chrono::Utc::now();
                        // let minute = (rec_time.minute() as f32 / 5.0).floor() as i32 * 5;
                        // TODO how to get ex/pair/id?
                        let formatted = rec_time.format("%Y_%m_%d_%H").to_string();
                        // + minute.to_string().as_str();
                        let file_name = format!("data/binance_l2_{}.dat", formatted);
                        flat_files::append_to_file(&text, &file_name).unwrap();
                    }
                    _ => {}
                };
            }

            // Transform `ExchangeMessage` into `Transformer::OutputIter`
            // ie/ IntoIterator<Item = Result<Output, SocketError>>
            self.transformer
                .transform(exchange_message)
                .into_iter()
                .for_each(
                    |output_result: Result<StreamTransformer::Output, StreamTransformer::Error>| {
                        self.buffer.push_back(output_result)
                    },
                );
        }
    }
}

impl<Protocol, InnerStream, StreamTransformer>
    ExchangeStream<Protocol, InnerStream, StreamTransformer>
where
    Protocol: StreamParser,
    InnerStream: Stream,
    StreamTransformer: Transformer,
{
    pub fn new(
        stream: InnerStream,
        transformer: StreamTransformer,
        backtest_mode: BacktestMode,
    ) -> Self {
        Self {
            stream,
            transformer,
            buffer: VecDeque::with_capacity(6),
            protocol_marker: PhantomData::default(),
            backtest_mode,
        }
    }
}

/// Initialise a `Subscriber` for `Tracing` Json logs and install it as the global default.
pub fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the `RUST_LOG` environment variable
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}
