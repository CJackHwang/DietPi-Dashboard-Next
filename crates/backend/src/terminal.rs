use anyhow::{Context, Result};
use log::error;
use proto::backend::{ActionBackendMessage, BackendMessage};
use proto::frontend::ActionFrontendMessage;
use pty_process::{Command, Pts, Pty, Size};
use tokio::process::Child;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};

#[cfg(target_os = "linux")]
fn spawn_agetty() -> Result<(Pty, Pts, Child)> {
    let (pty, pts) = pty_process::open().context("failed to open pty")?;
    pty.resize(Size::new(24, 80))?;
    let mut cmd = Command::new("agetty").args(["-8", "-L", "-c", "-", "xterm-256color"]);
    let child = cmd.spawn_borrowed(&pts).context("failed to spawn agetty")?;
    Ok((pty, pts, child))
}

// This is purely so that I can build this on a Mac to develop
#[cfg(not(target_os = "linux"))]
fn spawn_agetty() -> Result<(Pty, Pts, Child)> {
    anyhow::bail!("terminal only works on Linux targets");
}

pub struct Terminal {
    socket_tx: mpsc::UnboundedSender<BackendMessage>,
    rx: mpsc::UnboundedReceiver<ActionFrontendMessage>,
    pty: Pty,
    pts: Pts,
    child: Child,
}

impl Terminal {
    pub fn new(
        socket_tx: mpsc::UnboundedSender<BackendMessage>,
        rx: mpsc::UnboundedReceiver<ActionFrontendMessage>,
    ) -> Result<Self> {
        let (pty, pts, child) = spawn_agetty()?;

        Ok(Self {
            socket_tx,
            rx,
            pty,
            pts,
            child,
        })
    }

    pub async fn run(mut self) {
        let mut buf = [0; 4096];

        loop {
            loop {
                tokio::select! {
                    data = self.rx.recv() => {
                        let Some(data) = data else {
                            return;
                        };

                        match data {
                            ActionFrontendMessage::Terminal(data) => {
                                if self.pty.write_all(&data).await.is_err() {
                                    break;
                                }
                            }
                            ActionFrontendMessage::ResizeTerminal(size) => {
                                let size = Size::new(size.rows, size.cols);
                                if self.pty.resize(size).is_err() {
                                    break;
                                }
                            }
                            _ => unreachable!()
                        };
                    }
                    n = self.pty.read(&mut buf) => {
                        let Ok(n) = n else {
                            break;
                        };

                        if n == 0 {
                            break;
                        }

                        let msg = ActionBackendMessage::Terminal(buf[..n].to_vec());
                        let msg = BackendMessage::Action(msg);

                        let _ = self.socket_tx.send(msg);
                    }
                    _ = self.child.wait() => {
                        break;
                    }
                }
            }

            drop(self.pty);
            drop(self.pts);

            // Send escape sequence to clear terminal
            let _ = self
                .socket_tx
                .send(BackendMessage::Action(ActionBackendMessage::Terminal(
                    b"\x1Bc".to_vec(),
                )));

            match spawn_agetty() {
                Ok((pty, pts, child)) => {
                    self.pty = pty;
                    self.pts = pts;
                    self.child = child
                }
                Err(err) => {
                    error!("{err:#}");
                    break;
                }
            }
        }
    }
}
