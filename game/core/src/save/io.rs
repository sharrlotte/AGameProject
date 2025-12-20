use bevy::{
    asset::ron,
    ecs::{entity::EntityHashSet, query::QueryFilter},
    prelude::*,
};
use directories::ProjectDirs;
use std::{
    any::TypeId,
    fs,
    io::Write,
    marker::PhantomData,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::{
    content::block::base::BlockType,
    save::base::{SchematicBlockData, SchematicData},
};

pub fn save_dir() -> PathBuf {
    let project_dirs = ProjectDirs::from(
        "com",           // qualifier
        "somerandomdev", // organization
        "agameproject",  // application
    )
    .expect("Failed to get project directory");

    project_dirs.data_dir().to_path_buf()
}

pub fn load_saved_schematics() -> Vec<SchematicData> {
    let schematics_dir = save_dir().join("schematics");

    let mut schematics = Vec::new();

    if let Ok(entries) = fs::read_dir(schematics_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let path = save_dir().join("schematics").join(name);
                    if let Ok(json) = fs::read_to_string(path) {
                        if let Ok(mut ship) = serde_json::from_str::<SchematicData>(&json) {
                            ship.filename = name.to_string().replace(".json", "");
                            schematics.push(ship);
                        }
                    }
                }
            }
        }
    }

    while schematics.len() < 3 {
        let timetamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        schematics.push(SchematicData {
            id: Uuid::now_v7(),
            filename: timetamp,
            name: format!("ship_{}", schematics.len()),
            blocks: Vec::new(),
        });
    }

    for ship in &mut schematics {
        if ship.blocks.len() <= 2 {
            ship.blocks.clear();
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: BlockType::Core,
                position: IVec2::new(0, 0),
                rotation: 0,
            });
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: BlockType::LazerTurret,
                position: IVec2::new(1, 3),
                rotation: 0,
            });
        }
    }

    return schematics;
}

pub fn save_schematic(data: &SchematicData) -> Result<(), std::io::Error> {
    let schematics_dir = save_dir().join("schematics");
    let filename = format!("{}.json", data.filename);
    let path = schematics_dir.join(&filename);

    // Create schematics directory if it doesn't exist
    if !schematics_dir.exists() {
        if let Err(e) = fs::create_dir_all(&schematics_dir) {
            return Err(e);
        }
    }

    match serde_json::to_string_pretty(&data) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                return Err(e);
            } else {
                info!("Ship saved at: {}", path.display());
                return Ok(());
            }
        }
        Err(e) => return Err(e.into()),
    }
}

pub trait MapComponent<T: Component>: 'static + Clone + Send + Sync {
    type Output: Component;
    fn map_component(&self, component: &T) -> Self::Output;
}

impl<F: Fn(&T) -> U, T: Component, U: Component> MapComponent<T> for F
where
    F: 'static + Clone + Send + Sync,
{
    type Output = U;

    fn map_component(&self, component: &T) -> Self::Output {
        self(component)
    }
}

#[derive(Default)]
pub struct SceneMapper(Vec<ComponentMapperDyn>);

impl SceneMapper {
    pub fn map<T: Component>(mut self, m: impl MapComponent<T>) -> Self {
        self.0.push(Box::new(ComponentMapperImpl::new(m)));
        self
    }

    pub(crate) fn apply(&mut self, mut entity: EntityWorldMut) {
        for mapper in &mut self.0 {
            mapper.apply(&mut entity);
        }
    }

    pub(crate) fn replace(&mut self, mut entity: EntityWorldMut) {
        for mapper in &mut self.0 {
            mapper.replace(&mut entity);
        }
    }

    pub(crate) fn undo(&mut self, mut entity: EntityWorldMut) {
        for mapper in &mut self.0 {
            mapper.undo(&mut entity);
        }
    }
}

trait ComponentMapper: 'static {
    fn apply(&mut self, entity: &mut EntityWorldMut);

    fn replace(&mut self, entity: &mut EntityWorldMut);

    fn undo(&mut self, entity: &mut EntityWorldMut);
}

struct ComponentMapperImpl<T: Component, M: MapComponent<T>>(M, PhantomData<T>);

impl<T: Component, M: MapComponent<T>> ComponentMapperImpl<T, M> {
    fn new(m: M) -> Self {
        Self(m, PhantomData)
    }
}

