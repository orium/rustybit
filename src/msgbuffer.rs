use std::io::TcpStream;

use message::Message;

use message::version::Version;
use message::verack::VerAck;
use message::ping::Ping;
use message::pong::Pong;
use message::addr::Addr;
use message::inv::Inv;
use message::getdata::GetData;
use message::reject::Reject;
use message::tx::Tx;
use message::getaddr::GetAddr;

use message::header::Header;
use message::header::HEADER_SIZE;

use peer::PeerError;

const PAYLOAD_MAX_SIZE : uint = 4*(1<<20); /* 4MB */

const TIMEOUT_READ_MS : uint = 500;

pub struct MsgBuffer
{
    buf : Vec<u8>
}

impl MsgBuffer
{
    pub fn new() -> MsgBuffer
    {
        MsgBuffer
        {
            buf: Vec::with_capacity(PAYLOAD_MAX_SIZE+HEADER_SIZE)
        }
    }

    fn drop(&mut self, n : uint)
    {
        let len = self.buf.len();

        assert!(self.buf.len() >= n);

        for src in range(n,len)
        {
            let dst = src-n;

            self.buf[dst] = self.buf[src];
        }

        self.buf.truncate(len-n);
    }

    fn read_ensure_size(&mut self, size : uint, socket : &mut TcpStream)
                        -> Result<(),PeerError>
    {
        if self.buf.len() < size
        {
            let result;

            socket.set_read_timeout(Some(TIMEOUT_READ_MS as u64));
            result = socket.push(size-self.buf.len(),&mut self.buf);

            if result.is_err()
            {
                match result.err().unwrap().kind
                {
                    ::std::io::EndOfFile => return Err(PeerError::ReadEOF),
                    ::std::io::TimedOut  => return Err(PeerError::ReadTimeout),
                    _                    => return Err(PeerError::ReadIOError)
                }
            }

            assert!(self.buf.len() <= size);

            /* We fail to read the entire thing */
            if self.buf.len() < size
            {
                return Err(PeerError::ReadIncomplete);
            }

            assert!(self.buf.len() == size);
        }

        assert!(self.buf.len() >= size);

        Ok(())
    }

    pub fn read_message(&mut self, socket : &mut TcpStream)
                        -> Result<Message,PeerError>
    {
        let header : Header;
        let msg : Result<Message,PeerError>;

        /* We should never have to expand */
        assert!(self.buf.capacity() == PAYLOAD_MAX_SIZE+HEADER_SIZE);

        /* Read enoght to have a header */
        try!(self.read_ensure_size(HEADER_SIZE,socket));

        assert!(self.buf.len() >= HEADER_SIZE);

        header = Header::unserialize(&self.buf);

        if header.get_payload_size() > PAYLOAD_MAX_SIZE
        {
            return Err(PeerError::ReadMsgPayloadTooBig);
        }

        /* Read enoght to have the message payload */
        try!(self.read_ensure_size(HEADER_SIZE+header.get_payload_size(),socket));

        assert!(self.buf.len() == HEADER_SIZE+header.get_payload_size());

        /* We can now safely drop the header, since we have a complete message */
        self.drop(HEADER_SIZE);

        assert!(self.buf.len() == header.get_payload_size());

        if ::crypto::checksum(self.buf.as_slice()) != header.get_checksum()
        {
            return Err(PeerError::ReadMsgInvalidChecksum);
        }

        if header.get_network() != ::config::NETWORK
        {
            return Err(PeerError::ReadMsgWrongNetwork);
        }

        msg = match header.get_command().as_slice()
        {
            "version" =>
            {
                let version : Version;

                version = Version::unserialize(&self.buf,
                                               header.get_payload_size());

                Ok(Message::MsgVersion(version))
            },
            "verack" =>
            {
                let verack : VerAck;

                verack = VerAck::unserialize(&self.buf);

                Ok(Message::MsgVerAck(verack))
            },
            "ping" =>
            {
                let ping : Ping;

                ping = Ping::unserialize(&self.buf);

                Ok(Message::MsgPing(ping))
            },
            "pong" =>
            {
                let pong : Pong;

                pong = Pong::unserialize(&self.buf);

                Ok(Message::MsgPong(pong))
            },
            "addr" =>
            {
                let addr : Addr;

                addr = Addr::unserialize(&self.buf);

                Ok(Message::MsgAddr(addr))
            },
            "inv" =>
            {
                let inv : Inv;

                inv = Inv::unserialize(&self.buf);

                Ok(Message::MsgInv(inv))
            },
            "getdata" =>
            {
                let getdata : GetData;

                getdata = GetData::unserialize(&self.buf);

                Ok(Message::MsgGetData(getdata))
            },
            "reject" =>
            {
                let reject : Reject;

                reject = Reject::unserialize(&self.buf);

                Ok(Message::MsgReject(reject))
            },
            "tx" =>
            {
                let tx : Tx;

                tx = Tx::unserialize(&self.buf);

                Ok(Message::MsgTx(tx))
            },
            "getaddr" =>
            {
                let getaddr : GetAddr;

                getaddr = GetAddr::unserialize(&self.buf);

                Ok(Message::MsgGetAddr(getaddr))
            },
            _ => Err(PeerError::ReadMsgUnknownCommand)
        };

        self.buf.clear();

        msg
    }
}
