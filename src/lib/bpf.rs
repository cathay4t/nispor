// SPDX-License-Identifier: Apache-2.0

use netlink_sys::{AsyncSocket, Socket, TokioSocket};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct NisporBpfSocket(TokioSocket);

impl AsyncSocket for TokioSocket {
    fn socket_ref(&self) -> &Socket {
        self.0.socket_ref()
    }

    fn socket_mut(&mut self) -> &mut Socket {
        self.0.socket_mut()
    }

    fn new(protocol: isize) -> io::Result<Self> {
        let tk_sock = TokioSocket::new()?;
        Ok(Self(tk_sock))
    }

    fn poll_send(
        &mut self,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.poll_send(cx, buf)
    }

    fn poll_send_to(
        &mut self,
        cx: &mut Context<'_>,
        buf: &[u8],
        addr: &SocketAddr,
    ) -> Poll<io::Result<usize>> {
        self.0.poll_send_to(cx, buf, addr)
    }

    fn poll_recv<B>(
        &mut self,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<()>>
    where
        B: bytes::BufMut,
    {
        self.0.poll_recv(cx, buf)
    }

    fn poll_recv_from<B>(
        &mut self,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<SocketAddr>>
    where
        B: bytes::BufMut,
    {
        self.0.poll_recv_from(cx, buf)
    }

    fn poll_recv_from_full(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<(Vec<u8>, SocketAddr)>> {
        self.0.poll_recv_from_full(cx)
    }
}
