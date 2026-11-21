"""Hold the per-user backend lock while the local runtime runs."""
import errno
import fcntl
import os
from pathlib import Path
import socket

data = Path(os.environ["CHORD_DATA_DIR"])
data.mkdir(mode=0o700, parents=True, exist_ok=True)
with (data / "runtime.lock").open("a") as lock:
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        raise SystemExit("Chord's local backend is running already.")
    for port in (4789, 4808, 4811, 4812):
        with socket.socket() as listener:
            try:
                listener.bind(("127.0.0.1", port))
            except OSError as error:
                if error.errno == errno.EADDRINUSE:
                    raise SystemExit(f"Chord needs local port {port}, which is already in use.")
                raise
    os.set_inheritable(lock.fileno(), True)
    os.execvp("node", ["node", "scripts/runtime.mjs"])
