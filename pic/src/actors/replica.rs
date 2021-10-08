use crate::actors::child::signals::outbound::ProcessRestarted;
use crate::actors::child::signals::ProcessRestartSubscribe;
use crate::actors::child::{ChildProcessActor, ChildProcessActorConfig};
use crate::actors::shutdown_controller::ShutdownController;
use actix::{Actor, Addr, ArbiterHandle, AsyncContext, Context, Handler, Running};
use std::process::{Command, Stdio};

pub struct ReplicaActorConfig {
    pub shutdown_controller: Option<Addr<ShutdownController>>,
}

pub struct ReplicaActor {
    child_addr: Addr<ChildProcessActor>,
}

impl ReplicaActor {
    pub fn new(config: ReplicaActorConfig) -> Self {
        let mut cmd = Command::new("echo");
        cmd.arg("Hello World");
        cmd.stdout(Stdio::inherit());

        let child_addr = ChildProcessActor::new(ChildProcessActorConfig {
            name: "replica".to_string(),
            command: cmd,
            shutdown_controller: config.shutdown_controller,
        })
        .start();

        Self { child_addr }
    }
}

impl Actor for ReplicaActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.child_addr.do_send(ProcessRestartSubscribe(
            ctx.address().recipient::<ProcessRestarted>(),
        ));
    }
}

impl Handler<ProcessRestarted> for ReplicaActor {
    type Result = ();

    fn handle(&mut self, msg: ProcessRestarted, ctx: &mut Self::Context) -> Self::Result {
        log::trace!("Process restarted!!!");
    }
}
