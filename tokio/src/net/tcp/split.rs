//! `TcpStream` split support.
//!
//! A `TcpStream` can be split into a `ReadHalf` and a
//! `WriteHalf` with the `TcpStream::split` method. `ReadHalf`
//! implements `AsyncRead` while `WriteHalf` implements `AsyncWrite`.
//!
//! Compared to the generic split of `AsyncRead + AsyncWrite`, this specialized
//! split has no associated overhead and enforces all invariants at the type
//! level.

use crate::io::{AsyncRead, AsyncWrite};
use crate::net::TcpStream;

use bytes::{Buf, BufMut};
use std::io;
use std::net::Shutdown;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Read half of a `TcpStream`.
#[derive(Debug)]
pub struct ReadHalf<'a>(&'a TcpStream);

/// Write half of a `TcpStream`.
///
/// Note that in the `AsyncWrite` implemenation of `TcpStreamWriteHalf`,
/// `poll_shutdown` actually shuts down the TCP stream in the write direction.
#[derive(Debug)]
pub struct WriteHalf<'a>(&'a TcpStream);

pub(crate) fn split(stream: &mut TcpStream) -> (ReadHalf<'_>, WriteHalf<'_>) {
    (ReadHalf(&*stream), WriteHalf(&*stream))
}

impl AsyncRead for ReadHalf<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut dyn BufMut,
    ) -> Poll<io::Result<usize>> {
        self.0.poll_read_priv(cx, buf)
    }
}

impl AsyncWrite for WriteHalf<'_> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut dyn Buf,
    ) -> Poll<io::Result<usize>> {
        self.0.poll_write_priv(cx, buf)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // tcp flush is a no-op
        Poll::Ready(Ok(()))
    }

    // `poll_shutdown` on a write half shutdowns the stream in the "write" direction.
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.shutdown(Shutdown::Write).into()
    }
}

impl AsRef<TcpStream> for ReadHalf<'_> {
    fn as_ref(&self) -> &TcpStream {
        self.0
    }
}

impl AsRef<TcpStream> for WriteHalf<'_> {
    fn as_ref(&self) -> &TcpStream {
        self.0
    }
}
