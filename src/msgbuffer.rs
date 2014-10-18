extern crate time;

use std::io::TcpStream;

use message::Message;
use message::MsgVersion;
use message::MsgVersionAck;
use message::MsgPing;
use message::MsgPong;
use message::MsgAddresses;
use message::MsgInv;
use message::MsgGetData;
use message::MsgReject;
use message::MsgTx;

use message::version::Version;
use message::versionack::VersionAck;
use message::ping::Ping;
use message::pong::Pong;
use message::addresses::Addresses;
use message::inv::Inv;
use message::getdata::GetData;
use message::reject::Reject;
use message::tx::Tx;

use message::header::Header;
use message::header::HEADER_SIZE;

use peer::PeerError;
use peer::ReadEOF;
use peer::ReadTimeout;
use peer::ReadIncomplete;
use peer::ReadIOError;
use peer::ReadMsgPayloadTooBig;
use peer::ReadMsgInvalidChecksum;
use peer::ReadMsgWrongNetwork;
use peer::ReadMsgUnknownCommand;

static PAYLOAD_MAX_SIZE : uint = 4*(1<<20); /* 4MB */

static TIMEOUT_READ_MS : uint = 500;

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

            *self.buf.get_mut(dst) = self.buf[src];
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
                    ::std::io::EndOfFile => return Err(ReadEOF),
                    ::std::io::TimedOut  => return Err(ReadTimeout),
                    _                    => return Err(ReadIOError)
                }
            }

            assert!(self.buf.len() <= size);

            /* We fail to read the entire thing */
            if self.buf.len() < size
            {
                return Err(ReadIncomplete);
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
            println!("message payload length too big");

            return Err(ReadMsgPayloadTooBig);
        }

        /* Read enoght to have the message payload */
        try!(self.read_ensure_size(HEADER_SIZE+header.get_payload_size(),socket));

        assert!(self.buf.len() == HEADER_SIZE+header.get_payload_size());

        /* We can now safely drop the header, since we have a complete message */
        self.drop(HEADER_SIZE);

        assert!(self.buf.len() == header.get_payload_size());

        if ::crypto::checksum(&self.buf) != header.get_checksum()
        {
            println!("invalid checksum");

            self.buf.clear();

            return Err(ReadMsgInvalidChecksum);
        }

        if header.get_network() != ::config::NETWORK
        {
            println!("wrong network");

            self.buf.clear();

            return Err(ReadMsgWrongNetwork);
        }

        println!(">>> {}  {} \tcommand: {:9}",
                 time::now().rfc822z(),
                 socket.peer_name().unwrap(),
                 header.get_command());

        msg = match header.get_command().as_slice()
        {
            "version" =>
            {
                let version : Version;

                version = Version::unserialize(&self.buf,
                                               header.get_payload_size());

                Ok(MsgVersion(version))
            },
            "verack" =>
            {
                let verack : VersionAck;

                verack = VersionAck::unserialize(&self.buf);

                Ok(MsgVersionAck(verack))
            },
            "ping" =>
            {
                let ping : Ping;

                ping = Ping::unserialize(&self.buf);

                Ok(MsgPing(ping))
            },
            "pong" =>
            {
                let pong : Pong;

                pong = Pong::unserialize(&self.buf);

                Ok(MsgPong(pong))
            },
            "addr" =>
            {
                let addr : Addresses;

                addr = Addresses::unserialize(&self.buf);

                Ok(MsgAddresses(addr))
            },
            "inv" =>
            {
                let inv : Inv;

                inv = Inv::unserialize(&self.buf);

                Ok(MsgInv(inv))
            },
            "getdata" =>
            {
                let getdata : GetData;

                getdata = GetData::unserialize(&self.buf);

                Ok(MsgGetData(getdata))
            },
            "reject" =>
            {
                let reject : Reject;

                reject = Reject::unserialize(&self.buf);

                Ok(MsgReject(reject))
            },
            "tx" =>
            {
                let tx : Tx;

                tx = Tx::unserialize(&self.buf);

                Ok(MsgTx(tx))
            },
            _ => Err(ReadMsgUnknownCommand)
        };

        self.buf.clear();

        msg
    }
}
