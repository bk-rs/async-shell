/*
cargo run -p async-shell-demo-smol --bin sample
*/

use std::io;
use std::process::Command;
use std::sync::Arc;
use std::thread;

use async_executor::{Executor, LocalExecutor, Task};
use easy_parallel::Parallel;

use async_shell::ChildExt;

fn main() -> io::Result<()> {
    let ex = Executor::new();
    let ex = Arc::new(ex);
    let local_ex = LocalExecutor::new();
    let (trigger, shutdown) = async_channel::unbounded::<()>();

    let ret_vec: (_, io::Result<()>) = Parallel::new()
        .each(0..2, |_| {
            ex.clone().run(async {
                shutdown
                    .recv()
                    .await
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            })
        })
        .finish(|| {
            local_ex.run(async {
                run(ex.clone()).await?;

                drop(trigger);

                Ok(())
            })
        });

    println!("ret_vec: {:?}", ret_vec);

    Ok(())
}

async fn run(ex: Arc<Executor>) -> io::Result<()> {
    let mut receivers = vec![];
    for i in 0..10 {
        let (sender, receiver) = async_channel::unbounded();
        receivers.push(receiver);

        let task: Task<io::Result<()>> = ex.spawn(async move {
            println!("{} {:?} run", i, thread::current().id());

            let output = match i % 3 {
                0 => {
                    Command::new("sleep")
                        .arg(format!("{}", i))
                        .spawn_async(None, None, None)
                        .await?
                }
                1 => {
                    Command::new("echo")
                        .arg(format!("{}", i))
                        .spawn_async(None, None, None)
                        .await?
                }
                2 => {
                    Command::new("ruby")
                        .arg("-e")
                        .arg(format!("STDERR.puts {}", i))
                        .spawn_async(None, None, None)
                        .await?
                }
                _ => unreachable!(),
            };

            println!("{} {:?} output: {:?}", i, thread::current().id(), output);

            Ok(())
        });

        ex.spawn(async move {
            task.await
                .unwrap_or_else(|err| eprintln!("task {} failed, err: {}", i, err));

            sender.send(format!("task {} done", i)).await.unwrap()
        })
        .detach();
    }

    for receiver in receivers {
        let msg = receiver.recv().await.unwrap();
        println!("{}", msg);
    }

    Ok(())
}
