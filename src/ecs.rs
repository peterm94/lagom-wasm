use std::any::TypeId;
use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::DerefMut;

// Entity ID, Component Type, Component
// struct ComponentSlice(usize, usize, Box<dyn std::any::Any>);
struct ComponentSlice(usize, TypeId, Box<dyn std::any::Any>);


trait ComponentType {
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> ComponentType for RefCell<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }


    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

trait Filter
{
    fn matches(&self, slice: &ComponentSlice) -> bool;
}

struct HasComp {
    type_id: TypeId,
}

// TODO we can do the downcast thing instead of TypeId?
impl HasComp {
    fn new<T: 'static>() -> Self {
        return HasComp { type_id: TypeId::of::<T>() };
    }
}

impl Filter for HasComp {
    fn matches(&self, slice: &ComponentSlice) -> bool {
        return self.type_id == slice.1;
    }
}

#[derive(Default)]
struct Game {
    components: Vec<ComponentSlice>,
    entity_count: usize,
    // unique_component_count: usize,
    // component_types: Vec<Box<dyn ComponentType>>,
}

impl Game {
    fn create_entity(&mut self) -> usize {
        let entity_id = self.entity_count;
        self.entity_count += 1;
        entity_id
    }

    fn add_entity<T: EntityCreator>(&mut self, creator: &T) {
        creator.create_entity(self);
    }

    fn add_component<T: 'static>(&mut self, entity: usize, component: T) {
        self.components.push(ComponentSlice(entity, TypeId::of::<T>(), Box::new(RefCell::new(component))));
    }

    fn get_component<T: 'static>(&mut self, entity: usize) -> Option<RefMut<T>> {
        // TODO we can do the downcast thing here instead of TypeId?
        match self.components.iter_mut().find(|x| x.0 == entity && x.1 == TypeId::of::<T>()) {
            None => { None }
            Some(ComponentSlice(_, _, comp)) => {
                let mut a = comp.downcast_ref::<RefCell<T>>();
                let a = a.unwrap();
                return Some(a.borrow_mut());
                // return Some(a.borrow_mut().deref_mut());
                println!("AAA");
                None
            }
        }
    }

    fn get_entities_with_filter(&mut self, filters: &[&dyn Filter]) -> Vec<usize> {
        let mut matches = Vec::new();

        for entity_id in 0..self.entity_count {
            let mut entity_match = true;

            for filter in filters {
                let mut filter_matches = false;

                // Each filter has to match once.
                for component in self.components.iter().filter(|x| x.0 == entity_id) {
                    if filter.matches(component) {
                        filter_matches = true;
                        break;
                    }
                }

                // A filter does not match, exit the filter checker.
                if !filter_matches {
                    entity_match = false;
                    break;
                }
            }

            if entity_match {
                matches.push(entity_id);
            }
        }

        return matches;
    }
}

trait Updatable {
    fn update(delta: u16);
}

trait EntityCreator {
    fn create_entity(&self, game: &mut Game) -> usize;
}

struct TextRenderer {}

struct TextBox {}

struct TextValue(String);

impl EntityCreator for TextBox {
    fn create_entity(&self, game: &mut Game) -> usize {
        let entity_id = game.create_entity();

        game.add_component(entity_id, TextRenderer {});
        game.add_component(entity_id, TextValue("HAHAHA".to_string()));

        entity_id
    }
}

struct Parent(usize);

struct Position(f64, f64);

#[cfg(test)]
mod test {
    use std::any::TypeId;
    use std::cell::{RefCell, RefMut};

    use crate::ecs::{ComponentSlice, Game, HasComp, TextBox};

    struct TestComp(u32);

    struct TestComp2;

    #[test]
    fn test_hello() {
        let mut game = Game::default();
        let e1 = game.create_entity();
        let e2 = game.create_entity();

        game.add_component(e1, TestComp(1235));
        game.add_component(e1, TestComp(1555));
        game.add_component(e1, TestComp2 {});

        game.add_component(e2, TestComp2 {});
        // game.add_component(e1, TestComp {});

        // // Get a component slice out for an entity
        // let desired_type = TypeId::of::<TestComp>();
        // let iter = game.components.iter_mut().filter_map(|comp| {
        //     if comp.1 == desired_type {
        //         let x = comp.2.downcast_ref::<RefCell<desiredType>>();
        //     }
        //     return None;
        // });
        //
        // for comp in iter {
        //     let cast_comp = comp.as_mut();
        //     println!("{}", "aa");
        // }

        let test_comp_filter = HasComp::new::<TestComp>();

        let matches = game.get_entities_with_filter(&[&test_comp_filter]);

        println!("{:?}", &matches);

        game.get_component::<TestComp>(0);

        game.add_entity(&TextBox {});

        let a: Option<RefMut<TestComp>> = game.get_component(0);

        let mut a = a.unwrap();
        a.0 = 4;

        println!("{}", a.0);
    }

// use crate::ecs::{Component, ComponentId, Game};
//
// struct TestComp;
//
// impl Component for TestComp {}
//
// #[test]
// fn test_hello() {
//     let comp1 = TestComp;
//     let comp2 = TestComp;
//
//     let mut game = Game::default();
//     let e = game.create_entity();
//     e.with(comp1);
// }
}