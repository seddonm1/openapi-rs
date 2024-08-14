#![forbid(unsafe_code)]
#![warn(
    clippy::await_holding_lock,
    clippy::cargo_common_metadata,
    clippy::dbg_macro,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::inefficient_to_string,
    clippy::mem_forget,
    clippy::mutex_integer,
    clippy::needless_continue,
    clippy::todo,
    clippy::unimplemented,
    clippy::wildcard_imports,
    missing_debug_implementations
)]

use crossbeam_channel::Sender;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use rusqlite::OpenFlags;
use rusqlite_migration::Migrations;
use std::path::Path;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{
    fmt::{self, Debug, Display},
    thread,
};
use tokio::sync::oneshot;

static MESSAGE_BOUND: usize = 100;
static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::from_directory(&MIGRATIONS_DIR).unwrap();
}

const BUG_TEXT: &str = "bug in tokio-rusqlite, please report";

// Helper function to return a comma-separated sequence of `?`.
// - `repeat_vars(0) => panic!(...)`
// - `repeat_vars(1) => "?"`
// - `repeat_vars(2) => "?,?"`
// - `repeat_vars(3) => "?,?,?"`
// - ...
pub fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}

#[derive(Debug)]
/// Represents the errors specific for this library.
#[non_exhaustive]
#[allow(dead_code)]
pub(crate) enum Error {
    /// The connection to the SQLite has been closed and cannot be queried any more.
    ConnectionClosed,

    /// A `Rusqlite` error occured.
    Rusqlite(rusqlite::Error),

    /// An application-specific error occured.
    Other(anyhow::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConnectionClosed => write!(f, "ConnectionClosed"),
            Error::Rusqlite(e) => write!(f, "Rusqlite(\"{e}\")"),
            Error::Other(ref e) => write!(f, "Other(\"{e}\")"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ConnectionClosed => None,
            Error::Rusqlite(e) => Some(e),
            Error::Other(ref e) => Some(&**e),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Error::Rusqlite(value)
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Error::Other(value)
    }
}

impl From<rusqlite_migration::Error> for Error {
    fn from(value: rusqlite_migration::Error) -> Self {
        Error::Other(value.into())
    }
}

/// The result returned on method calls in this crate.
pub(crate) type Result<T> = std::result::Result<T, Error>;

type CallFn = Box<dyn FnOnce(&mut rusqlite::Connection) + Send + 'static>;

#[allow(dead_code)]
enum Message {
    Execute(CallFn),
    Close,
}

/// A handle to call functions in background thread.
#[derive(Clone)]
pub struct Database {
    writer_sender: Sender<Message>,
    reader_sender: Sender<Message>,
    writer_handle: Arc<JoinHandle<()>>,
    reader_handles: Arc<Vec<JoinHandle<()>>>,
}

impl Database {
    /// Open a new connection to a SQLite database.
    ///
    /// `Connection::open(path)` is equivalent to
    /// `Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE |
    /// OpenFlags::SQLITE_OPEN_CREATE)`.
    ///
    /// # Failure
    ///
    /// Will return `Err` if `path` cannot be converted to a C-compatible
    /// string or if the underlying SQLite open call fails.
    pub(crate) async fn open<P: AsRef<Path>>(path: P, readers: usize) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let path_clone = path.clone();

        start(
            move || {
                let mut writer: rusqlite::Connection = rusqlite::Connection::open(path)?;
                writer.set_prepared_statement_cache_capacity(1024);
                writer.execute_batch(
                    "
                    PRAGMA journal_mode=WAL;
                    PRAGMA synchronous=NORMAL;
                    PRAGMA temp_store=MEMORY;
                    PRAGMA cache_size=-67108864;
                    PRAGMA foreign_keys=true;
                    PRAGMA busy_timeout=5000;
                    ",
                )?;

                MIGRATIONS
                    .to_latest(&mut writer)
                    .map_err(|err| rusqlite::Error::UserFunctionError(Box::new(err)))?;

                Ok(writer)
            },
            Arc::new(move || {
                let reader = rusqlite::Connection::open_with_flags(
                    path_clone.clone(),
                    OpenFlags::SQLITE_OPEN_READ_ONLY,
                )?;
                reader.set_prepared_statement_cache_capacity(1024);

                Ok(reader)
            }),
            readers,
        )
        .await
        .map_err(Error::Rusqlite)
    }

    /// Open a new connection to an in-memory SQLite database.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the underlying SQLite open call fails.
    #[allow(dead_code)]
    pub(crate) async fn open_in_memory(readers: usize) -> Result<Self> {
        let name = format!("file:{}?mode=memory&cache=shared", uuid::Uuid::new_v4());
        Self::open(name, readers).await
    }