impl<T: Component, M: MapComponent<T>> ComponentMapper for ComponentMapperImpl<T, M> {
    fn apply(&mut self, entity: &mut EntityWorldMut) {
        if let Some(component) = entity.get::<T>() {
            entity.insert(self.0.map_component(component));
        }
    }

    fn replace(&mut self, entity: &mut EntityWorldMut) {
        if let Some(component) = entity.take::<T>() {
            entity.insert(self.0.map_component(&component));
        }
    }

    fn undo(&mut self, entity: &mut EntityWorldMut) {
        entity.remove::<M::Output>();
    }
}

type ComponentMapperDyn = Box<dyn ComponentMapper>;

pub type SaveResult = Result<Saved, SaveError>;

#[derive(Component, Default, Debug, Clone)]
pub struct Save;

pub trait TriggerSave {
    #[doc(alias = "trigger_single")]
    fn trigger_save(self, event: impl SaveEvent);
}

pub trait SaveEvent {
    type SaveFilter: QueryFilter;

    fn filter_entity(&self, _entity: EntityRef) -> bool {
        true
    }
    fn before_save(&mut self, _world: &mut World) {}
    fn before_serialize(&mut self, _world: &mut World, _entities: &[Entity]) {}
    fn component_filter(&mut self) -> SceneFilter {
        SceneFilter::allow_all()
    }
    fn resource_filter(&mut self) -> SceneFilter {
        SceneFilter::deny_all()
    }
    fn after_save(&mut self, _world: &mut World, _result: &SaveResult) {}
    fn output(&mut self) -> SaveOutput;
}

pub struct SaveWorld<F: QueryFilter = DefaultSaveFilter> {
    pub entities: EntityFilter,
    pub resources: SceneFilter,
    pub components: SceneFilter,
    pub mapper: SceneMapper,
    pub output: SaveOutput,
    pub filter: PhantomData<F>,
}

impl<F: QueryFilter> SaveWorld<F> {
    pub fn new(output: SaveOutput) -> Self {
        Self {
            entities: EntityFilter::allow_all(),
            resources: SceneFilter::deny_all(),
            components: SceneFilter::allow_all(),
            mapper: SceneMapper::default(),
            output,
            filter: PhantomData,
        }
    }

    pub fn into_file(path: impl Into<PathBuf>) -> Self {
        Self {
            entities: EntityFilter::allow_all(),
            resources: SceneFilter::deny_all(),
            components: SceneFilter::allow_all(),
            mapper: SceneMapper::default(),
            output: SaveOutput::file(path),
            filter: PhantomData,
        }
    }

    pub fn into_stream(stream: impl SaveStream) -> Self {
        Self {
            entities: EntityFilter::allow_all(),
            resources: SceneFilter::deny_all(),
            components: SceneFilter::allow_all(),
            mapper: SceneMapper::default(),
            output: SaveOutput::stream(stream),
            filter: PhantomData,
        }
    }

    pub fn include_resource<R: Resource>(mut self) -> Self {
        self.resources = self.resources.allow::<R>();
        self
    }

    pub fn include_resource_by_id(mut self, type_id: TypeId) -> Self {
        self.resources = self.resources.allow_by_id(type_id);
        self
    }

    pub fn exclude_component<T: Component>(mut self) -> Self {
        self.components = self.components.deny::<T>();
        self
    }

    pub fn exclude_component_by_id(mut self, type_id: TypeId) -> Self {
        self.components = self.components.deny_by_id(type_id);
        self
    }

    pub fn map_component<T: Component>(mut self, m: impl MapComponent<T>) -> Self {
        self.mapper = self.mapper.map(m);
        self
    }
}

impl SaveWorld {
    pub fn default_into_file(path: impl Into<PathBuf>) -> Self {
        Self::into_file(path)
    }

    pub fn default_into_stream(stream: impl SaveStream) -> Self {
        Self::into_stream(stream)
    }
}

impl SaveWorld<()> {
    pub fn all_into_file(path: impl Into<PathBuf>) -> Self {
        Self::into_file(path)
    }

    pub fn all_into_stream(stream: impl SaveStream) -> Self {
        Self::into_stream(stream)
    }
}

