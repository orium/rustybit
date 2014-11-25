pub struct DuplexChannel<S,R>
{
    pub sender   : SyncSender<S>,
    pub receiver : Receiver<R>
}

pub fn sync_duplex_channel<T : Send,U : Send>(buff_cap : uint)
           -> (DuplexChannel<T,U>, DuplexChannel<U,T>)
{
    let (send_a, recv_b) = sync_channel(buff_cap);
    let (send_b, recv_a) = sync_channel(buff_cap);
    let dup_a : DuplexChannel<T,U> = DuplexChannel { sender:   send_a,
                                                     receiver: recv_a};
    let dup_b : DuplexChannel<U,T> = DuplexChannel { sender:   send_b,
                                                     receiver: recv_b};

    (dup_a, dup_b)
}
