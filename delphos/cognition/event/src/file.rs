use std::{fs::{File,OpenOptions},io::{BufRead,BufReader,Write},path::{Path,PathBuf}};
use bkg_core::{BkgError,BkgResult,EventId};
use crate::{event::Event,ledger::EventLedger,memory::InMemoryLedger};

pub struct FileLedger { path: PathBuf, file: File, cache: InMemoryLedger }

impl FileLedger {
    pub fn open(path: impl AsRef<Path>) -> BkgResult<Self> {
        let path = path.as_ref().to_path_buf();
        let cache = if path.exists() {
            let mut events = Vec::new();
            for line in BufReader::new(File::open(&path)?).lines() {
                let line = line?; let t = line.trim();
                if t.is_empty() { continue; }
                if let Ok(e) = serde_json::from_str::<Event>(t) { events.push(e); }
            }
            InMemoryLedger::from_events(events)?
        } else { InMemoryLedger::new() };
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { path, file, cache })
    }
    pub fn path(&self) -> &Path { &self.path }
}

impl EventLedger for FileLedger {
    fn append(&mut self, event: Event) -> BkgResult<()> {
        if !event.verify_hash() { return Err(BkgError::CapsuleIntegrityError(format!("event {} bad hash",event.id))); }
        let exp = self.cache.tail_hash();
        if event.parent_hash != exp { return Err(BkgError::HashChainBroken{event_id:event.id.to_string(),expected:exp.to_hex(),actual:event.parent_hash.to_hex()}); }
        let mut line = serde_json::to_string(&event)?; line.push('\n');
        self.file.write_all(line.as_bytes())?; self.file.flush()?; self.file.sync_data()?;
        self.cache.append(event)?;
        Ok(())
    }
    fn get(&self, id: &EventId) -> BkgResult<Option<&Event>> { self.cache.get(id) }
    fn head(&self) -> Option<&Event> { self.cache.head() }
    fn tail(&self) -> Option<&Event> { self.cache.tail() }
    fn len(&self) -> usize { self.cache.len() }
    fn events_in_range(&self, f: u64, t: u64) -> Vec<&Event> { self.cache.events_in_range(f, t) }
    fn all_events(&self) -> Vec<&Event> { self.cache.all_events() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::{ExecutionSeed,Hash256,RealmId};
    use crate::event::EventBuilder;
    fn tmp() -> PathBuf { let mut p=std::env::temp_dir(); p.push(format!("bkg_{}.ndjson",uuid::Uuid::new_v4())); p }
    fn genesis() -> Event { EventBuilder::new(RealmId::Styx).seed(ExecutionSeed::random()).payload(serde_json::json!({})).parent(Hash256::ZERO).build() }
    fn next(p:&Event)->Event { EventBuilder::new(RealmId::Telum).seed(ExecutionSeed::random()).payload(serde_json::json!({})).parent(p.hash).timestamp(p.timestamp.next()).build() }
    #[test] fn reopen() {
        let path=tmp(); let e0=genesis(); let e1=next(&e0); let id0=e0.id; let id1=e1.id;
        { let mut l=FileLedger::open(&path).unwrap(); l.append(e0).unwrap(); l.append(e1).unwrap(); }
        let l=FileLedger::open(&path).unwrap();
        assert_eq!(l.len(),2); assert!(l.get(&id0).unwrap().is_some()); assert!(l.get(&id1).unwrap().is_some());
        let _=std::fs::remove_file(&path);
    }
}
