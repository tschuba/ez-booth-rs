use crate::models::{Event, EventReport, Transaction};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use super::{EventStorage, StorageResult};

/// Service for managing events and transactions
pub struct EventService {
    storage: Arc<dyn EventStorage>,
}

impl EventService {
    pub fn new(storage: Arc<dyn EventStorage>) -> Self {
        Self { storage }
    }

    pub fn create_event(&self, name: String) -> StorageResult<Event> {
        let event = Event::new(name);
        self.storage.save_event(&event)?;
        Ok(event)
    }

    pub fn get_event(&self, id: &Uuid) -> StorageResult<Event> {
        self.storage.load_event(id)
    }

    pub fn get_all_events(&self) -> StorageResult<Vec<Event>> {
        self.storage.load_all_events()
    }

    pub fn add_transaction(&self, event_id: &Uuid, transaction: Transaction) -> StorageResult<Event> {
        let mut event = self.storage.load_event(event_id)?;
        event.add_transaction(transaction);
        self.storage.save_event(&event)?;
        Ok(event)
    }

    pub fn delete_event(&self, id: &Uuid) -> StorageResult<()> {
        self.storage.delete_event(id)
    }

    pub fn generate_report(&self, event_id: &Uuid, commission_rate: Decimal) -> StorageResult<EventReport> {
        let event = self.storage.load_event(event_id)?;
        Ok(EventReport::from_event(&event, commission_rate))
    }

    pub fn export_data(&self) -> StorageResult<String> {
        self.storage.export_data()
    }

    pub fn import_data(&self, data: &str) -> StorageResult<Vec<Event>> {
        self.storage.import_data(data)
    }
}
