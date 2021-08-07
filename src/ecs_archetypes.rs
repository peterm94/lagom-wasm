use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::rc::Rc;

type EcsId = usize;

type TypeVec = Vec<EcsId>;

// type ComponentArray = Vec<Box<dyn std::any::Any>>;
type ComponentArray = Vec<Box<RefCell<dyn std::any::Any>>>;

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
    entity_index: HashMap<EcsId, TypeVec>,

    // This is to convert type_id to a usize
    type_id_map: HashMap<TypeId, usize>,
    ecs_id_count: usize,
}

// TODO entities need to go in the archetypes somehow
// TODO need to sort the component arrays, make sure it is the same order as the type map

impl World {
    fn has_comp<T: 'static>(&self, entity: EcsId) -> bool {
        if let Some(type_vec) = self.entity_index.get(&entity) {
            let type_id = TypeId::of::<T>();
            if let Some(type_id) = self.type_id_map.get(&type_id) {
                return type_vec.contains(type_id);
            }
        }
        return false;
    }

    fn create_entity(&mut self) -> EcsId {
        let entity_id: EcsId = self.ecs_id_count;
        self.ecs_id_count += 1;

        self.entity_index.insert(entity_id, TypeVec::new());
        self.archetypes.borrow_mut().components.insert(entity_id, ComponentArray::new());

        return entity_id;
    }

    fn find_archetype(&self, type_vec: &TypeVec) -> Rc<RefCell<Archetype>> {
        let root = self.archetypes.clone();

        let mut node = root;
        for ecs_type in type_vec {
            node = Archetype::with(node.clone(), *ecs_type)
        }
        return node;
    }

    fn get_type_vec(&self, entity_id: &EcsId) -> TypeVec {
        return self.entity_index.get(entity_id).unwrap().clone();
    }

    fn add_entity(&mut self, parent_entity: &EcsId, child_entity: &EcsId) {
        let type_vec = self.get_type_vec(parent_entity);
        let archetype = self.find_archetype(&type_vec).clone();

        // TODO...
    }

    fn add_component<T: 'static>(&mut self, entity_id: &EcsId, component: T) {
        let type_id = self.id_for_type(component.type_id());
        let type_vec = self.get_type_vec(entity_id);

        let archetype = self.find_archetype(&type_vec).clone();

        let mut updated_components = archetype.borrow_mut().components.remove(entity_id).unwrap();
        updated_components.push(Box::new(RefCell::new(component)));

        let destination_archetype = Archetype::with(archetype.clone(), type_id);
        destination_archetype.borrow_mut().components.insert(*entity_id, updated_components);

        // Update the record entry.
        self.entity_index.insert(*entity_id, destination_archetype.borrow().type_vec.clone()).unwrap();

        // Add the component definition ot the entity index.
        self.entity_index.insert(type_id, TypeVec::new());
    }

    fn id_for_type(&mut self, type_id: TypeId) -> usize {
        match self.type_id_map.get(&type_id) {
            None => {
                let next_id = self.ecs_id_count;
                self.ecs_id_count += 1;
                self.type_id_map.insert(type_id.clone(), next_id);
                // self.entity_index.insert(next_id, TypeVec::new());
                next_id
            }
            Some(id) => { *id }
        }
    }

    fn remove_component<T: 'static>(&mut self, entity_id: &EcsId) {
        let type_id = self.type_id_map.get(&TypeId::of::<T>()).unwrap();
        let type_vec = self.get_type_vec(entity_id);

        let archetype = self.find_archetype(&type_vec);

        // Get the index of the type in the component array.
        let type_index = archetype.borrow().type_vec.iter().position(|&v| v == *type_id).unwrap();
        let mut current_components = archetype.borrow_mut().components.remove(entity_id).unwrap();

        // Remove the component
        current_components.remove(type_index);

        // TODO optimize, we know the index of the one to remove.
        let destination_archetype = Archetype::without(archetype.clone(), *type_id);
        destination_archetype.borrow_mut().components.insert(*entity_id, current_components);

        // Update the record entry.
        self.entity_index.insert(*entity_id, destination_archetype.borrow().type_vec.clone()).unwrap();
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
    fn add_component() {
        let mut world = World::default();

        let entity = world.create_entity();
        world.add_component(&entity, A);
        world.add_component(&entity, B);
        // world.add_component(&entity, C);
        //
        // assert!(world.has_comp::<A>(entity));
        // assert!(world.has_comp::<B>(entity));
        // assert!(world.has_comp::<C>(entity));
        // assert!(!world.has_comp::<D>(entity));

        println!("{:#?}", &world);
    }

    #[test]
    fn nest_entities() {
        let mut world = World::default();

        let entity = world.create_entity();
        world.add_component(&entity, A);

        let entity2 = world.create_entity();
        world.add_component(&entity, B);
        world.add_entity(&entity, &entity2);

        println!("{:#?}", &world);
    }

    #[test]
    fn remove_component() {
        let mut world = World::default();

        let entity = world.create_entity();
        world.add_component(&entity, A);
        world.add_component(&entity, B);
        world.add_component(&entity, C);

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

        let entity = world.create_entity();
        world.add_component(&entity, A);
        world.add_component(&entity, A);

        println!("{:#?}", &world);
    }

    #[test]
    fn add_same_entity_type() {
        let mut world = World::default();

        let entity = world.create_entity();
        world.add_component(&entity, A);
        world.add_component(&entity, B);
        world.add_component(&entity, C);

        let entity2 = world.create_entity();
        world.add_component(&entity2, A);
        world.add_component(&entity2, B);
        world.add_component(&entity2, C);

        println!("{:#?}", &world);
    }
}