use std::marker::PhantomData;

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::ecs::error::{BevyError, Severity};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;
use ron::de::from_bytes;
use serde::de::DeserializeOwned;

#[derive(TypePath)]
pub struct RonAssetLoader<A> {
    extensions: &'static [&'static str],
    marker: PhantomData<fn() -> A>,
}

impl<A: Asset + DeserializeOwned> RonAssetLoader<A> {
    pub fn new(extensions: &'static [&'static str]) -> Self {
        Self {
            extensions,
            marker: PhantomData,
        }
    }
}

impl<A: Asset + DeserializeOwned> AssetLoader for RonAssetLoader<A> {
    type Asset = A;
    type Settings = ();
    type Error = BevyError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<A, BevyError>> {
        async move {
            let mut bytes = Vec::new();

            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|e| BevyError::new(Severity::Error, e))?;

            from_bytes(&bytes).map_err(|e| BevyError::new(Severity::Error, e))
        }
    }

    fn extensions(&self) -> &[&str] {
        self.extensions
    }
}
