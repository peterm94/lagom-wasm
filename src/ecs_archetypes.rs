use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::fmt;

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
    // We aren't using a map to help with the eventual SIMD effort.
    components: Vec<ComponentArray>,
    entity_ids: Vec<EcsId>,

}

impl Debug for Archetype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Archetype")
            .field("type_vec", &self.type_vec)
            .field("add", &self.add)
            .field("components", &self.components)
            .field("entity_ids", &self.entity_ids)
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
    row: usize,
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

        // let components = vec![Box::new(RefCell::new((EntityId(entity_id))))];
        // self.add_component(&entity_id, EntityId(entity_id));

        return entity_id;
    }

    //TODO make this add_component, need to figure out the archetype traversal first
    fn add_entity(&mut self, entity_id: &EcsId, components: Vec<EcsId>) {

        // TODO sort the components
        // let mut components = components;
        // components.push(EntityId(entity_id));

        // Match the archetype.
        let mut archetype = self.get_archetype2(&components);

        // Update the entity index with a record so it can be found easily.
        let row = archetype.borrow().components.len();
        self.entity_index.insert(*entity_id, Record { archetype: archetype.clone(), row });

        // Insert the data into the archetype container.
        // TODO actually add the components.
        let mut archetype = archetype.borrow_mut();
        archetype.components.push(ComponentArray::new());
        archetype.entity_ids.push(*entity_id);
    }

    fn get_archetype2(&mut self, component_types: &TypeVec) -> Rc<RefCell<Archetype>> {
        let mut root = &self.archetypes;

        let mut new = root.clone();
        for comp_type in component_types {
            new = Archetype::with(new, *comp_type);
        }

        return new.clone();
    }

    // fn get_archetype<T: 'static>(&mut self, components: &Vec<T>) -> usize {
    //     let mut type_ids = Vec::with_capacity(components.len());
    //
    //     for component in components {
    //         let comp_type_id = component.type_id();
    //         let type_id = match self.type_id_map.get(&comp_type_id) {
    //             None => {
    //                 let idx = self.type_id_map.len();
    //                 self.type_id_map.insert(comp_type_id, idx);
    //                 idx
    //             }
    //             Some(idx) => {
    //                 *idx
    //             }
    //         };
    //
    //         type_ids.push(type_id);
    //     }
    //
    //     for (archetype_id, archetype) in self.archetypes.iter_mut().enumerate() {
    //         let type_vec = &archetype.type_vec;
    //
    //         if type_vec.len() != type_ids.len() { continue; }
    //         let mut same_type = true;
    //
    //         for types in &type_ids {
    //             if !archetype.type_vec.contains(types) {
    //                 same_type = false;
    //                 break;
    //             }
    //         }
    //
    //         // Found the right type, insert it into here.
    //         if same_type {
    //             return archetype_id;
    //         }
    //     }
    //
    //     // No match, new archetype.
    //     let archetype = Archetype::new(type_ids);
    //     self.archetypes.push(archetype);
    //     return self.archetypes.len() - 1;
    // }
}


#[cfg(test)]
mod test {
    use crate::ecs_archetypes::{EcsId, World};

    #[test]
    fn test_complicated() {
        let mut world = World::default();

        let component_type = 1;
        let entity_type = 2;

        let e: EcsId = 3;
        let c1: EcsId = 4;
        let c2: EcsId = 5;
        let c3: EcsId = 6;

        world.add_entity(&e, vec![c1, c2]);
        world.add_entity(&45, vec![c1, c2]);

        println!("{:#?}", &world);
    }

    #[test]
    fn test_has_comp() {
        let mut world = World::default();

        let e: EcsId = 1;
        let c1: EcsId = 2;

        world.add_entity(&e, vec![c1]);

        assert!(world.has_comp(e, c1));
    }

    #[test]
    fn test_no_comp() {
        let mut world = World::default();

        let e: EcsId = 1;
        let c1: EcsId = 2;
        let c2: EcsId = 3;

        world.add_entity(&e, vec![c1]);

        assert!(!world.has_comp(e, c2));
    }
}