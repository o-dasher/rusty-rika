use std::fmt::{Debug, Formatter};
use std::{error::Error, sync::Arc};

use async_callable::AsyncCallable1;
use itertools::Itertools;
use nasus::{CmdIn, CmdOut, Nasus};
use strum::Display;
use tokio::sync::Mutex;

pub type BoxedError = Box<dyn Error + Sync + Send>;

#[derive(thiserror::Error, Display)]
pub enum KaniError<D, E> {
    CommandError(E, Arc<KaniContext<D>>),

    #[error(transparent)]
    OutputError(E),

    #[error(transparent)]
    SetupError(BoxedError),

    Fucked,
}

impl<D, E: Debug> Debug for KaniError<D, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandError(..) => f.debug_tuple("CommandError").finish(),
            Self::OutputError(arg0) => f.debug_tuple("OutputError").field(arg0).finish(),
            Self::SetupError(arg0) => f.debug_tuple("SetupError").field(arg0).finish(),
            Self::Fucked => write!(f, "Fucked"),
        }
    }
}

pub type KaniResult<D, E> = Result<(), KaniError<D, E>>;

pub struct KaniContext<D> {
    pub irc: Arc<Mutex<Nasus>>,
    pub args: Vec<String>,
    pub sender: String,
    pub data: Arc<D>,
}

impl<D> KaniContext<D> {
    pub async fn say(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let Self { sender, irc, .. } = self;

        irc.lock()
            .await
            .write_command(CmdOut::SendPM {
                receiver: sender.to_string(),
                message: text.to_string(),
            })
            .await
    }
}

pub type CommandDefiner<'a, D, E> = Vec<(
    Vec<&'static str>,
    &'a dyn AsyncCallable1<'a, Arc<KaniContext<D>>, Result<(), E>>,
)>;

pub type OnErrorHandler<'a, D, E> = &'a dyn AsyncCallable1<'a, KaniError<D, E>, Result<(), E>>;

pub struct KaniFramework<'a, D, E> {
    pub config: nasus::BanchoConfig,
    pub data: D,
    pub prefix: String,
    pub commands: CommandDefiner<'a, D, E>,
    pub on_error: OnErrorHandler<'a, D, E>,
}

pub struct KaniWorker<'a, D, E> {
    pub irc: Arc<Mutex<Nasus>>,
    pub data: Arc<D>,
    pub prefix: String,
    pub commands: CommandDefiner<'a, D, E>,
    pub on_error: OnErrorHandler<'a, D, E>,
}

impl<'a, D: Send + Sync, E> KaniWorker<'a, D, E> {
    async fn handle_input(&self, cmd_in: CmdIn) -> Result<(), (E, Arc<KaniContext<D>>)> {
        let CmdIn::ReceivePM {
            sender, message, ..
        } = cmd_in
        else {
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

        let ctx = Arc::new(KaniContext {
            irc: self.irc.clone(),
            args: args.iter().map(ToString::to_string).collect_vec(),
            sender,
            data: self.data.clone(),
        });

        let result_call = runnable.call(ctx.clone()).await;

        if let Err(e) = result_call {
            return Err((e, ctx));
        }

        Ok(())
    }

    async fn handle_output(&self, _cmd_out: CmdOut) -> Result<(), E> {
        Ok(())
    }
}

impl<'a, D: Send + Sync + 'static, E: Error + Send + Sync + 'static> KaniFramework<'a, D, E> {
    pub async fn run(self) -> KaniResult<D, E> {
        let kani_irc =
            Arc::new(Mutex::new(Nasus::new(self.config).await.map_err(|_| {
                KaniError::SetupError(Box::new(KaniError::<D, E>::Fucked))
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
                if let Err((e, ctx)) = kani_worker.handle_input(last_input).await {
                    // handle error handling error must.
                    let _ = kani_worker
                        .on_error
                        .call(KaniError::CommandError(e, ctx))
                        .await;
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
