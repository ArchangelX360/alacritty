use std::ffi::OsStr;
use std::io::{self, Result};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};

use crate::event::{OnResize, WindowSize};
use crate::tty::windows::child::ChildExitWatcher;
use crate::tty::{ChildEvent, EventedPty, EventedReadWrite, ExitStatus, Options, Shell};

mod blocking;
mod child;
mod conpty;

use blocking::{UnblockedReader, UnblockedWriter};
use conpty::Conpty as Backend;
use miow::pipe::{AnonRead, AnonWrite};
use polling::{Event, Poller};

pub const PTY_CHILD_EVENT_TOKEN: usize = 1;
pub const PTY_READ_WRITE_TOKEN: usize = 2;

type ReadPipe = UnblockedReader<AnonRead>;
type WritePipe = UnblockedWriter<AnonWrite>;

pub struct Pty {
    // XXX: Backend is required to be the first field, to ensure correct drop order. Dropping
    // `conout` before `backend` will cause a deadlock (with Conpty).
    backend: Backend,
    conout: ReadPipe,
    conin: WritePipe,
    child_watcher: ChildExitWatcher,
    child_process_id: u32,
    on_exit: Arc<Mutex<Option<Box<dyn FnOnce(ExitStatus) + Send>>>>,
}

pub fn new(
    config: &Options,
    window_size: WindowSize,
    _window_id: u64,
    on_exit: impl 'static + FnOnce(ExitStatus) + Send,
) -> Result<Pty> {
    conpty::new(config, window_size, on_exit)
}

impl Pty {
    fn new(
        backend: impl Into<Backend>,
        conout: impl Into<ReadPipe>,
        conin: impl Into<WritePipe>,
        child_watcher: ChildExitWatcher,
        child_process_id: u32,
        on_exit: Arc<Mutex<Option<Box<dyn FnOnce(ExitStatus) + Send>>>>,
    ) -> Self {
        Self {
            backend: backend.into(),
            conout: conout.into(),
            conin: conin.into(),
            child_watcher,
            child_process_id,
            on_exit,
        }
    }

    pub fn child_process_id(&self) -> u32 {
        self.child_process_id
    }

    pub fn child_watcher(&self) -> &ChildExitWatcher {
        &self.child_watcher
    }
}

fn with_key(mut event: Event, key: usize) -> Event {
    event.key = key;
    event
}

impl EventedReadWrite for Pty {
    type Reader = ReadPipe;
    type Writer = WritePipe;

    #[inline]
    unsafe fn register(
        &mut self,
        poll: &Arc<Poller>,
        interest: polling::Event,
        poll_opts: polling::PollMode,
    ) -> io::Result<()> {
        self.conin.register(poll, with_key(interest, PTY_READ_WRITE_TOKEN), poll_opts);
        self.conout.register(poll, with_key(interest, PTY_READ_WRITE_TOKEN), poll_opts);
        self.child_watcher.register(poll, with_key(interest, PTY_CHILD_EVENT_TOKEN));

        Ok(())
    }

    #[inline]
    fn reregister(
        &mut self,
        poll: &Arc<Poller>,
        interest: polling::Event,
        poll_opts: polling::PollMode,
    ) -> io::Result<()> {
        self.conin.register(poll, with_key(interest, PTY_READ_WRITE_TOKEN), poll_opts);
        self.conout.register(poll, with_key(interest, PTY_READ_WRITE_TOKEN), poll_opts);
        self.child_watcher.register(poll, with_key(interest, PTY_CHILD_EVENT_TOKEN));

        Ok(())
    }

    #[inline]
    fn deregister(&mut self, _poll: &Arc<Poller>) -> io::Result<()> {
        self.conin.deregister();
        self.conout.deregister();
        self.child_watcher.deregister();

        Ok(())
    }

    #[inline]
    fn reader(&mut self) -> &mut Self::Reader {
        &mut self.conout
    }

    #[inline]
    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.conin
    }
}

impl EventedPty for Pty {
    fn next_child_event(&mut self) -> Option<ChildEvent> {
        match self.child_watcher.event_rx().try_recv() {
            Ok(ev) => Some(ev),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                if let Some(on_exit) = self.on_exit.lock().unwrap().take() {
                    on_exit(ExitStatus::Other)
                }
                Some(ChildEvent::Exited(None))
            },
        }
    }
}

impl OnResize for Pty {
    fn on_resize(&mut self, window_size: WindowSize) {
        self.backend.on_resize(window_size)
    }
}

fn cmdline(config: &Options) -> String {
    let default_shell = Shell::new("powershell".to_owned(), Vec::new());
    let shell = config.shell.as_ref().unwrap_or(&default_shell);

    encode_command_line(shell.program.as_str(), shell.args.as_slice())
}

/// Converts the string slice into a Windows-standard representation for "W"-
/// suffixed function variants, which accept UTF-16 encoded string values.
pub fn win32_string<S: AsRef<OsStr> + ?Sized>(value: &S) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

pub fn encode_command_line(program: &str, arguments: &[String]) -> String {
    // Always quote the program name.
    let commandline = format!("\"{}\"", program);

    if arguments.is_empty() {
        commandline
    } else {
        format!(
            "{} {}",
            commandline,
            arguments.iter().flat_map(quote_arg).collect::<Vec<_>>().join(" ")
        )
    }
}

/// Quotes a command line argument.
///
/// See: https://docs.microsoft.com/en-gb/archive/blogs/twistylittlepassagesallalike/everyone-quotes-command-line-arguments-the-wrong-way
pub fn quote_arg(arg: impl AsRef<str>) -> Option<String> {
    let arg = arg.as_ref();
    if arg.is_empty() {
        return None;
    }

    let need_quotation = &[' ', '\t', '\n', '"'];
    if !arg.chars().any(|ch| need_quotation.contains(&ch)) {
        return Some(arg.to_owned());
    }

    let mut buf = String::new();
    buf.push_str("\"");
    let mut it = arg.chars().peekable();
    loop {
        let mut num_backslashes = 0;
        while let Some(ch) = it.peek() {
            if *ch == '\\' {
                it.next();
                num_backslashes += 1;
            } else {
                break;
            }
        }

        match it.next() {
            None => {
                while num_backslashes > 0 {
                    buf.push_str(r"\\");
                    num_backslashes -= 1;
                }
                break;
            },
            Some(c) => {
                if c == '"' {
                    while num_backslashes > 0 {
                        buf.push_str(r"\\");
                        num_backslashes -= 1;
                    }
                    buf.push_str(r"\");
                    buf.push_str("\"");
                } else {
                    while num_backslashes > 0 {
                        buf.push_str(r"\");
                        num_backslashes -= 1;
                    }
                    buf.push(c);
                }
            },
        }
    }

    buf.push('"');

    Some(buf)
}
