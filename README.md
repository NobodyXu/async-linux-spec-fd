# async-linux-spec-fd

asynchronous linux specific fd in rust.

Supports:
 - `PidFd` for async and efficient method of reaping children process and race-free
   signal sending.
 - `SignalFd` for async way of accepting signals.
