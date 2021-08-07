use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::rc::Rc;

type EcsId = usize;

type TypeVec = Vec<EcsId>;

type ComponentArray = Vec<Box<dyn std::any::Any>>;


#[derive(Default)]
struct Archetype {
    // Vector of component type IDs that map to the component array positions.
    type_vec: TypeVec,

    // Archetype traversal.
    add: HashMap<EcsId, Rc<RefCell<Archetype>>>,
    remove: HashMap<EcsId, Rc<RefCell<Archetype>>>,

    // Archetype component storage.
    // Vector of component groups that match the type_vec type structure.
    // We aren't using a map to help with the eventual SIMD effort. -- maybe one day
    components: HashMap<EcsId, ComponentArray>,

}

impl Debug for Archetype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Archetype")
            .field("type_vec", &self.type_vec)
            .field("add", &self.add)
            .field("components", &self.components)
            .finish()
    }
}

impl Archetype {
    fn new(type_vec: TypeVec) -> Self {
        Self {
            type_vec,
            ..Default::default()
        }
    }

    fn with(archetype: Rc<RefCell<Archetype>>, type_id: EcsId) -> Rc<RefCell<Archetype>> {
        let mut original_archetype = archetype.borrow_mut();

        // Find the edge that adds the id.
        match original_archetype.add.get(&type_id) {
            None => {
                // Create the archetype.
                let mut new_types_vec = original_archetype.type_vec.clone();
                new_types_vec.push(type_id);
                let mut new_archetype = Rc::new(RefCell::new(Archetype::new(new_types_vec)));

                // Add the forward link
                original_archetype.add.insert(type_id, new_archetype.clone());

                // Add the backref.
                new_archetype.borrow_mut().remove.insert(type_id, archetype.clone());

                return new_archetype;
            }
            Some(archetype) => archetype.clone()
        }
    }

    // fn without(&mut self, type_id: EcsId) -> Archetype {}
}


//
// #[derive(Default, Debug)]
// struct ComponentArray {
//     elements: Vec<Box<dyn std::any::Any>>,
// }

#[derive(Debug)]
struct Record {
    archetype: Rc<RefCell<Archetype>>,
}

#[derive(Default, Debug)]
struct World {
    archetypes: Rc<RefCell<Archetype>>,
    entity_index: HashMap<EcsId, Record>,
    // type_id_map: HashMap<TypeId, usize>,
    entity_count: usize,
}

impl World {
    fn has_comp(&self, entity: EcsId, component: EcsId) -> bool {
        let record = self.entity_index.get(&entity).unwrap();
        return record.archetype.borrow().type_vec.contains(&component);
    }

    fn create_entity(&mut self) -> EcsId {
        let entity_id: EcsId = self.entity_count;
        self.entity_count += 1;

        self.entity_index.insert(entity_id, Record { archetype: self.archetypes.clone() });
        self.archetypes.borrow_mut().components.insert(entity_id, ComponentArray::new());

        return entity_id;
    }

    fn add_component(&mut self, entity_id: &EcsId, component: EcsId) {
        self.add_components(entity_id, vec![component]);
    }

    fn add_components(&mut self, entity_id: &EcsId, components: Vec<EcsId>) {
        let record = self.entity_index.get(entity_id).unwrap();

        // TODO insert the components
        let mut current_components = record.archetype.borrow_mut().components.remove(entity_id).unwrap();
        // current_components.extend(components.into_iter());
        let mut type_ids = record.archetype.borrow().type_vec.clone();
        type_ids.extend(components.into_iter());

        let destination_archetype = self.get_archetype(&type_ids);

        destination_archetype.borrow_mut().components.insert(*entity_id, current_components);

        // Update the record entry.
        self.entity_index.insert(*entity_id, Record { archetype: destination_archetype.clone() }).unwrap();
    }

    fn add_entity(&mut self, components: Vec<EcsId>) -> EcsId {
        let entity_id = self.create_entity();
        self.add_components(&entity_id, components);
        return entity_id;
    }

    fn get_archetype(&mut self, component_types: &TypeVec) -> Rc<RefCell<Archetype>> {
        let mut root = &self.archetypes;

        let mut new = root.clone();
        for comp_type in component_types {
            new = Archetype::with(new, *comp_type);
        }

        return new.clone();
    }
}


#[cfg(test)]
mod test {
    use crate::ecs_archetypes::{EcsId, World};

    #[test]
    fn test_complicated() {
        let mut world = World::default();

        let component_type = 1;
        let entity_type = 2;

        let c1: EcsId = 4;
        let c2: EcsId = 5;
        let c3: EcsId = 6;

        world.add_entity(vec![c1, c2]);
        world.add_entity(vec![c1, c2]);

        println!("{:#?}", &world);
    }

    #[test]
    fn test_has_comp() {
        let mut world = World::default();

        let c1: EcsId = 2;

        let e = world.add_entity(vec![c1]);

        assert!(world.has_comp(e, c1));
    }

    #[test]
    fn test_no_comp() {
        let mut world = World::default();

        let c1: EcsId = 2;
        let c2: EcsId = 3;

        let e = world.add_entity(vec![c1]);

        assert!(!world.has_comp(e, c2));
    }
}