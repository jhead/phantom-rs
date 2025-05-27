use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;

/// Trait for async behavior that can process messages by mutating state
pub trait AsyncBehavior<Message, State>: Send + Sync {
    fn handle(&self, message: Message, state: State)
        -> Pin<Box<dyn Future<Output = State> + Send>>;
}

/// A simple wrapper that implements AsyncBehavior for a function
pub struct SimpleBehavior<F> {
    handler: F,
}

impl<Message, State, F, Fut> AsyncBehavior<Message, State> for SimpleBehavior<F>
where
    Message: Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    F: Fn(Message, State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = State> + Send + 'static,
{
    fn handle(
        &self,
        message: Message,
        state: State,
    ) -> Pin<Box<dyn Future<Output = State> + Send>> {
        Box::pin((self.handler)(message, state))
    }
}

/// The behavior function type that processes messages by mutating state asynchronously
pub type BehaviorFn<Message, State> = Box<dyn AsyncBehavior<Message, State>>;

/// Helper function to create a behavior from an async closure
pub fn behavior<Message, State, F, Fut>(handler: F) -> BehaviorFn<Message, State>
where
    Message: Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    F: Fn(Message, State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = State> + Send + 'static,
{
    Box::new(SimpleBehavior { handler })
}
pub struct Actor<Message, State: Clone + Send + 'static> {
    is_running: Arc<AtomicBool>,
    state: State,
    behavior: BehaviorFn<Message, State>,
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>,
}

#[derive(Debug, Clone)]
pub struct ActorRef<Message> {
    sender: mpsc::UnboundedSender<Message>,
}

#[derive(Debug, Error)]
pub enum ActorError {
    #[error("Actor is already running")]
    AlreadyRunning,
}

impl<Message: Send + 'static> ActorRef<Message> {
    /// Send a message to the actor
    pub fn send(&self, message: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(message)
    }
}

impl<Message: Send + 'static, State: Clone + Send + 'static> Actor<Message, State> {
    /// Create a new Actor with initial state and behavior
    pub fn new(initial_state: State, behavior: BehaviorFn<Message, State>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let actor = Self {
            is_running: Arc::new(AtomicBool::new(false)),
            state: initial_state,
            behavior,
            sender,
            receiver,
        };

        actor
    }

    /// Process one message from the channel, waiting if necessary
    async fn process_one(&mut self) -> bool {
        if let Some(message) = self.receiver.recv().await {
            let state_clone = self.state.clone();
            let new_state = self.behavior.handle(message, state_clone).await;
            self.state = new_state;
            true
        } else {
            false
        }
    }

    pub fn start(mut self) -> Result<ActorRef<Message>, ActorError> {
        if self.is_running.load(Ordering::SeqCst) {
            eprintln!("Actor is already running");
            return Err(ActorError::AlreadyRunning);
        }

        self.is_running.store(true, Ordering::SeqCst);
        let sender = self.sender.clone();

        tokio::spawn(async move {
            self.run_loop().await;
        });

        Ok(ActorRef { sender })
    }

    /// Run the actor in a continuous loop, processing messages as they arrive
    async fn run_loop(&mut self) {
        while self.process_one().await {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[derive(Debug, Clone, PartialEq)]
    struct CounterState {
        count: i32,
    }

    // Define message types for the counter
    #[derive(Debug, Clone, Copy)]
    enum CounterMessage {
        Increment,
        Add(i32),
        Reset,
    }

    #[tokio::test]
    async fn test_simple_counter_actor() {
        // Create a counter that handles different message types
        let initial_state = CounterState { count: 0 };
        let behavior = behavior(
            |message: CounterMessage, mut state: CounterState| async move {
                // Immediately extract values from the message to avoid borrowing issues
                match message {
                    CounterMessage::Increment => state.count += 1,
                    CounterMessage::Add(n) => state.count += n,
                    CounterMessage::Reset => state.count = 0,
                }

                state
            },
        );

        let mut actor = Actor::new(initial_state, behavior);
        let sender = &actor.sender;

        // Send some messages
        sender.send(CounterMessage::Increment).unwrap();
        sender.send(CounterMessage::Add(5)).unwrap();
        sender.send(CounterMessage::Increment).unwrap();

        // Process messages one by one
        actor.process_one().await;
        assert_eq!(actor.state.count, 1);

        actor.process_one().await;
        assert_eq!(actor.state.count, 6);

        actor.process_one().await;
        assert_eq!(actor.state.count, 7);
    }

    #[tokio::test]
    async fn test_actor_run_loop() {
        let initial_state = CounterState { count: 0 };
        let behavior = behavior(
            |message: CounterMessage, mut state: CounterState| async move {
                match message {
                    CounterMessage::Increment => state.count += 1,
                    CounterMessage::Add(n) => state.count += n,
                    CounterMessage::Reset => state.count = 0,
                }

                state
            },
        );

        let mut actor = Actor::new(initial_state, behavior);
        let sender = &actor.sender;

        // Send some messages
        sender.send(CounterMessage::Increment).unwrap();
        sender.send(CounterMessage::Add(10)).unwrap();
        sender.send(CounterMessage::Reset).unwrap();
        sender.send(CounterMessage::Add(42)).unwrap();

        // Drop the sender to close the channel after sending messages
        drop(sender);

        // Run the actor loop - it will process all messages and then exit
        actor.run_loop().await;

        // Final state should be 42 (after reset and add 42)
        assert_eq!(actor.state.count, 42);
    }

    #[tokio::test]
    async fn test_actor_timeout() {
        let initial_state = CounterState { count: 0 };
        let behavior =
            behavior(|_message: CounterMessage, state: CounterState| async move { state });

        let mut actor = Actor::new(initial_state, behavior);
        // Don't send any messages and drop sender immediately

        // process_one should timeout since no messages will be sent
        let result = timeout(Duration::from_millis(100), actor.process_one()).await;
        assert!(result.is_err()); // Should timeout
    }
}
