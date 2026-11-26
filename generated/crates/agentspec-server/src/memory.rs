// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `agentspec.session.Agent`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryAgentStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::session::AgentSnapshot>>>);
impl agentspec_types::behaviour::AgentStorage for MemoryAgentStorage {
fn get(&self, identity: &agentspec_types::session::AgentId) -> Option<agentspec_types::session::AgentSnapshot> { let key = memory_key_44(identity); self.0.borrow().iter().find(|row| memory_key_44(&row.data.agent_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::session::AgentSnapshot) { self.delete(&snapshot.data.agent_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_44(&row.data.agent_id).cmp(&memory_key_44(&other.data.agent_id))); }
fn delete(&mut self, identity: &agentspec_types::session::AgentId) { let key = memory_key_44(identity); self.0.borrow_mut().retain(|row| memory_key_44(&row.data.agent_id) != key); }
fn list(&self) -> Vec<agentspec_types::session::AgentSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::AgentStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::session::AgentId) -> Option<agentspec_types::session::AgentSnapshot> { agentspec_types::behaviour::AgentStorage::get(&self.agent_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::session::AgentSnapshot) { agentspec_types::behaviour::AgentStorage::put(&mut self.agent_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::session::AgentId) { agentspec_types::behaviour::AgentStorage::delete(&mut self.agent_storage, identity); }
fn list(&self) -> Vec<agentspec_types::session::AgentSnapshot> { agentspec_types::behaviour::AgentStorage::list(&self.agent_storage) }
}

/// Ephemeral storage of `agentspec.session.Exposure`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryExposureStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::session::ExposureSnapshot>>>);
impl agentspec_types::behaviour::ExposureStorage for MemoryExposureStorage {
fn get(&self, identity: &agentspec_types::session::ExposureId) -> Option<agentspec_types::session::ExposureSnapshot> { let key = memory_key_53(identity); self.0.borrow().iter().find(|row| memory_key_53(&row.data.exposure_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::session::ExposureSnapshot) { self.delete(&snapshot.data.exposure_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_53(&row.data.exposure_id).cmp(&memory_key_53(&other.data.exposure_id))); }
fn delete(&mut self, identity: &agentspec_types::session::ExposureId) { let key = memory_key_53(identity); self.0.borrow_mut().retain(|row| memory_key_53(&row.data.exposure_id) != key); }
fn list(&self) -> Vec<agentspec_types::session::ExposureSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::ExposureStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::session::ExposureId) -> Option<agentspec_types::session::ExposureSnapshot> { agentspec_types::behaviour::ExposureStorage::get(&self.exposure_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::session::ExposureSnapshot) { agentspec_types::behaviour::ExposureStorage::put(&mut self.exposure_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::session::ExposureId) { agentspec_types::behaviour::ExposureStorage::delete(&mut self.exposure_storage, identity); }
fn list(&self) -> Vec<agentspec_types::session::ExposureSnapshot> { agentspec_types::behaviour::ExposureStorage::list(&self.exposure_storage) }
}

/// Ephemeral storage of `agentspec.session.Session`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySessionStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::session::SessionSnapshot>>>);
impl agentspec_types::behaviour::SessionStorage for MemorySessionStorage {
fn get(&self, identity: &agentspec_types::session::SessionId) -> Option<agentspec_types::session::SessionSnapshot> { let key = memory_key_59(identity); self.0.borrow().iter().find(|row| memory_key_59(&row.data.session_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::session::SessionSnapshot) { self.delete(&snapshot.data.session_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_59(&row.data.session_id).cmp(&memory_key_59(&other.data.session_id))); }
fn delete(&mut self, identity: &agentspec_types::session::SessionId) { let key = memory_key_59(identity); self.0.borrow_mut().retain(|row| memory_key_59(&row.data.session_id) != key); }
fn list(&self) -> Vec<agentspec_types::session::SessionSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::SessionStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::session::SessionId) -> Option<agentspec_types::session::SessionSnapshot> { agentspec_types::behaviour::SessionStorage::get(&self.session_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::session::SessionSnapshot) { agentspec_types::behaviour::SessionStorage::put(&mut self.session_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::session::SessionId) { agentspec_types::behaviour::SessionStorage::delete(&mut self.session_storage, identity); }
fn list(&self) -> Vec<agentspec_types::session::SessionSnapshot> { agentspec_types::behaviour::SessionStorage::list(&self.session_storage) }
}

/// Ephemeral storage of `agentspec.session.ToolCall`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryToolCallStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::session::ToolCallSnapshot>>>);
impl agentspec_types::behaviour::ToolCallStorage for MemoryToolCallStorage {
fn get(&self, identity: &agentspec_types::session::ToolCallId) -> Option<agentspec_types::session::ToolCallSnapshot> { let key = memory_key_62(identity); self.0.borrow().iter().find(|row| memory_key_62(&row.data.tool_call_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::session::ToolCallSnapshot) { self.delete(&snapshot.data.tool_call_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_62(&row.data.tool_call_id).cmp(&memory_key_62(&other.data.tool_call_id))); }
fn delete(&mut self, identity: &agentspec_types::session::ToolCallId) { let key = memory_key_62(identity); self.0.borrow_mut().retain(|row| memory_key_62(&row.data.tool_call_id) != key); }
fn list(&self) -> Vec<agentspec_types::session::ToolCallSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::ToolCallStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::session::ToolCallId) -> Option<agentspec_types::session::ToolCallSnapshot> { agentspec_types::behaviour::ToolCallStorage::get(&self.tool_call_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::session::ToolCallSnapshot) { agentspec_types::behaviour::ToolCallStorage::put(&mut self.tool_call_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::session::ToolCallId) { agentspec_types::behaviour::ToolCallStorage::delete(&mut self.tool_call_storage, identity); }
fn list(&self) -> Vec<agentspec_types::session::ToolCallSnapshot> { agentspec_types::behaviour::ToolCallStorage::list(&self.tool_call_storage) }
}

/// Ephemeral storage of `agentspec.view.AgentListing`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryAgentListingStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::view::AgentListingSnapshot>>>);
impl agentspec_types::behaviour::AgentListingStorage for MemoryAgentListingStorage {
fn get(&self, identity: &agentspec_types::session::AgentId) -> Option<agentspec_types::view::AgentListingSnapshot> { let key = memory_key_44(identity); self.0.borrow().iter().find(|row| memory_key_44(&row.data.agent_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::view::AgentListingSnapshot) { self.delete(&snapshot.data.agent_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_44(&row.data.agent_id).cmp(&memory_key_44(&other.data.agent_id))); }
fn delete(&mut self, identity: &agentspec_types::session::AgentId) { let key = memory_key_44(identity); self.0.borrow_mut().retain(|row| memory_key_44(&row.data.agent_id) != key); }
fn list(&self) -> Vec<agentspec_types::view::AgentListingSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::AgentListingStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::session::AgentId) -> Option<agentspec_types::view::AgentListingSnapshot> { agentspec_types::behaviour::AgentListingStorage::get(&self.agent_listing_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::view::AgentListingSnapshot) { agentspec_types::behaviour::AgentListingStorage::put(&mut self.agent_listing_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::session::AgentId) { agentspec_types::behaviour::AgentListingStorage::delete(&mut self.agent_listing_storage, identity); }
fn list(&self) -> Vec<agentspec_types::view::AgentListingSnapshot> { agentspec_types::behaviour::AgentListingStorage::list(&self.agent_listing_storage) }
}

/// Ephemeral storage of `agentspec.view.EventRow`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryEventRowStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::view::EventRowSnapshot>>>);
impl agentspec_types::behaviour::EventRowStorage for MemoryEventRowStorage {
fn get(&self, identity: &agentspec_types::view::EventId) -> Option<agentspec_types::view::EventRowSnapshot> { let key = memory_key_74(identity); self.0.borrow().iter().find(|row| memory_key_74(&row.data.event_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::view::EventRowSnapshot) { self.delete(&snapshot.data.event_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_74(&row.data.event_id).cmp(&memory_key_74(&other.data.event_id))); }
fn delete(&mut self, identity: &agentspec_types::view::EventId) { let key = memory_key_74(identity); self.0.borrow_mut().retain(|row| memory_key_74(&row.data.event_id) != key); }
fn list(&self) -> Vec<agentspec_types::view::EventRowSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::EventRowStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::view::EventId) -> Option<agentspec_types::view::EventRowSnapshot> { agentspec_types::behaviour::EventRowStorage::get(&self.event_row_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::view::EventRowSnapshot) { agentspec_types::behaviour::EventRowStorage::put(&mut self.event_row_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::view::EventId) { agentspec_types::behaviour::EventRowStorage::delete(&mut self.event_row_storage, identity); }
fn list(&self) -> Vec<agentspec_types::view::EventRowSnapshot> { agentspec_types::behaviour::EventRowStorage::list(&self.event_row_storage) }
}

/// Ephemeral storage of `agentspec.view.FigureBucket`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryFigureBucketStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::view::FigureBucketSnapshot>>>);
impl agentspec_types::behaviour::FigureBucketStorage for MemoryFigureBucketStorage {
fn get(&self, identity: &agentspec_types::view::BucketId) -> Option<agentspec_types::view::FigureBucketSnapshot> { let key = memory_key_68(identity); self.0.borrow().iter().find(|row| memory_key_68(&row.data.bucket_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::view::FigureBucketSnapshot) { self.delete(&snapshot.data.bucket_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_68(&row.data.bucket_id).cmp(&memory_key_68(&other.data.bucket_id))); }
fn delete(&mut self, identity: &agentspec_types::view::BucketId) { let key = memory_key_68(identity); self.0.borrow_mut().retain(|row| memory_key_68(&row.data.bucket_id) != key); }
fn list(&self) -> Vec<agentspec_types::view::FigureBucketSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::FigureBucketStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::view::BucketId) -> Option<agentspec_types::view::FigureBucketSnapshot> { agentspec_types::behaviour::FigureBucketStorage::get(&self.figure_bucket_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::view::FigureBucketSnapshot) { agentspec_types::behaviour::FigureBucketStorage::put(&mut self.figure_bucket_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::view::BucketId) { agentspec_types::behaviour::FigureBucketStorage::delete(&mut self.figure_bucket_storage, identity); }
fn list(&self) -> Vec<agentspec_types::view::FigureBucketSnapshot> { agentspec_types::behaviour::FigureBucketStorage::list(&self.figure_bucket_storage) }
}

/// Ephemeral storage of `agentspec.view.SessionCheck`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySessionCheckStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::view::SessionCheckSnapshot>>>);
impl agentspec_types::behaviour::SessionCheckStorage for MemorySessionCheckStorage {
fn get(&self, identity: &agentspec_types::view::CheckId) -> Option<agentspec_types::view::SessionCheckSnapshot> { let key = memory_key_69(identity); self.0.borrow().iter().find(|row| memory_key_69(&row.data.check_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::view::SessionCheckSnapshot) { self.delete(&snapshot.data.check_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_69(&row.data.check_id).cmp(&memory_key_69(&other.data.check_id))); }
fn delete(&mut self, identity: &agentspec_types::view::CheckId) { let key = memory_key_69(identity); self.0.borrow_mut().retain(|row| memory_key_69(&row.data.check_id) != key); }
fn list(&self) -> Vec<agentspec_types::view::SessionCheckSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::SessionCheckStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::view::CheckId) -> Option<agentspec_types::view::SessionCheckSnapshot> { agentspec_types::behaviour::SessionCheckStorage::get(&self.session_check_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::view::SessionCheckSnapshot) { agentspec_types::behaviour::SessionCheckStorage::put(&mut self.session_check_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::view::CheckId) { agentspec_types::behaviour::SessionCheckStorage::delete(&mut self.session_check_storage, identity); }
fn list(&self) -> Vec<agentspec_types::view::SessionCheckSnapshot> { agentspec_types::behaviour::SessionCheckStorage::list(&self.session_check_storage) }
}

/// Ephemeral storage of `agentspec.view.SessionListing`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySessionListingStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::view::SessionListingSnapshot>>>);
impl agentspec_types::behaviour::SessionListingStorage for MemorySessionListingStorage {
fn get(&self, identity: &agentspec_types::session::SessionId) -> Option<agentspec_types::view::SessionListingSnapshot> { let key = memory_key_59(identity); self.0.borrow().iter().find(|row| memory_key_59(&row.data.session_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::view::SessionListingSnapshot) { self.delete(&snapshot.data.session_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_59(&row.data.session_id).cmp(&memory_key_59(&other.data.session_id))); }
fn delete(&mut self, identity: &agentspec_types::session::SessionId) { let key = memory_key_59(identity); self.0.borrow_mut().retain(|row| memory_key_59(&row.data.session_id) != key); }
fn list(&self) -> Vec<agentspec_types::view::SessionListingSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::SessionListingStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::session::SessionId) -> Option<agentspec_types::view::SessionListingSnapshot> { agentspec_types::behaviour::SessionListingStorage::get(&self.session_listing_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::view::SessionListingSnapshot) { agentspec_types::behaviour::SessionListingStorage::put(&mut self.session_listing_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::session::SessionId) { agentspec_types::behaviour::SessionListingStorage::delete(&mut self.session_listing_storage, identity); }
fn list(&self) -> Vec<agentspec_types::view::SessionListingSnapshot> { agentspec_types::behaviour::SessionListingStorage::list(&self.session_listing_storage) }
}

/// Ephemeral storage of `agentspec.view.TimelineSlot`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryTimelineSlotStorage(std::rc::Rc<std::cell::RefCell<Vec<agentspec_types::view::TimelineSlotSnapshot>>>);
impl agentspec_types::behaviour::TimelineSlotStorage for MemoryTimelineSlotStorage {
fn get(&self, identity: &agentspec_types::view::SlotId) -> Option<agentspec_types::view::TimelineSlotSnapshot> { let key = memory_key_80(identity); self.0.borrow().iter().find(|row| memory_key_80(&row.data.slot_id) == key).cloned() }
fn put(&mut self, snapshot: agentspec_types::view::TimelineSlotSnapshot) { self.delete(&snapshot.data.slot_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_80(&row.data.slot_id).cmp(&memory_key_80(&other.data.slot_id))); }
fn delete(&mut self, identity: &agentspec_types::view::SlotId) { let key = memory_key_80(identity); self.0.borrow_mut().retain(|row| memory_key_80(&row.data.slot_id) != key); }
fn list(&self) -> Vec<agentspec_types::view::TimelineSlotSnapshot> { self.0.borrow().clone() }
}
impl agentspec_types::behaviour::TimelineSlotStorage for MemoryPorts {
fn get(&self, identity: &agentspec_types::view::SlotId) -> Option<agentspec_types::view::TimelineSlotSnapshot> { agentspec_types::behaviour::TimelineSlotStorage::get(&self.timeline_slot_storage, identity) }
fn put(&mut self, snapshot: agentspec_types::view::TimelineSlotSnapshot) { agentspec_types::behaviour::TimelineSlotStorage::put(&mut self.timeline_slot_storage, snapshot); }
fn delete(&mut self, identity: &agentspec_types::view::SlotId) { agentspec_types::behaviour::TimelineSlotStorage::delete(&mut self.timeline_slot_storage, identity); }
fn list(&self) -> Vec<agentspec_types::view::TimelineSlotSnapshot> { agentspec_types::behaviour::TimelineSlotStorage::list(&self.timeline_slot_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `agentspec.session.Agent`.
pub agent_storage: MemoryAgentStorage,
/// Storage of `agentspec.session.Exposure`.
pub exposure_storage: MemoryExposureStorage,
/// Storage of `agentspec.session.Session`.
pub session_storage: MemorySessionStorage,
/// Storage of `agentspec.session.ToolCall`.
pub tool_call_storage: MemoryToolCallStorage,
/// Storage of `agentspec.view.AgentListing`.
pub agent_listing_storage: MemoryAgentListingStorage,
/// Storage of `agentspec.view.EventRow`.
pub event_row_storage: MemoryEventRowStorage,
/// Storage of `agentspec.view.FigureBucket`.
pub figure_bucket_storage: MemoryFigureBucketStorage,
/// Storage of `agentspec.view.SessionCheck`.
pub session_check_storage: MemorySessionCheckStorage,
/// Storage of `agentspec.view.SessionListing`.
pub session_listing_storage: MemorySessionListingStorage,
/// Storage of `agentspec.view.TimelineSlot`.
pub timeline_slot_storage: MemoryTimelineSlotStorage,
}

fn memory_key_44(value: &agentspec_types::session::AgentId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_53(value: &agentspec_types::session::ExposureId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_59(value: &agentspec_types::session::SessionId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_62(value: &agentspec_types::session::ToolCallId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_68(value: &agentspec_types::view::BucketId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_69(value: &agentspec_types::view::CheckId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_74(value: &agentspec_types::view::EventId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_80(value: &agentspec_types::view::SlotId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

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

impl agentspec_types::view::obligations::SessionEventCountsQuery for MemoryPorts {
fn session_event_counts(&self, session_id: agentspec_types::session::SessionId, agent: Option<agentspec_types::session::AgentId>) -> Result<Vec<agentspec_types::view::SessionEventCounts>, agentspec_types::obligation::UnmetObligation> { agentspec_types::view::obligations::SessionEventCountsQuery::session_event_counts(&agentspec_types::view::obligations::Unimplemented, session_id, agent) }
}
impl agentspec_types::view::obligations::SessionEventsQuery for MemoryPorts {
fn session_events(&self, session_id: agentspec_types::session::SessionId, agent: Option<agentspec_types::session::AgentId>, kind: Option<agentspec_types::view::EventKind>, notable: Option<bool>, before: Option<i64>) -> Result<Vec<agentspec_types::view::SessionEvents>, agentspec_types::obligation::UnmetObligation> { agentspec_types::view::obligations::SessionEventsQuery::session_events(&agentspec_types::view::obligations::Unimplemented, session_id, agent, kind, notable, before) }
}
impl agentspec_types::view::obligations::SessionListQuery for MemoryPorts {
fn session_list(&self, since: Option<agentspec_types::primitives::Timestamp>, search: Option<String>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<agentspec_types::view::SessionList>, agentspec_types::obligation::UnmetObligation> { agentspec_types::view::obligations::SessionListQuery::session_list(&agentspec_types::view::obligations::Unimplemented, since, search, limit, offset) }
}
