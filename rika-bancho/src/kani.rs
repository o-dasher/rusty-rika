use std::sync::Arc;

use async_callable::AsyncCallable1;
use itertools::Itertools;
use nasus::{CmdIn, CmdOut, Nasus};
use strum::Display;
use tokio::sync::Mutex;

pub type KaniResult = Result<(), Box<dyn std::error::Error + Sync + Send>>;

pub struct KaniContext<D> {
    pub irc: Arc<Mutex<Nasus>>,
    pub args: Vec<String>,
    pub sender: String,
    pub data: Arc<D>,
}

#[derive(thiserror::Error, Debug, Display)]
pub enum WorkaroundError {
    Fucked,
}

pub type CommandDefiner<'a, D> = Vec<(
    Vec<&'static str>,
    &'a dyn AsyncCallable1<'a, KaniContext<D>, KaniResult>,
)>;

pub struct KaniFramework<'a, D> {
    pub config: nasus::BanchoConfig,
    pub data: D,
    pub prefix: String,
    pub commands: CommandDefiner<'a, D>,
}

pub struct KaniWorker<'a, D> {
    pub irc: Arc<Mutex<Nasus>>,
    pub data: Arc<D>,
    pub prefix: String,
    pub commands: CommandDefiner<'a, D>,
}

impl<'a, D> KaniWorker<'a, D> {
    async fn handle_input(&self, cmd_in: CmdIn) -> KaniResult {
        let CmdIn::ReceivePM { sender,  message, .. } = cmd_in else {
            return Ok(());
        };

        if !message.starts_with(&self.prefix) {
            return Ok(());
        }

        let cmd_msg = message.replacen(&self.prefix, "", 1);

        let cmd_args_vec = cmd_msg.split(' ').collect_vec();
        let cmd_args = &cmd_args_vec[..];

        let [name, args @ ..] = cmd_args else {
            return Ok(());
        };

        let possible_cmd = self
            .commands
            .iter()
            .find(|(names, ..)| names.contains(name));

        let Some((.., runnable)) = possible_cmd else {
            return Ok(());
        };

        runnable
            .call(KaniContext {
                irc: self.irc.clone(),
                args: args.iter().map(ToString::to_string).collect_vec(),
                sender,
                data: self.data.clone(),
            })
            .await?;

        Ok(())
    }

    async fn handle_output(&self, _cmd_out: CmdOut) -> KaniResult {
        Ok(())
    }
}

impl<'a, D> KaniFramework<'a, D> {
    pub async fn run(self) -> KaniResult {
        let kani_irc = Arc::new(Mutex::new(
            Nasus::new(self.config)
                .await
                .map_err(|_| WorkaroundError::Fucked)?,
        ));

        let kani_worker = KaniWorker {
            irc: kani_irc.clone(),
            data: Arc::new(self.data),
            prefix: self.prefix,
            commands: self.commands,
        };

        loop {
            let (last_input, last_output) = {
                let mut locking = kani_irc.lock().await;

                locking.work().await.map_err(|_| WorkaroundError::Fucked)?;

                (locking.inputs.pop(), locking.outputs.pop())
            };

            if let Some(last_input) = last_input {
                kani_worker.handle_input(last_input).await?;
            }

            if let Some(last_output) = last_output {
                kani_worker.handle_output(last_output).await?;
            };
        }
    }
}
