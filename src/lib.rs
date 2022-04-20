#![no_std]

use embedded_nal::{nb, SocketAddr, TcpClientStack};

pub trait TcpClientStackPlus<TCS: TcpClientStack> {
    /// create a tuple referencing the TcpClientStack and a TcpSocket that has all the information necessary to read and write data.
    fn with_socket<'a>(&'a mut self, socket: &'a mut TCS::TcpSocket) -> StackAndSocket<'a, TCS>;
}

impl<TCS: TcpClientStack> TcpClientStackPlus<TCS> for TCS {
    /// create a tuple referencing the TcpClientStack and a TcpSocket that has all the information necessary to read and write data.
    fn with_socket<'a>(&'a mut self, socket: &'a mut TCS::TcpSocket) -> StackAndSocket<'a, TCS>
    where
        Self: Sized,
    {
        StackAndSocket::new(self, socket)
    }
}

/// This tuple combines a reference to the [TcpClientStack] and a [TcpSocket] into a single entity that knows enough to read or write to the socket.
/// It is intended to be an ephemeral construction, and was primarily motivated as an implementer of [ufmt::uWrite].
pub struct StackAndSocket<'a, TCS>
where
    TCS: TcpClientStack,
{
    /// the [TcpClientStack] needed to perform operations on the [self.socket]
    pub tcp_stack: &'a mut TCS,
    /// a [TcpSocket] usable with the [self.stack] to perform I/O.
    pub socket: &'a mut TCS::TcpSocket,
}

impl<'a, TCS> StackAndSocket<'a, TCS>
where
    TCS: TcpClientStack,
{
    /// create a new [StackAndSocket] from the stack reference and socket reference
    pub fn new(tcp_stack: &'a mut TCS, socket: &'a mut TCS::TcpSocket) -> Self {
        StackAndSocket { tcp_stack, socket }
    }

    /// Connect to the given remote host and port.
    ///
    /// Returns `Ok` if the connection was successful. Otherwise, if the connection could not be
    /// completed immediately, this function should return [`nb::Error::WouldBlock`].
    pub fn connect(&mut self, remote: SocketAddr) -> nb::Result<(), TCS::Error> {
        self.tcp_stack.connect(self.socket, remote)
    }

    /// Check if this socket is connected
    pub fn is_connected(&mut self) -> Result<bool, TCS::Error> {
        self.tcp_stack.is_connected(self.socket)
    }

    /// Receive data from the stream.
    ///
    /// Returns `Ok(n)`, which means `n` bytes of data have been received and
    /// they have been placed in `&buffer[0..n]`, or an error. If a packet has
    /// not been received when called, then [`nb::Error::WouldBlock`]
    /// should be returned.
    pub fn receive(&mut self, buffer: &mut [u8]) -> nb::Result<usize, TCS::Error> {
        self.tcp_stack.receive(self.socket, buffer)
    }

    /// Write to the stream.
    ///
    /// Returns the number of bytes written (which may be less than `buffer.len()`) or an error.
    pub fn send(&mut self, buffer: &[u8]) -> nb::Result<usize, TCS::Error> {
        self.tcp_stack.send(self.socket, buffer)
    }
}

impl<'a, TCS> ufmt::uWrite for StackAndSocket<'a, TCS>
where
    TCS: TcpClientStack,
{
    type Error = TCS::Error;
    fn write_str(&mut self, message: &str) -> Result<(), <Self as ufmt::uWrite>::Error> {
        let message = message.as_bytes();
        let mut cursor = 0;
        while cursor < message.len() {
            let n = nb::block!(
                self.tcp_stack.send(self.socket, &message[cursor..]) //.map_err(|e| nb::Error::Other(e))
            )?;
            cursor += n;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