impl<F: QueryFilter> SaveEvent for SaveWorld<F>
where
    F: 'static,
{
    type SaveFilter = F;

    fn filter_entity(&self, entity: EntityRef) -> bool {
        match &self.entities {
            EntityFilter::Allow(allow) => allow.contains(&entity.id()),
            EntityFilter::Block(block) => !block.contains(&entity.id()),
        }
    }

    fn before_serialize(&mut self, world: &mut World, entities: &[Entity]) {
        for entity in entities {
            self.mapper.apply(world.entity_mut(*entity));
        }
    }

    fn after_save(&mut self, world: &mut World, result: &SaveResult) {
        let Ok(saved) = result else {
            return;
        };

        for entity in saved.entities() {
            self.mapper.undo(world.entity_mut(entity));
        }
    }

    fn component_filter(&mut self) -> SceneFilter {
        std::mem::replace(&mut self.components, SceneFilter::Unset)
    }

    fn resource_filter(&mut self) -> SceneFilter {
        std::mem::replace(&mut self.resources, SceneFilter::Unset)
    }

    fn output(&mut self) -> SaveOutput {
        self.output.consume().unwrap()
    }
}

pub type DefaultSaveFilter = With<Save>;

pub enum SaveOutput {
    File(PathBuf),
    Stream(Box<dyn SaveStream>),
    Drop,
    Invalid,
}

impl SaveOutput {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    pub fn stream<S: SaveStream + 'static>(stream: S) -> Self {
        Self::Stream(Box::new(stream))
    }

    pub fn consume(&mut self) -> Option<SaveOutput> {
        let output = std::mem::replace(self, SaveOutput::Invalid);
        if let SaveOutput::Invalid = output {
            return None;
        }
        Some(output)
    }
}

#[derive(Clone, Debug)]
pub enum EntityFilter {
    Allow(EntityHashSet),
    Block(EntityHashSet),
}

impl EntityFilter {
    pub fn allow_all() -> Self {
        Self::Block(EntityHashSet::new())
    }

    pub fn allow(entities: impl IntoIterator<Item = Entity>) -> Self {
        Self::Allow(entities.into_iter().collect())
    }

    pub fn block(entities: impl IntoIterator<Item = Entity>) -> Self {
        Self::Block(entities.into_iter().collect())
    }
}

impl Default for EntityFilter {
    fn default() -> Self {
        Self::allow_all()
    }
}

pub trait SaveStream: Write
where
    Self: 'static,
{
}

impl<S: Write> SaveStream for S where S: 'static {}

#[derive(Event)]
pub struct Saved {
    pub scene: DynamicScene,
}

impl Saved {
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.scene.entities.iter().map(|de| de.entity)
    }
}

#[doc(hidden)]
#[deprecated(since = "0.5.2", note = "use `Saved` instead")]
pub type OnSave = Saved;

pub enum SaveError {
    Ron(ron::Error),
    Io(std::io::Error),
}

impl From<ron::Error> for SaveError {
    fn from(e: ron::Error) -> Self {
        Self::Ron(e)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

fn save_world<E: SaveEvent>(mut event: E, world: &mut World) -> SaveResult {
    // Notify
    event.before_save(world);

    // Filter
    let entities: Vec<_> = world
        .query_filtered::<Entity, E::SaveFilter>()
        .iter(world)
        .filter(|entity| event.filter_entity(world.entity(*entity)))
        .collect();

    // Serialize
    event.before_serialize(world, &entities);
    let scene = DynamicSceneBuilder::from_world(world)
        .with_component_filter(event.component_filter())
        .with_resource_filter(event.resource_filter())
        .extract_resources()
        .extract_entities(entities.iter().copied())
        .build();

    // Write
    let saved = match event.output() {
        SaveOutput::File(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let type_registry = world.resource::<AppTypeRegistry>().read();
            let data = scene.serialize(&type_registry)?;
            std::fs::write(&path, data.as_bytes())?;
            debug!("saved into file: {path:?}");
            Saved { scene }
        }
        SaveOutput::Stream(mut stream) => {
            let type_registry = world.resource::<AppTypeRegistry>().read();
            let data = scene.serialize(&type_registry)?;
            stream.write_all(data.as_bytes())?;
            debug!("saved into stream");
            Saved { scene }
        }
        SaveOutput::Drop => {
            debug!("saved data dropped");
            Saved { scene }
        }
        SaveOutput::Invalid => {
            panic!("SaveOutput is invalid");
        }
    };

    let result = Ok(saved);
    event.after_save(world, &result);
    result
}

struct SaveCommand<E>(E);

impl<E: SaveEvent + std::marker::Send + 'static> Command<Result<(), SaveError>> for SaveCommand<E> {
    fn apply(self, world: &mut World) -> Result<(), SaveError> {
        let saved = save_world(self.0, world)?;
        world.trigger(saved);
        Ok(())
    }
}
