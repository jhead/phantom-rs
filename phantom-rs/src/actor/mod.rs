use log::debug;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::task::CancellableTask;

/// Trait for async behavior that can process messages by mutating state
pub trait AsyncBehavior<Message: Send + 'static, State>: Send + Sync {
    fn handle(
        &self,
        self_ref: ActorRef<Message>,
        message: Message,
        state: State,
    ) -> Pin<Box<dyn Future<Output = State> + Send>>;
}

/// A simple wrapper that implements AsyncBehavior for a function
pub struct SimpleBehavior<F> {
    handler: F,
}

impl<Message, State, F, Fut> AsyncBehavior<Message, State> for SimpleBehavior<F>
where
    Message: Send + 'static,
    State: Clone + Send + Sync + 'static,
    F: Fn(ActorRef<Message>, Message, State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = State> + Send + 'static,
{
    fn handle(
        &self,
        self_ref: ActorRef<Message>,
        message: Message,
        state: State,
    ) -> Pin<Box<dyn Future<Output = State> + Send>> {
        Box::pin((self.handler)(self_ref, message, state))
    }
}

/// The behavior function type that processes messages by mutating state asynchronously
pub type BehaviorFn<Message, State> = Box<dyn AsyncBehavior<Message, State>>;

/// Helper function to create a behavior from an async closure
pub fn behavior<Message, State, F, Fut>(handler: F) -> BehaviorFn<Message, State>
where
    Message: Send + 'static,
    State: Clone + Send + Sync + 'static,
    F: Fn(ActorRef<Message>, Message, State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = State> + Send + 'static,
{
    Box::new(SimpleBehavior { handler })
}
pub struct Actor<Message: Send + 'static, State: Clone + Send + 'static> {
    behavior: BehaviorFn<Message, State>,
    sender: mpsc::UnboundedSender<ActorSignal<Message>>,
    receiver: mpsc::UnboundedReceiver<ActorSignal<Message>>,
}

#[derive(Debug, Clone)]
pub struct ActorRef<Message: Send + 'static> {
    sender: mpsc::UnboundedSender<ActorSignal<Message>>,
}

#[derive(Debug, Error)]
pub enum ActorError {
    #[error("Actor is already running")]
    AlreadyRunning,

    #[error("Failed to send message: {0}")]
    FailedToSend(String),
}

impl<Message: Send + 'static> ActorRef<Message> {
    pub fn send(&self, message: Message) -> Result<(), ActorError> {
        self.sender
            .send(ActorSignal::Message(message))
            .map_err(|e| ActorError::FailedToSend(e.to_string()))
    }

    pub fn shutdown(&self) {
        let _ = self.sender.send(ActorSignal::Shutdown);
    }

    // Create a new Actor and attach it as a child by sending a message to the parent
    pub fn run_child<State>(&self, initial_state: State, behavior: BehaviorFn<Message, State>)
    where
        State: Send + Clone + 'static,
    {
        let child_actor = Actor::run(initial_state, behavior);
        self.attach_child(child_actor);
    }

    pub fn attach_child(&self, child: impl CancellableTask) {
        self.sender
            .send(ActorSignal::SpawnChild(Box::new(child)))
            .unwrap_or_else(|e| {
                debug!("[actor] failed to attach child task: {}", e);
            });

        debug!("[actor] child task attached successfully");
    }
}

struct ActorInternalState<State: Clone + Send + 'static> {
    children: Vec<Box<dyn CancellableTask>>,
    state: State,
}

enum ActorSignal<Message: Send + 'static> {
    Message(Message),
    SpawnChild(Box<dyn CancellableTask>),
    Shutdown,
}

pub struct RunningActor<Message: Send + 'static> {
    actor_ref: ActorRef<Message>,
    join_handle: JoinHandle<()>,
}

impl<Message: Send + 'static> Deref for RunningActor<Message> {
    type Target = ActorRef<Message>;

    fn deref(&self) -> &Self::Target {
        &self.actor_ref
    }
}

impl<Message: Send + 'static> CancellableTask for RunningActor<Message> {
    fn cancel(&self) {
        self.actor_ref.shutdown();
    }

    fn join(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            let _ = self.join_handle.await;
        })
    }
}

impl<Message: Send + 'static, State: Clone + Send + 'static> Actor<Message, State> {
    /// Create a new Actor with initial state and behavior
    pub fn run(
        initial_state: State,
        behavior: BehaviorFn<Message, State>,
    ) -> RunningActor<Message> {
        let (sender, receiver) = mpsc::unbounded_channel();

        let actor = Self {
            behavior,
            sender,
            receiver,
        };

        let actor_ref = ActorRef {
            sender: actor.sender.clone(),
        };

        let join_handle = tokio::spawn(async move {
            actor.run_loop(initial_state).await;
        });

        RunningActor {
            actor_ref,
            join_handle,
        }
    }

    /// Process one message from the channel, waiting if necessary
    async fn process_one(&mut self, internal_state: &mut ActorInternalState<State>) -> bool {
        let incoming = self.receiver.recv().await;
        match incoming {
            Some(ActorSignal::Message(message)) => {
                let new_state = self
                    .behavior
                    .handle(
                        ActorRef {
                            sender: self.sender.clone(),
                        },
                        message,
                        internal_state.state.clone(),
                    )
                    .await;
                internal_state.state = new_state;
                true
            }
            Some(ActorSignal::SpawnChild(child_task)) => {
                debug!("[actor] spawning child task");
                internal_state.children.push(child_task);
                true
            }
            Some(ActorSignal::Shutdown) => false,
            None => false,
        }
    }

    /// Run the actor in a continuous loop, processing messages as they arrive
    async fn run_loop(mut self, initial_state: State) {
        let mut state = ActorInternalState {
            state: initial_state,
            children: Vec::new(),
        };

        while self.process_one(&mut state).await {}
        debug!("[actor] shutting down children");

        for child in state.children {
            child.cancel();
            child.join().await;
        }

        debug!("[actor] shut down gracefully");
    }
}
