use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use crate::actors::shutdown_controller::signals::ShutdownSubscribe;
use crate::actors::shutdown_controller::ShutdownController;
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, ArbiterHandle, AsyncContext, Context, Handler,
    ResponseActFuture, Running, WrapFuture,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::thread::JoinHandle;

pub mod signals {
    use actix::prelude::*;

    /// A message sent to the Replica when the process is restarted. Since we're
    /// restarting inside our own actor, this message should not be exposed.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct ReplicaRestarted {
        pub port: u16,
    }
}

pub struct ReplicaActorConfig {
    pub shutdown_controller: Addr<ShutdownController>,
}

pub struct ReplicaActor {
    /// The configs used to run the replica.
    config: ReplicaActorConfig,
    /// The latest replica port.
    port: Option<u16>,
    /// Send a message to the thread to command it to stop the replica.
    stop_sender: Option<Sender<()>>,
    /// The thread handler for the replica thread.
    join_handler: Option<JoinHandle<()>>,
}

impl ReplicaActor {
    pub fn new(config: ReplicaActorConfig) -> Self {
        ReplicaActor {
            config,
            port: None,
            stop_sender: None,
            join_handler: None,
        }
    }

    fn start_replica(&mut self, addr: Addr<Self>) {
        log::trace!("Starting the replica actor.");
    }
}

impl Actor for ReplicaActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_replica(ctx.address());
        // .expect("Could not start the replica.")

        self.config
            .shutdown_controller
            .do_send(ShutdownSubscribe(ctx.address().recipient::<Shutdown>()));
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::info!("Stopping the replica.");

        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(join) = self.join_handler.take() {
            let _ = join.join();
        }

        log::info!("Stopped.");
        Running::Stop
    }
}

impl Handler<signals::ReplicaRestarted> for ReplicaActor {
    type Result = ();

    fn handle(&mut self, msg: signals::ReplicaRestarted, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

impl Handler<Shutdown> for ReplicaActor {
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
