use bevy::utils::hashbrown::hash_map::DefaultHashBuilder;
use indexmap::IndexMap as LibIndexMap;

pub type IndexMap<K, V> = LibIndexMap<K, V, DefaultHashBuilder>;
