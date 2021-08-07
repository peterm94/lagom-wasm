use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::rc::Rc;

type EcsId = usize;

type TypeVec = Vec<EcsId>;

type ComponentArray = Vec<Box<dyn std::any::Any>>;
// type ComponentArray = Vec<Box<RefCell<dyn std::any::Any>>>;

fn comp1<T: 'static>(c1: T) -> ComponentArray {
    let mut tuple = ComponentArray::new();
    tuple.push(Box::new(RefCell::new(c1)));
    tuple
}

fn comp2<T: 'static, T2: 'static>(c1: T, c2: T2) -> ComponentArray {
    let mut tuple = ComponentArray::new();
    tuple.push(Box::new(RefCell::new(c1)));
    tuple.push(Box::new(RefCell::new(c2)));
    tuple
}

fn comp3<T: 'static, T2: 'static, T3: 'static>(c1: T, c2: T2, c3: T3) -> ComponentArray {
    let mut tuple = ComponentArray::new();
    tuple.push(Box::new(RefCell::new(c1)));
    tuple.push(Box::new(RefCell::new(c2)));
    tuple.push(Box::new(RefCell::new(c3)));
    tuple
}

#[derive(Default)]
struct Archetype {
    // Vector of component type IDs that map to the component array positions.
    type_vec: TypeVec,

    // Archetype traversal.
    add: HashMap<EcsId, Rc<RefCell<Archetype>>>,
    remove: HashMap<EcsId, Rc<RefCell<Archetype>>>,

    // Archetype component storage.
    // Entities that have components that match the type_vec type structure.
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

    fn without(archetype: Rc<RefCell<Archetype>>, type_id: EcsId) -> Rc<RefCell<Archetype>> {
        let mut original_archetype = archetype.borrow_mut();

        // Find the edge that adds the id.
        match original_archetype.remove.get(&type_id) {
            None => {
                // Create the archetype.
                let mut new_types_vec = original_archetype.type_vec.clone();
                let new_types_vec = new_types_vec.into_iter().filter(|x| *x != type_id).collect::<Vec<_>>();
                let new_archetype = Rc::new(RefCell::new(Archetype::new(new_types_vec)));

                // Add the backwards link
                original_archetype.remove.insert(type_id, new_archetype.clone());

                // Add the forward link.
                new_archetype.borrow_mut().add.insert(type_id, archetype.clone());

                return new_archetype;
            }
            Some(archetype) => archetype.clone()
        }
    }
}

#[derive(Debug)]
struct Record {
    archetype: Rc<RefCell<Archetype>>,
}

#[derive(Default, Debug)]
struct World {
    archetypes: Rc<RefCell<Archetype>>,
    entity_index: HashMap<EcsId, Record>,
    entity_count: usize,
    type_id_map: HashMap<TypeId, usize>,
    type_id_count: usize,
}

