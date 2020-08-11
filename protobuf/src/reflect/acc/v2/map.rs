use std::collections::HashMap;
use std::hash::Hash;

use crate::message::Message;

use crate::message_dyn::MessageDyn;
use crate::reflect::acc::v2::AccessorV2;
use crate::reflect::acc::FieldAccessor;
use crate::reflect::map::ReflectMapMut;
use crate::reflect::map::ReflectMapRef;
use crate::reflect::ProtobufValueSized;
use crate::reflect::RuntimeTypeBox;

pub(crate) trait MapFieldAccessor: Send + Sync + 'static {
    fn get_reflect<'a>(&self, m: &'a dyn MessageDyn) -> ReflectMapRef<'a>;
    fn mut_reflect<'a>(&self, m: &'a mut dyn MessageDyn) -> ReflectMapMut<'a>;
    fn element_type(&self) -> (RuntimeTypeBox, RuntimeTypeBox);
}

pub(crate) struct MapFieldAccessorHolder {
    pub accessor: Box<dyn MapFieldAccessor>,
}

struct MapFieldAccessorImpl<M, K, V>
where
    M: Message,
    K: ProtobufValueSized,
    V: ProtobufValueSized,
{
    get_field: fn(&M) -> &HashMap<K, V>,
    mut_field: fn(&mut M) -> &mut HashMap<K, V>,
}

impl<M, K, V> MapFieldAccessor for MapFieldAccessorImpl<M, K, V>
where
    M: Message,
    K: ProtobufValueSized + Eq + Hash,
    V: ProtobufValueSized,
{
    fn get_reflect<'a>(&self, m: &'a dyn MessageDyn) -> ReflectMapRef<'a> {
        let m = m.downcast_ref().unwrap();
        let map = (self.get_field)(m);
        ReflectMapRef::new(map)
    }

    fn mut_reflect<'a>(&self, m: &'a mut dyn MessageDyn) -> ReflectMapMut<'a> {
        let m = m.downcast_mut().unwrap();
        let map = (self.mut_field)(m);
        ReflectMapMut::new(map)
    }

    fn element_type(&self) -> (RuntimeTypeBox, RuntimeTypeBox) {
        (K::runtime_type_box(), V::runtime_type_box())
    }
}

/// Make accessor for map field
pub fn make_map_simpler_accessor<M, K, V>(
    name: &'static str,
    get_field: for<'a> fn(&'a M) -> &'a HashMap<K, V>,
    mut_field: for<'a> fn(&'a mut M) -> &'a mut HashMap<K, V>,
) -> FieldAccessor
where
    M: Message + 'static,
    K: ProtobufValueSized,
    V: ProtobufValueSized,
    K: Hash + Eq,
{
    FieldAccessor::new_v2(
        name,
        AccessorV2::Map(MapFieldAccessorHolder {
            accessor: Box::new(MapFieldAccessorImpl::<M, K, V> {
                get_field,
                mut_field,
            }),
        }),
    )
}
