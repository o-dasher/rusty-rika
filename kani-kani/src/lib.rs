use std::{error::Error, sync::Arc};

use async_callable::AsyncCallable1;
use itertools::Itertools;
use nasus::{CmdIn, CmdOut, Nasus};
use strum::Display;
use tokio::sync::Mutex;

pub type BoxedError = Box<dyn std::error::Error + Sync + Send>;

#[derive(thiserror::Error, Debug, Display)]
pub enum KaniError<E> {
    #[error(transparent)]
    CommandError(E),

    #[error(transparent)]
    OutputError(E),

    #[error(transparent)]
    SetupError(BoxedError),

    Fucked,
}

pub type KaniResult<E> = Result<(), KaniError<E>>;

pub struct KaniContext<D> {
    pub irc: Arc<Mutex<Nasus>>,
    pub args: Vec<String>,
    pub sender: String,
    pub data: Arc<D>,
}

pub type CommandDefiner<'a, D, E> = Vec<(
    Vec<&'static str>,
    &'a dyn AsyncCallable1<'a, KaniContext<D>, Result<(), E>>,
)>;

pub type OnErrorHandler<'a, E> = &'a dyn AsyncCallable1<'a, KaniError<E>, Result<(), E>>;

pub struct KaniFramework<'a, D, E> {
    pub config: nasus::BanchoConfig,
    pub data: D,
    pub prefix: String,
    pub commands: CommandDefiner<'a, D, E>,
    pub on_error: OnErrorHandler<'a, E>,
}

pub struct KaniWorker<'a, D, E> {
    pub irc: Arc<Mutex<Nasus>>,
    pub data: Arc<D>,
    pub prefix: String,
    pub commands: CommandDefiner<'a, D, E>,
    pub on_error: OnErrorHandler<'a, E>,
}

impl<'a, D, E> KaniWorker<'a, D, E> {
    async fn handle_input(&self, cmd_in: CmdIn) -> Result<(), E> {
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

    async fn handle_output(&self, _cmd_out: CmdOut) -> Result<(), E> {
        Ok(())
    }
}

impl<'a, D, E: Error + Send + Sync + 'static> KaniFramework<'a, D, E> {
    pub async fn run(self) -> KaniResult<E> {
        let kani_irc =
            Arc::new(Mutex::new(Nasus::new(self.config).await.map_err(|_| {
                KaniError::SetupError(Box::new(KaniError::<E>::Fucked))
            })?));

        let kani_worker = KaniWorker {
            irc: kani_irc.clone(),
            data: Arc::new(self.data),
            prefix: self.prefix,
            commands: self.commands,
            on_error: self.on_error,
        };

        loop {
            let (last_input, last_output) = {
                let mut locking = kani_irc.lock().await;

                locking.work().await.map_err(|_| KaniError::Fucked)?;

                (locking.inputs.pop(), locking.outputs.pop())
            };

            if let Some(last_input) = last_input {
                if let Err(e) = kani_worker.handle_input(last_input).await {
                    // handle error handling error must.
                    let _ = kani_worker.on_error.call(KaniError::CommandError(e)).await;
                }
            }

            if let Some(last_output) = last_output {
                if let Err(e) = kani_worker.handle_output(last_output).await {
                    let _ = kani_worker.on_error.call(KaniError::OutputError(e)).await;
                }
            };
        }
    }
}
