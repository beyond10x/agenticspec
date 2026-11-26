// generated from sessionspec v1
// contract digest 5a5e91b41741f78d8f88d9bbbc3f8590bdcf3ccc51173eb11b074eef53627627
// do not edit: regenerate with `ess synthesize`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `sessionspec.session.Agent`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryAgentStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::session::AgentSnapshot>>>);
impl sessionspec_types::behaviour::AgentStorage for MemoryAgentStorage {
fn get(&self, identity: &sessionspec_types::session::AgentId) -> Option<sessionspec_types::session::AgentSnapshot> { let key = memory_key_44(identity); self.0.borrow().iter().find(|row| memory_key_44(&row.data.agent_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::session::AgentSnapshot) { self.delete(&snapshot.data.agent_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_44(&row.data.agent_id).cmp(&memory_key_44(&other.data.agent_id))); }
fn delete(&mut self, identity: &sessionspec_types::session::AgentId) { let key = memory_key_44(identity); self.0.borrow_mut().retain(|row| memory_key_44(&row.data.agent_id) != key); }
fn list(&self) -> Vec<sessionspec_types::session::AgentSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::AgentStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::session::AgentId) -> Option<sessionspec_types::session::AgentSnapshot> { sessionspec_types::behaviour::AgentStorage::get(&self.agent_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::session::AgentSnapshot) { sessionspec_types::behaviour::AgentStorage::put(&mut self.agent_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::session::AgentId) { sessionspec_types::behaviour::AgentStorage::delete(&mut self.agent_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::session::AgentSnapshot> { sessionspec_types::behaviour::AgentStorage::list(&self.agent_storage) }
}

/// Ephemeral storage of `sessionspec.session.Exposure`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryExposureStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::session::ExposureSnapshot>>>);
impl sessionspec_types::behaviour::ExposureStorage for MemoryExposureStorage {
fn get(&self, identity: &sessionspec_types::session::ExposureId) -> Option<sessionspec_types::session::ExposureSnapshot> { let key = memory_key_53(identity); self.0.borrow().iter().find(|row| memory_key_53(&row.data.exposure_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::session::ExposureSnapshot) { self.delete(&snapshot.data.exposure_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_53(&row.data.exposure_id).cmp(&memory_key_53(&other.data.exposure_id))); }
fn delete(&mut self, identity: &sessionspec_types::session::ExposureId) { let key = memory_key_53(identity); self.0.borrow_mut().retain(|row| memory_key_53(&row.data.exposure_id) != key); }
fn list(&self) -> Vec<sessionspec_types::session::ExposureSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::ExposureStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::session::ExposureId) -> Option<sessionspec_types::session::ExposureSnapshot> { sessionspec_types::behaviour::ExposureStorage::get(&self.exposure_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::session::ExposureSnapshot) { sessionspec_types::behaviour::ExposureStorage::put(&mut self.exposure_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::session::ExposureId) { sessionspec_types::behaviour::ExposureStorage::delete(&mut self.exposure_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::session::ExposureSnapshot> { sessionspec_types::behaviour::ExposureStorage::list(&self.exposure_storage) }
}

/// Ephemeral storage of `sessionspec.session.Session`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySessionStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::session::SessionSnapshot>>>);
impl sessionspec_types::behaviour::SessionStorage for MemorySessionStorage {
fn get(&self, identity: &sessionspec_types::session::SessionId) -> Option<sessionspec_types::session::SessionSnapshot> { let key = memory_key_59(identity); self.0.borrow().iter().find(|row| memory_key_59(&row.data.session_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::session::SessionSnapshot) { self.delete(&snapshot.data.session_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_59(&row.data.session_id).cmp(&memory_key_59(&other.data.session_id))); }
fn delete(&mut self, identity: &sessionspec_types::session::SessionId) { let key = memory_key_59(identity); self.0.borrow_mut().retain(|row| memory_key_59(&row.data.session_id) != key); }
fn list(&self) -> Vec<sessionspec_types::session::SessionSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::SessionStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::session::SessionId) -> Option<sessionspec_types::session::SessionSnapshot> { sessionspec_types::behaviour::SessionStorage::get(&self.session_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::session::SessionSnapshot) { sessionspec_types::behaviour::SessionStorage::put(&mut self.session_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::session::SessionId) { sessionspec_types::behaviour::SessionStorage::delete(&mut self.session_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::session::SessionSnapshot> { sessionspec_types::behaviour::SessionStorage::list(&self.session_storage) }
}

/// Ephemeral storage of `sessionspec.session.ToolCall`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryToolCallStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::session::ToolCallSnapshot>>>);
impl sessionspec_types::behaviour::ToolCallStorage for MemoryToolCallStorage {
fn get(&self, identity: &sessionspec_types::session::ToolCallId) -> Option<sessionspec_types::session::ToolCallSnapshot> { let key = memory_key_62(identity); self.0.borrow().iter().find(|row| memory_key_62(&row.data.tool_call_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::session::ToolCallSnapshot) { self.delete(&snapshot.data.tool_call_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_62(&row.data.tool_call_id).cmp(&memory_key_62(&other.data.tool_call_id))); }
fn delete(&mut self, identity: &sessionspec_types::session::ToolCallId) { let key = memory_key_62(identity); self.0.borrow_mut().retain(|row| memory_key_62(&row.data.tool_call_id) != key); }
fn list(&self) -> Vec<sessionspec_types::session::ToolCallSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::ToolCallStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::session::ToolCallId) -> Option<sessionspec_types::session::ToolCallSnapshot> { sessionspec_types::behaviour::ToolCallStorage::get(&self.tool_call_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::session::ToolCallSnapshot) { sessionspec_types::behaviour::ToolCallStorage::put(&mut self.tool_call_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::session::ToolCallId) { sessionspec_types::behaviour::ToolCallStorage::delete(&mut self.tool_call_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::session::ToolCallSnapshot> { sessionspec_types::behaviour::ToolCallStorage::list(&self.tool_call_storage) }
}

/// Ephemeral storage of `sessionspec.view.AgentListing`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryAgentListingStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::view::AgentListingSnapshot>>>);
impl sessionspec_types::behaviour::AgentListingStorage for MemoryAgentListingStorage {
fn get(&self, identity: &sessionspec_types::session::AgentId) -> Option<sessionspec_types::view::AgentListingSnapshot> { let key = memory_key_44(identity); self.0.borrow().iter().find(|row| memory_key_44(&row.data.agent_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::view::AgentListingSnapshot) { self.delete(&snapshot.data.agent_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_44(&row.data.agent_id).cmp(&memory_key_44(&other.data.agent_id))); }
fn delete(&mut self, identity: &sessionspec_types::session::AgentId) { let key = memory_key_44(identity); self.0.borrow_mut().retain(|row| memory_key_44(&row.data.agent_id) != key); }
fn list(&self) -> Vec<sessionspec_types::view::AgentListingSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::AgentListingStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::session::AgentId) -> Option<sessionspec_types::view::AgentListingSnapshot> { sessionspec_types::behaviour::AgentListingStorage::get(&self.agent_listing_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::view::AgentListingSnapshot) { sessionspec_types::behaviour::AgentListingStorage::put(&mut self.agent_listing_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::session::AgentId) { sessionspec_types::behaviour::AgentListingStorage::delete(&mut self.agent_listing_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::view::AgentListingSnapshot> { sessionspec_types::behaviour::AgentListingStorage::list(&self.agent_listing_storage) }
}

/// Ephemeral storage of `sessionspec.view.EventRow`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryEventRowStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::view::EventRowSnapshot>>>);
impl sessionspec_types::behaviour::EventRowStorage for MemoryEventRowStorage {
fn get(&self, identity: &sessionspec_types::view::EventId) -> Option<sessionspec_types::view::EventRowSnapshot> { let key = memory_key_71(identity); self.0.borrow().iter().find(|row| memory_key_71(&row.data.event_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::view::EventRowSnapshot) { self.delete(&snapshot.data.event_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_71(&row.data.event_id).cmp(&memory_key_71(&other.data.event_id))); }
fn delete(&mut self, identity: &sessionspec_types::view::EventId) { let key = memory_key_71(identity); self.0.borrow_mut().retain(|row| memory_key_71(&row.data.event_id) != key); }
fn list(&self) -> Vec<sessionspec_types::view::EventRowSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::EventRowStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::view::EventId) -> Option<sessionspec_types::view::EventRowSnapshot> { sessionspec_types::behaviour::EventRowStorage::get(&self.event_row_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::view::EventRowSnapshot) { sessionspec_types::behaviour::EventRowStorage::put(&mut self.event_row_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::view::EventId) { sessionspec_types::behaviour::EventRowStorage::delete(&mut self.event_row_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::view::EventRowSnapshot> { sessionspec_types::behaviour::EventRowStorage::list(&self.event_row_storage) }
}

/// Ephemeral storage of `sessionspec.view.SessionCheck`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySessionCheckStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::view::SessionCheckSnapshot>>>);
impl sessionspec_types::behaviour::SessionCheckStorage for MemorySessionCheckStorage {
fn get(&self, identity: &sessionspec_types::view::CheckId) -> Option<sessionspec_types::view::SessionCheckSnapshot> { let key = memory_key_68(identity); self.0.borrow().iter().find(|row| memory_key_68(&row.data.check_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::view::SessionCheckSnapshot) { self.delete(&snapshot.data.check_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_68(&row.data.check_id).cmp(&memory_key_68(&other.data.check_id))); }
fn delete(&mut self, identity: &sessionspec_types::view::CheckId) { let key = memory_key_68(identity); self.0.borrow_mut().retain(|row| memory_key_68(&row.data.check_id) != key); }
fn list(&self) -> Vec<sessionspec_types::view::SessionCheckSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::SessionCheckStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::view::CheckId) -> Option<sessionspec_types::view::SessionCheckSnapshot> { sessionspec_types::behaviour::SessionCheckStorage::get(&self.session_check_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::view::SessionCheckSnapshot) { sessionspec_types::behaviour::SessionCheckStorage::put(&mut self.session_check_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::view::CheckId) { sessionspec_types::behaviour::SessionCheckStorage::delete(&mut self.session_check_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::view::SessionCheckSnapshot> { sessionspec_types::behaviour::SessionCheckStorage::list(&self.session_check_storage) }
}

/// Ephemeral storage of `sessionspec.view.SessionListing`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySessionListingStorage(std::rc::Rc<std::cell::RefCell<Vec<sessionspec_types::view::SessionListingSnapshot>>>);
impl sessionspec_types::behaviour::SessionListingStorage for MemorySessionListingStorage {
fn get(&self, identity: &sessionspec_types::session::SessionId) -> Option<sessionspec_types::view::SessionListingSnapshot> { let key = memory_key_59(identity); self.0.borrow().iter().find(|row| memory_key_59(&row.data.session_id) == key).cloned() }
fn put(&mut self, snapshot: sessionspec_types::view::SessionListingSnapshot) { self.delete(&snapshot.data.session_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_59(&row.data.session_id).cmp(&memory_key_59(&other.data.session_id))); }
fn delete(&mut self, identity: &sessionspec_types::session::SessionId) { let key = memory_key_59(identity); self.0.borrow_mut().retain(|row| memory_key_59(&row.data.session_id) != key); }
fn list(&self) -> Vec<sessionspec_types::view::SessionListingSnapshot> { self.0.borrow().clone() }
}
impl sessionspec_types::behaviour::SessionListingStorage for MemoryPorts {
fn get(&self, identity: &sessionspec_types::session::SessionId) -> Option<sessionspec_types::view::SessionListingSnapshot> { sessionspec_types::behaviour::SessionListingStorage::get(&self.session_listing_storage, identity) }
fn put(&mut self, snapshot: sessionspec_types::view::SessionListingSnapshot) { sessionspec_types::behaviour::SessionListingStorage::put(&mut self.session_listing_storage, snapshot); }
fn delete(&mut self, identity: &sessionspec_types::session::SessionId) { sessionspec_types::behaviour::SessionListingStorage::delete(&mut self.session_listing_storage, identity); }
fn list(&self) -> Vec<sessionspec_types::view::SessionListingSnapshot> { sessionspec_types::behaviour::SessionListingStorage::list(&self.session_listing_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `sessionspec.session.Agent`.
pub agent_storage: MemoryAgentStorage,
/// Storage of `sessionspec.session.Exposure`.
pub exposure_storage: MemoryExposureStorage,
/// Storage of `sessionspec.session.Session`.
pub session_storage: MemorySessionStorage,
/// Storage of `sessionspec.session.ToolCall`.
pub tool_call_storage: MemoryToolCallStorage,
/// Storage of `sessionspec.view.AgentListing`.
pub agent_listing_storage: MemoryAgentListingStorage,
/// Storage of `sessionspec.view.EventRow`.
pub event_row_storage: MemoryEventRowStorage,
/// Storage of `sessionspec.view.SessionCheck`.
pub session_check_storage: MemorySessionCheckStorage,
/// Storage of `sessionspec.view.SessionListing`.
pub session_listing_storage: MemorySessionListingStorage,
}

fn memory_key_44(value: &sessionspec_types::session::AgentId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_53(value: &sessionspec_types::session::ExposureId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_59(value: &sessionspec_types::session::SessionId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_62(value: &sessionspec_types::session::ToolCallId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_68(value: &sessionspec_types::view::CheckId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_71(value: &sessionspec_types::view::EventId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

impl sessionspec_types::view::obligations::SessionListQuery for MemoryPorts {
fn session_list(&self, since: Option<sessionspec_types::primitives::Timestamp>, search: Option<String>, limit: Option<i64>) -> Result<Vec<sessionspec_types::view::SessionList>, sessionspec_types::obligation::UnmetObligation> { sessionspec_types::view::obligations::SessionListQuery::session_list(&sessionspec_types::view::obligations::Unimplemented, since, search, limit) }
}