impl World {
    fn has_comp<T: 'static>(&self, entity: EcsId) -> bool {
        let record = self.entity_index.get(&entity).unwrap();
        let type_id = TypeId::of::<T>();
        return record.archetype.borrow().type_vec.contains(self.type_id_map.get(&type_id).unwrap());
    }

    fn create_entity(&mut self) -> EcsId {
        let entity_id: EcsId = self.entity_count;
        self.entity_count += 1;

        self.entity_index.insert(entity_id, Record { archetype: self.archetypes.clone() });
        self.archetypes.borrow_mut().components.insert(entity_id, ComponentArray::new());

        return entity_id;
    }

    fn add_component<T: 'static>(&mut self, entity_id: &EcsId, component: T) {
        self.add_components(entity_id, comp1(component));
    }

    fn type_ids(&mut self, components: &ComponentArray) -> TypeVec {
        components.iter().map(|x| {
            let type_id = &x.type_id();
            match self.type_id_map.get(type_id) {
                None => {
                    let next_id = self.type_id_count;
                    self.type_id_count += 1;
                    self.type_id_map.insert(type_id.clone(), next_id);
                    next_id
                }
                Some(id) => { *id }
            }
        }).collect::<Vec<_>>()
    }

    fn add_components(&mut self, entity_id: &EcsId, components: ComponentArray) {
        let type_ids = self.type_ids(&components);
        let record = self.entity_index.get(entity_id).unwrap();

        let mut updated_components = record.archetype.borrow_mut().components.remove(entity_id).unwrap();
        updated_components.extend(components.into_iter());

        let mut update_type_ids = record.archetype.borrow().type_vec.clone();
        update_type_ids.extend(type_ids.into_iter());

        let destination_archetype = self.get_archetype(record.archetype.clone(), &update_type_ids);

        destination_archetype.borrow_mut().components.insert(*entity_id, updated_components);

        // Update the record entry.
        self.entity_index.insert(*entity_id, Record { archetype: destination_archetype.clone() }).unwrap();
    }

    fn remove_component<T: 'static>(&mut self, entity_id: &EcsId) {
        let record = self.entity_index.get(&entity_id).unwrap();
        let type_id = self.type_id_map.get(&TypeId::of::<T>()).unwrap();

        // Get the index of the type in the component array.
        let type_index = record.archetype.borrow().type_vec.iter().position(|&v| v == *type_id).unwrap();
        let mut current_components = record.archetype.borrow_mut().components.remove(entity_id).unwrap();

        // Remove the component
        current_components.remove(type_index);

        // TODO optimize, we know the index of the one to remove.
        let destination_archetype = Archetype::without(record.archetype.clone(), *type_id);
        destination_archetype.borrow_mut().components.insert(*entity_id, current_components);

        // Update the record entry.
        self.entity_index.insert(*entity_id, Record { archetype: destination_archetype.clone() }).unwrap();
    }

    fn add_entity(&mut self, components: ComponentArray) -> EcsId {
        let entity_id = self.create_entity();
        self.add_components(&entity_id, components);
        return entity_id;
    }

    fn get_archetype(&mut self, current_archetype: Rc<RefCell<Archetype>>,
                     component_types: &TypeVec) -> Rc<RefCell<Archetype>> {
        let mut new = current_archetype.clone();
        for comp_type in component_types {
            new = Archetype::with(new, *comp_type);
        }

        return new.clone();
    }
}


#[cfg(test)]
mod test {
    use crate::ecs_archetypes::{comp1, comp2, comp3, EcsId, World};

    struct A;

    struct B;

    struct C;

    struct D;

    #[test]
    fn create_entity() {
        let mut world = World::default();

        let entity = world.add_entity(comp3(A, B, C));

        assert!(world.has_comp::<A>(entity));
        assert!(world.has_comp::<B>(entity));
        assert!(world.has_comp::<C>(entity));
        assert!(!world.has_comp::<D>(entity));
    }

    #[test]
    fn add_component() {
        let mut world = World::default();

        let entity = world.add_entity(comp1(A));

        world.add_component(&entity, D);

        assert!(world.has_comp::<A>(entity));
        assert!(world.has_comp::<D>(entity));
    }

    #[test]
    fn add_component_batch() {
        let mut world = World::default();

        let entity = world.add_entity(comp1(B));

        world.add_components(&entity, comp3(A, C, D));

        assert!(world.has_comp::<A>(entity));
        assert!(world.has_comp::<C>(entity));
        assert!(world.has_comp::<D>(entity));
    }

    #[test]
    fn remove_component() {
        let mut world = World::default();

        let entity = world.add_entity(comp3(A, B, C));
        assert!(world.has_comp::<A>(entity));
        assert!(world.has_comp::<B>(entity));
        assert!(world.has_comp::<C>(entity));

        world.remove_component::<B>(&entity);
        assert!(world.has_comp::<A>(entity));
        assert!(!world.has_comp::<B>(entity));
        assert!(world.has_comp::<C>(entity));
    }

    // TODO Make this impossible
    #[test]
    fn add_same_component_to_entity() {
        let mut world = World::default();

        world.add_entity(comp2(A, A));

        println!("{:#?}", &world);
    }

    #[test]
    fn add_same_entity_type() {
        let mut world = World::default();

        world.add_entity(comp3(A, B, C));
        world.add_entity(comp3(A, B, C));

        println!("{:#?}", &world);
    }


    #[test]
    fn test_complicated() {
        let mut world = World::default();

        let component_type = 1;
        let entity_type = 2;

        let c1: EcsId = 4;
        let c2: EcsId = 5;
        let c3: EcsId = 6;

        world.add_entity(comp2(A, B));
        world.add_entity(comp2(A, B));

        println!("{:#?}", &world);
    }
}