use crate::actors::shutdown::{wait_for_child_or_receiver, ChildOrReceiver};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Recipient,
    ResponseActFuture, Running, WrapFuture,
};
use anyhow::{Context as AnyhowContext, Result};
use crossbeam::channel::{unbounded, Receiver, Sender};
use garcon::{Delay, Waiter};
use std::process::Command;
use std::thread::JoinHandle;
use std::time::Duration;

pub mod signals {
    use actix::prelude::*;

    pub mod outbound {
        use super::*;

        #[derive(Message)]
        #[rtype(result = "()")]
        pub struct ProcessRestarted {}
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct ProcessRestartSubscribe(pub Recipient<outbound::ProcessRestarted>);

    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct TriggerProcessRestarted {}
}

pub struct ChildProcessActorConfig {
    /// Name for this child process actor, used for logging.
    pub name: String,
    /// The command that should be executed by this runner.
    pub command: Command,
    /// The shutdown controller that we must use.
    pub shutdown_controller: Option<Addr<ShutdownController>>,
}

/// An actix actor that can be used to spawn a [Command] in a different thread keep it running
/// in a loop and send signals when the process is restarted. It also handles graceful exits
/// for the command.
pub struct ChildProcessActor {
    /// Name used for logging.
    name: String,
    /// The shutdown controller that we must use.
    shutdown_controller: Option<Addr<ShutdownController>>,
    /// The command that should be executed by this runner.
    command: Option<Command>,
    /// A sender that sends is used to send the termination signal to the runner thread.
    terminate_sender: Option<Sender<()>>,
    /// The handle for the runner thread.
    thread_handle: Option<JoinHandle<()>>,
    /// List of subscribers that are interested in receiving the ProcessRestarted signal.
    subscribers: Vec<Recipient<signals::outbound::ProcessRestarted>>,
}

impl ChildProcessActor {
    pub fn new(config: ChildProcessActorConfig) -> Self {
        Self {
            name: config.name,
            shutdown_controller: config.shutdown_controller,
            command: Some(config.command),
            terminate_sender: None,
            thread_handle: None,
            subscribers: Vec::new(),
        }
    }

    pub fn run_command(&mut self, addr: Addr<Self>) -> Result<()> {
        let command = self
            .command
            .take()
            .context("Child process actor already started.")?;

        let (sender, kill_receiver) = unbounded();

        let handle = start_runner_thread(addr, command, self.name.clone(), kill_receiver)?;

        self.terminate_sender = Some(sender);
        self.thread_handle = Some(handle);

        Ok(())
    }
}

impl Actor for ChildProcessActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.run_command(ctx.address())
            .expect("Could not start the child process.");

        if let Some(shutdown_controller) = &self.shutdown_controller {
            shutdown_controller.do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        log::info!("Stopping child process {}", self.name);

        if let Some(sender) = self.terminate_sender.take() {
            let _ = sender.send(());
        }

        if let Some(join) = self.thread_handle.take() {
            let _ = join.join();
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

impl Handler<signals::ProcessRestartSubscribe> for ChildProcessActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: signals::ProcessRestartSubscribe,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.subscribers.push(msg.0)
    }
}

impl Handler<signals::TriggerProcessRestarted> for ChildProcessActor {
    type Result = ();

    fn handle(
        &mut self,
        _msg: signals::TriggerProcessRestarted,
        _: &mut Self::Context,
    ) -> Self::Result {
        for sub in &self.subscribers {
            let _ = sub.send(signals::outbound::ProcessRestarted {});
        }
    }
}

/// Start the thread that executes the given command, and sends R
fn start_runner_thread(
    addr: Addr<ChildProcessActor>,
    mut command: Command,
    name: String,
    kill_receiver: Receiver<()>,
) -> Result<JoinHandle<()>> {
    let thread_name = format!("child-process:{}", name);

    let thread_handler = move || {
        // Create a waiter to delay between executions of the command in the loop.
        let mut waiter = Delay::builder()
            .throttle(Duration::from_millis(1000))
            .exponential_backoff(Duration::from_secs(1), 1.2)
            .build();
        waiter.start();

        let mut done = false;
        while !done {
            let last_start = std::time::Instant::now();
            log::info!("Starting the process for '{}'", name);
            let mut child = command
                .spawn()
                .expect(&format!("Could not start the process for '{}'.", name));

            addr.do_send(signals::TriggerProcessRestarted {});

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
    };

    std::thread::Builder::new()
        .name(thread_name)
        .spawn(thread_handler)
        .map_err(|e| e.into())
}
