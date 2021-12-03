use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Recipient};
use crossbeam::channel::Receiver;
use garcon::{Delay, Waiter};

use crate::actors::child_process::{ChildProcessActor, ChildProcessActorConfig};
use crate::actors::replica::signals::{PortChangeSubscribe, ProcessRestarted};
use crate::actors::shutdown_controller::ShutdownController;

pub mod signals {
    use actix::prelude::*;

    pub mod outbound {
        use super::*;

        #[derive(Message)]
        #[rtype(result = "()")]
        pub struct PortChanged(pub u16);
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortChangeSubscribe(pub Recipient<outbound::PortChanged>);

    #[derive(Message)]
    #[rtype(result = "()")]
    pub(super) struct ProcessRestarted(pub u16);
}

pub struct ReplicaActorConfig {
    pub ic_starter_path: PathBuf,
    pub replica_path: PathBuf,
    pub state_directory: PathBuf,
    pub write_port_to: PathBuf,
    pub write_pid_to: Option<PathBuf>,
    pub no_artificial_delay: bool,
    pub shutdown_controller: Option<Addr<ShutdownController>>,
}

/// Runs a IC replica as a child process and emits the port.
pub struct ReplicaActor {
    port: Option<u16>,
    config: ReplicaActorConfig,
    spawn_actor: Option<Addr<ChildProcessActor>>,
    subscribers: Vec<Recipient<signals::outbound::PortChanged>>,
}

impl ReplicaActor {
    pub fn new(config: ReplicaActorConfig) -> Self {
        Self {
            port: None,
            config,
            spawn_actor: None,
            subscribers: Vec::new(),
        }
    }

    fn start_replica(&mut self, addr: Addr<ReplicaActor>) {
        let command = Command::from(&self.config);
        let port_file = self.config.write_port_to.clone();

        let handle_restart = move |kill_receiver: &Receiver<()>| {
            // To make sure that the port we read is for this process.
            fs::write(&port_file, "").expect("Cannot write to the port file");

            log::trace!("Replica command executed. Now checking the port file...");
            log::trace!("Reading port file: {:?}", port_file);

            let mut waiter = Delay::builder()
                .throttle(Duration::from_millis(100))
                .timeout(Duration::from_secs(10))
                .build();

            waiter.start();

            loop {
                if let Ok(content) = std::fs::read_to_string(&port_file) {
                    if let Ok(port) = content.parse::<u16>() {
                        log::info!("Replica is listening on port {}", port);
                        addr.do_send(signals::ProcessRestarted(port));
                        return;
                    }
                }

                if kill_receiver
                    .recv_timeout(Duration::from_millis(100))
                    .is_ok()
                {
                    return;
                }

                waiter.wait().expect("Can not start the replica.");
            }
        };

        let spawn_actor = ChildProcessActor::new(ChildProcessActorConfig {
            name: "replica".into(),
            command,
            shutdown_controller: self.config.shutdown_controller.take(),
            callback: Some(Box::new(handle_restart)),
            pid_file: self.config.write_pid_to.clone(),
        })
        .start();

        self.spawn_actor = Some(spawn_actor);
    }
}

impl Actor for ReplicaActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_replica(ctx.address());
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        let _ = fs::remove_file(&self.config.write_port_to);
    }
}

impl Handler<signals::PortChangeSubscribe> for ReplicaActor {
    type Result = ();

    fn handle(&mut self, msg: PortChangeSubscribe, _ctx: &mut Self::Context) -> Self::Result {
        // If we already know the port, emit it.
        if let Some(port) = self.port {
            let _ = msg.0.do_send(signals::outbound::PortChanged(port));
        }

        self.subscribers.push(msg.0);
    }
}

impl Handler<signals::ProcessRestarted> for ReplicaActor {
    type Result = ();

    fn handle(&mut self, msg: ProcessRestarted, _ctx: &mut Self::Context) -> Self::Result {
        let port = msg.0;

        if Some(port) == self.port {
            return;
        }

        self.port = Some(port);

        for sub in &self.subscribers {
            let _ = sub.do_send(signals::outbound::PortChanged(port));
        }
    }
}

impl From<&ReplicaActorConfig> for Command {
    fn from(config: &ReplicaActorConfig) -> Self {
        let mut cmd = Command::new(&config.ic_starter_path);

        cmd.args(&[
            "--replica-path",
            config.replica_path.to_str().unwrap_or_default(),
            "--state-dir",
            config.state_directory.to_str().unwrap_or_default(),
            "--create-funds-whitelist",
            "*",
            "--consensus-pool-backend",
            "rocksdb",
        ]);

        cmd.args(&[
            "--http-port-file",
            config.write_port_to.to_str().unwrap_or_default(),
        ]);

        if config.no_artificial_delay {
            cmd.args(&[
                "--initial-notary-delay-millis",
                // The intial notary delay is set to 2500ms in the replica's
                // default subnet configuration.
                // For local consensus, we can set it to a smaller value in order
                // to speed up update calls.
                "500",
            ]);
        }

        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        cmd
    }
}
