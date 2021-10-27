use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread::JoinHandle;
use std::time::Duration;

use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, ResponseActFuture,
    Running, WrapFuture,
};
use anyhow::{Context as AnyhowContext, Result};
use crossbeam::channel::{unbounded, Receiver, Sender};
use garcon::{Delay, Waiter};

use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::{ShutdownSubscribe, ShutdownTrigger};
use crate::actors::shutdown_controller::ShutdownController;

/// The callback which gets executed after each process restart.
pub type Callback = Box<dyn Fn(&Receiver<()>) + Send>;

pub struct ChildProcessActorConfig {
    /// Name for this child process actor, used for logging.
    pub name: String,
    /// The command that should be executed by this runner.
    pub command: Command,
    /// The shutdown controller that we must use.
    pub shutdown_controller: Option<Addr<ShutdownController>>,
    /// The callback that gets called after each execution.
    pub callback: Option<Callback>,
    /// The file to write the PID to. If this file already exists on the system, and a shutdown
    /// controller is provided, we send the shutdown signal.
    pub pid_file: Option<PathBuf>,
}

/// An actix actor that can be used to spawn a [Command] in a different thread keep it running
/// in a loop and send signals when the process is restarted. It also handles graceful exits
/// for the command.
pub struct ChildProcessActor {
    /// Name used for logging.
    name: String,
    /// The shutdown controller that we must use.
    shutdown_controller: Option<Addr<ShutdownController>>,
    /// The file to write the PID to.
    pid_file: Option<PathBuf>,
    /// The command that should be executed by this runner.
    command: Option<Command>,
    /// A sender that sends is used to send the termination signal to the runner thread.
    terminate_sender: Option<Sender<()>>,
    /// The handle for the runner thread.
    thread_handle: Option<JoinHandle<()>>,
    /// The callback to be called after each execution.
    callback: Option<Callback>,
}

impl ChildProcessActor {
    /// Create a new [`ChildProcessActor`] using the provided configurations.
    pub fn new(config: ChildProcessActorConfig) -> Self {
        Self {
            name: config.name,
            shutdown_controller: config.shutdown_controller,
            pid_file: config.pid_file,
            command: Some(config.command),
            terminate_sender: None,
            thread_handle: None,
            callback: config.callback,
        }
    }

    /// Start the runner thread.
    fn run_command(&mut self) -> Result<()> {
        let command = self
            .command
            .take()
            .context("Child process actor already started.")?;

        let callback = self.callback.take();
        let name = self.name.clone();
        let pid_file = self.pid_file.clone();
        let shutdown_controller = self.shutdown_controller.clone();

        let (sender, kill_receiver) = unbounded();

        let handle = start_runner_thread(
            command,
            name,
            pid_file,
            shutdown_controller,
            kill_receiver,
            callback,
        )?;

        self.terminate_sender = Some(sender);
        self.thread_handle = Some(handle);

        Ok(())
    }
}

impl Actor for ChildProcessActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.run_command()
            .expect("Could not start the child process.");

        if let Some(shutdown_controller) = &self.shutdown_controller {
            shutdown_controller.do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        log::info!("Stopping child process {}", self.name);

        if let Some(sender) = self.terminate_sender.take() {
            let _ = sender.send(());
            let _ = sender.send(());
        }

        if let Some(join) = self.thread_handle.take() {
            let _ = join.join();

            if let Some(path) = &self.pid_file {
                if fs::remove_file(path).is_ok() {
                    log::trace!(
                        "Removed the pid file for process '{}' after probable panic.",
                        self.name
                    )
                }
            }
        }

        Running::Stop
    }
}

impl Handler<Shutdown> for ChildProcessActor {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, _msg: Shutdown, _ctx: &mut Self::Context) -> Self::Result {
        // This is just the example for ResponseActFuture but stopping the context
        Box::pin(
            async {}
                .into_actor(self) // converts future to ActorFuture
                .map(|_, _act, ctx| {
                    ctx.stop();
                    Ok(())
                }),
        )
    }
}

/// Start the thread that executes the given command, and sends R
fn start_runner_thread(
    mut command: Command,
    name: String,
    pid_file: Option<PathBuf>,
    shutdown_controller: Option<Addr<ShutdownController>>,
    kill_receiver: Receiver<()>,
    callback: Option<Callback>,
) -> Result<JoinHandle<()>> {
    let thread_name = format!("child-process:{}", name);

    let thread_handler = move || {
        // Create a waiter to delay between executions of the command in the loop.
        let mut waiter = Delay::builder()
            .throttle(Duration::from_millis(1000))
            .exponential_backoff(Duration::from_secs(1), 1.2)
            .build();
        waiter.start();

        if let Some(path) = &pid_file {
            if path.is_file() {
                log::error!(
                    "Cannot start the '{}' process since the lock file already exists.",
                    name
                );

                if let Some(controller) = shutdown_controller {
                    log::trace!("Sending the shutdown signal due the error.");
                    controller.do_send(ShutdownTrigger());
                }

                return;
            }

            fs::write(path, "")
                .unwrap_or_else(|_| panic!("Could not obtain the lock for process '{}'.", name));
        }

        let mut done = false;
        while !done {
            let last_start = std::time::Instant::now();
            log::info!("Starting the process for '{}'", name);

            let mut child = command
                .spawn()
                .unwrap_or_else(|_| panic!("Could not start the process for '{}'.", name));

            if let Some(path) = &pid_file {
                fs::write(path, format!("{}", child.id())).unwrap_or_else(|_| {
                    panic!("Could not write the PID lock for process '{}'.", name)
                });
            }

            if let Some(cb) = &callback {
                cb(&kill_receiver);
            }

            // This waits for the child to stop, or the receiver to receive a message.
            // We don't restart the replica if done = true.
            match wait_for_child_or_receiver(&mut child, &kill_receiver) {
                ChildOrReceiver::Receiver => {
                    log::trace!("Got signal to stop. Killing process '{}'...", name);
                    let _ = child.kill();
                    let _ = child.wait();
                    done = true;
                }
                ChildOrReceiver::Child => {
                    log::trace!("Child process '{}' failed.", name);
                    // Reset waiter if last start was over 2 seconds ago, and do not wait.
                    if std::time::Instant::now().duration_since(last_start)
                        >= Duration::from_secs(2)
                    {
                        log::info!("Last run seemed to have been healthy, not waiting...");
                        waiter.start();
                    } else {
                        // Wait before we start it again.
                        let _ = waiter.wait();
                    }
                }
            }
        }

        if let Some(path) = &pid_file {
            fs::remove_file(path)
                .unwrap_or_else(|_| panic!("Cannot remove the PID lock for process '{}'", name));

            log::trace!("Removed the PID file for process '{}'.", name)
        }
    };

    std::thread::Builder::new()
        .name(thread_name)
        .spawn(thread_handler)
        .map_err(|e| e.into())
}
