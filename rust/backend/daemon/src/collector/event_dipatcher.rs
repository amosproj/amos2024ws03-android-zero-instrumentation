use async_broadcast::Sender;
use ractor::Actor;
use shared::ziofa::Event;
use tonic::Status;



pub struct EventDispatcher;

pub struct EventDispatcherState {
    destination: Sender<Result<Event, Status>>,
}

impl EventDispatcherState {
    pub fn new(destination: Sender<Result<Event, Status>>) -> Self {
        Self { destination }
    }
}


impl Actor for EventDispatcher {
    type Arguments = EventDispatcherState;
    type Msg = Event;
    type State = EventDispatcherState;
    
    async fn pre_start(
            &self,
            _: ractor::ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(args)
    }
    
    async fn handle(
            &self,
            _: ractor::ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ractor::ActorProcessingErr> {
        state.destination.broadcast_direct(Ok(message))
            .await?;
        
        Ok(())
    }
}