    /// Call a function in background thread and get the result
    /// asynchronously.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the database connection has been closed.
    #[allow(dead_code)]
    pub(crate) async fn write<F, R>(&self, function: F) -> Result<R>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<R> + 'static + Send,
        R: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel::<Result<R>>();

        self.writer_sender
            .send(Message::Execute(Box::new(move |conn| {
                let value = function(conn);
                let _ = sender.send(value);
            })))
            .map_err(|_| Error::ConnectionClosed)?;

        receiver.await.map_err(|_| Error::ConnectionClosed)?
    }

    /// Call a function in background thread and get the result
    /// asynchronously.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the database connection has been closed.
    #[allow(dead_code)]
    pub(crate) async fn read<F, R>(&self, function: F) -> Result<R>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<R> + 'static + Send,
        R: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel::<Result<R>>();

        self.reader_sender
            .send(Message::Execute(Box::new(move |conn| {
                let value = function(conn);
                let _ = sender.send(value);
            })))
            .map_err(|_| Error::ConnectionClosed)?;

        receiver.await.map_err(|_| Error::ConnectionClosed)?
    }

    /// Close the database connection.
    ///
    /// This is functionally equivalent to the `Drop` implementation for
    /// `Connection`. It consumes the `Connection`, but on error returns it
    /// to the caller for retry purposes.
    ///
    /// If successful, any following `close` operations performed
    /// on `Connection` copies will succeed immediately.
    ///
    /// On the other hand, any calls to [`Connection::call`] will return a [`Error::ConnectionClosed`],
    /// and any calls to [`Connection::call_unwrap`] will cause a `panic`.
    ///
    /// # Failure
    ///
    /// Will return `Err` if the underlying SQLite close call fails.
    #[allow(dead_code)]
    pub(crate) async fn close(self) -> Result<()> {
        // close readers
        let reader_sender = self.reader_sender.clone();
        while self
            .reader_handles
            .iter()
            .any(|reader_handle| !reader_handle.is_finished())
        {
            let reader_sender = reader_sender.clone();
            reader_sender.send(Message::Close).ok();
        }

        // close writer
        while !self.writer_handle.is_finished() {
            self.writer_sender.send(Message::Close).ok();
        }

        Ok(())
    }
}

impl Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection").finish()
    }
}

async fn start<F, G>(
    open_writer: F,
    open_reader: Arc<G>,
    readers: usize,
) -> rusqlite::Result<Database>
where
    F: FnOnce() -> rusqlite::Result<rusqlite::Connection> + Send + 'static,
    G: Fn() -> rusqlite::Result<rusqlite::Connection> + Send + Sync + 'static,
{
    let (writer_sender, writer_receiver) = crossbeam_channel::bounded::<Message>(MESSAGE_BOUND);
    let (writer_result_sender, writer_result_receiver) = oneshot::channel();

    let writer_handle = thread::spawn(move || {
        let mut conn = match open_writer() {
            Ok(c) => c,
            Err(e) => {
                let _ = writer_result_sender.send(Err(e));
                return;
            }
        };

        if let Err(_e) = writer_result_sender.send(Ok(())) {
            return;
        }

        while let Ok(message) = writer_receiver.recv() {
            match message {
                Message::Execute(f) => f(&mut conn),
                Message::Close => {
                    let result = conn.close();

                    match result {
                        Ok(_) => {
                            break;
                        }
                        Err((_c, _e)) => {
                            break;
                        }
                    }
                }
            }
        }
    });
    writer_result_receiver.await.expect(BUG_TEXT)?;

    let (reader_sender, reader_receiver) = crossbeam_channel::bounded::<Message>(MESSAGE_BOUND);
    let mut reader_handles = Vec::with_capacity(readers);
    for _ in 0..readers {
        let (reader_result_sender, reader_result_receiver) = oneshot::channel();
        let reader_receiver = reader_receiver.clone();
        let open_reader = open_reader.clone();
        reader_handles.push(thread::spawn(move || {
            let mut conn = match open_reader() {
                Ok(c) => c,
                Err(e) => {
                    let _ = reader_result_sender.send(Err(e));
                    return;
                }
            };

            if let Err(_e) = reader_result_sender.send(Ok(())) {
                return;
            }

            while let Ok(message) = reader_receiver.recv() {
                match message {
                    Message::Execute(f) => f(&mut conn),
                    Message::Close => {
                        let result = conn.close();

                        match result {
                            Ok(_) => {
                                break;
                            }
                            Err((_c, _e)) => {
                                break;
                            }
                        }
                    }
                }
            }
        }));
        reader_result_receiver.await.expect(BUG_TEXT)?;
    }

    Ok(Database {
        writer_sender,
        reader_sender,
        writer_handle: Arc::new(writer_handle),
        reader_handles: Arc::new(reader_handles),
    })
}
