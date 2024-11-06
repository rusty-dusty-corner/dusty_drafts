use std::error::Error;
use std::fmt;
use tokio::sync::mpsc;

// Abstract task definition
trait Task: Send + Sync + 'static {
    type Input;
    type Output;
    type Error: Error + Send;

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

// Error types
#[derive(Debug)]
struct ProcessingError(String);

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Processing error: {}", self.0)
    }
}

impl Error for ProcessingError {}

// Data source trait
trait DataSource: Send + Sync + 'static {
    type Item;
    type Error: Error + Send;

    fn get_data(&self) -> Result<Self::Item, Self::Error>;
}

// Concrete implementation for pixel processing task
#[derive(Clone)]
struct BluePixelTask;

impl Task for BluePixelTask {
    type Input = Vec<u8>;
    type Output = Option<(u32, u32)>;
    type Error = ProcessingError;

    fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let width = 100; // Assuming fixed dimensions
        let height = 100;

        for y in 0..height {
            for x in 0..width {
                let pos = ((y * width + x) * 3) as usize;
                if pos + 2 < input.len() {
                    let (r, g, b) = (input[pos], input[pos + 1], input[pos + 2]);
                    if b > 200 && r < 100 && g < 100 {
                        return Ok(Some((x, y)));
                    }
                }
            }
        }
        Ok(None)
    }
}

// Mock task implementation
#[derive(Clone)]
struct MockTask;

impl Task for MockTask {
    type Input = Vec<u8>;
    type Output = Option<(u32, u32)>;
    type Error = ProcessingError;

    fn process(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(Some((42, 42)))
    }
}

// Mock data source implementation
#[derive(Clone)]
struct MockImageSource;

impl DataSource for MockImageSource {
    type Item = Vec<u8>;
    type Error = ProcessingError;

    fn get_data(&self) -> Result<Self::Item, Self::Error> {
        Ok(vec![0u8; 30000]) // Mock 100x100x3 RGB image
    }
}

// Message types for the processing system
#[derive(Debug)]
enum SystemMessage {
    ProcessingResult(Result<Option<(u32, u32)>, ProcessingError>),
    Completed,
}

// Processing system
struct ProcessingSystem<T, D>
where
    T: Task<Input = Vec<u8>, Output = Option<(u32, u32)>, Error = ProcessingError>,
    D: DataSource<Item = Vec<u8>, Error = ProcessingError>,
{
    task: T,
    data_source: D,
}

impl<T, D> ProcessingSystem<T, D>
where
    T: Clone + Task<Input = Vec<u8>, Output = Option<(u32, u32)>, Error = ProcessingError>,
    D: Clone + DataSource<Item = Vec<u8>, Error = ProcessingError>,
{
    fn new(task: T, data_source: D) -> Self {
        Self { task, data_source }
    }

    async fn run(&self, num_workers: usize) {
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn workers
        for _ in 0..num_workers {
            let tx = tx.clone();
            let task = self.task.clone();
            let data_source = self.data_source.clone();

            tokio::spawn(async move {
                match data_source.get_data() {
                    Ok(data) => {
                        let result = task.process(data);
                        let _ = tx.send(SystemMessage::ProcessingResult(result)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(SystemMessage::ProcessingResult(Err(e))).await;
                    }
                }
                let _ = tx.send(SystemMessage::Completed).await;
            });
        }

        // Process results
        let mut completed = 0;
        while let Some(msg) = rx.recv().await {
            match msg {
                SystemMessage::ProcessingResult(Ok(Some(pos))) => {
                    println!("Found blue pixel at: {:?}", pos);
                }
                SystemMessage::ProcessingResult(Ok(None)) => {
                    println!("No blue pixel found");
                }
                SystemMessage::ProcessingResult(Err(e)) => {
                    println!("Error: {}", e);
                }
                SystemMessage::Completed => {
                    completed += 1;
                    if completed == num_workers {
                        break;
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Create system with real implementation
    let system = ProcessingSystem::new(BluePixelTask, MockImageSource);

    println!("Starting processing system...");
    system.run(4).await; // Run with 4 workers
}
