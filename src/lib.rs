use std::io::{self, Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

use async_trait::async_trait;
use blocking::Unblock;
use wait_timeout::ChildExt as _;

#[derive(Debug)]
pub struct SpawnAsyncOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_status: ExitStatus,
}

#[async_trait]
pub trait ChildExt {
    async fn spawn_async(
        &mut self,
        stdin: Option<Vec<u8>>,
        timeout: Option<Duration>,
        max_size: Option<usize>,
    ) -> io::Result<SpawnAsyncOutput>;
}

#[async_trait]
impl ChildExt for Command {
    async fn spawn_async(
        &mut self,
        stdin: Option<Vec<u8>>,
        timeout: Option<Duration>,
        max_size: Option<usize>,
    ) -> io::Result<SpawnAsyncOutput> {
        let command = self.stdout(Stdio::piped()).stderr(Stdio::piped());
        if let Some(_) = stdin {
            command.stdin(Stdio::piped());
        }

        let mut child = command.spawn()?;

        Unblock::new(())
            .with_mut(move |_| {
                if let Some(stdin_bytes) = stdin {
                    let stdin = match child.stdin.as_mut() {
                        Some(stdin) => stdin,
                        None => unreachable!("never"),
                    };
                    stdin.write_all(&stdin_bytes[..])?;
                }

                let exit_status =
                    match child.wait_timeout(timeout.unwrap_or(Duration::from_millis(1000)))? {
                        Some(exit_status) => exit_status,
                        None => {
                            // child hasn't exited yet
                            child.kill()?;
                            child.wait()?;

                            return Err(io::Error::new(io::ErrorKind::TimedOut, "run timeout"));
                        }
                    };

                let max_size = max_size.unwrap_or(2048);
                let mut buf = Vec::<u8>::with_capacity(max_size);
                buf.resize(max_size, 0);

                let stdout = child
                    .stdout
                    .as_mut()
                    .ok_or(io::Error::new(io::ErrorKind::Other, "never"))?;
                let n = stdout.read(&mut buf)?;
                let stdout_bytes = buf[..n].to_vec();

                let stderr = child
                    .stderr
                    .as_mut()
                    .ok_or(io::Error::new(io::ErrorKind::Other, "never"))?;
                let n = stderr.read(&mut buf)?;
                let stderr_bytes = buf[..n].to_vec();

                Ok(SpawnAsyncOutput {
                    stdout: stdout_bytes,
                    stderr: stderr_bytes,
                    exit_status,
                })
            })
            .await
    }
}
