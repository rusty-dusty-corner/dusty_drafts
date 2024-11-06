
std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

// Message types for our actor system
#[derive(Debug)]
enum WorkerMessage {
    DoWork(u32),
    Crash,
    Status,
}

#[derive(Debug)]
enum SupervisorMessage {
    WorkerResult(u32),
    WorkerError(String),
}

// Worker actor - similar to Erlang process
async fn worker_actor(
    mut receiver: mpsc::Receiver<WorkerMessage>,
    supervisor: mpsc::Sender<SupervisorMessage>,
) {
    println!("Worker started!");

    while let Some(msg) = receiver.recv().await {
        match msg {
            WorkerMessage::DoWork(n) => {
                println!("Worker processing: {}", n);
                sleep(Duration::from_millis(500)).await;

                let result = n * 2;
                if let Err(_) = supervisor
                    .send(SupervisorMessage::WorkerResult(result))
                    .await
                {
                    println!("Supervisor appears to be down!");
                    return;
                }
            }
            WorkerMessage::Crash => {
                println!("Worker is crashing!");
                if let Err(_) = supervisor
                    .send(SupervisorMessage::WorkerError(
                        "Worker crashed!".to_string(),
                    ))
                    .await
                {
                    println!("Failed to notify supervisor about crash!");
                }
                return;
            }
            WorkerMessage::Status => {
                println!("Worker is healthy!");
            }
        }
    }
}

// Supervisor actor - similar to Erlang supervisor
async fn supervisor(worker_count: u32) {
    println!("Supervisor started!");

    let (sup_tx, mut sup_rx) = mpsc::channel::<SupervisorMessage>(100);
    let mut workers = vec![];

    // Start initial workers
    for id in 0..worker_count {
        let (worker_tx, worker_rx) = mpsc::channel::<WorkerMessage>(100);
        let sup_tx_clone = sup_tx.clone();

        // Spawn worker
        let worker_handle = tokio::spawn(worker_actor(worker_rx, sup_tx_clone));
        workers.push((worker_tx, worker_handle));

        // Send initial work
        let tx = &workers[id as usize].0;
        tx.send(WorkerMessage::DoWork(id)).await.unwrap();
    }

    // Supervision loop
    while let Some(msg) = sup_rx.recv().await {
        match msg {
            SupervisorMessage::WorkerResult(n) => {
                println!("Supervisor received result: {}", n);
            }
            SupervisorMessage::WorkerError(err) => {
                println!("Supervisor received error: {}", err);
                println!("Restarting worker...");

                // Restart the worker
                let (worker_tx, worker_rx) = mpsc::channel::<WorkerMessage>(100);
                let sup_tx_clone = sup_tx.clone();
                let worker_handle = tokio::spawn(worker_actor(worker_rx, sup_tx_clone));
                workers.push((worker_tx, worker_handle));

                // Send test message to new worker
                let tx = workers.last().unwrap().0.clone();
                tx.send(WorkerMessage::Status).await.unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting actor system...");

    // Start supervisor with 2 workers
    let supervisor_handle = tokio::spawn(supervisor(2));

    // Let the system run for a while
    sleep(Duration::from_secs(2)).await;

    // Keep main alive
    supervisor_handle.await.unwrap();
}
