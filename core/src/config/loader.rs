use std::marker::PhantomData;

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::ecs::error::{BevyError, Severity};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;
use ron::de::from_bytes;
use serde::de::DeserializeOwned;

use crate::validate::Validate;

#[derive(TypePath)]
pub struct RonAssetLoader<A> {
    extensions: &'static [&'static str],
    marker: PhantomData<fn() -> A>,
}

impl<A: Asset + DeserializeOwned + Validate<Context = ()>> RonAssetLoader<A> {
    pub fn new(extensions: &'static [&'static str]) -> Self {
        Self {
            extensions,
            marker: PhantomData,
        }
    }
}

impl<A: Asset + DeserializeOwned + Validate<Context = ()>> AssetLoader for RonAssetLoader<A>
where
    A::Error: std::fmt::Display,
{
    type Asset = A;
    type Settings = ();
    type Error = BevyError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<A, BevyError>> {
        let path_display = load_context.path().to_string();
        async move {
            let mut bytes = Vec::new();

            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|e| BevyError::new(Severity::Error, e))?;

            let asset: A = from_bytes(&bytes).map_err(|e| BevyError::new(Severity::Error, e))?;

            asset.validate(&()).map_err(|err| {
                BevyError::new(
                    Severity::Error,
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("{path_display}: {err}"),
                    ),
                )
            })?;

            Ok(asset)
        }
    }

    fn extensions(&self) -> &[&str] {
        self.extensions
    }
}
