use std::any::TypeId;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

/// This one will be more faithful to the original ts implementation.

type WrappedComp = Rc<RefCell<dyn std::any::Any>>;
type WrappedEntity<'a> = Rc<RefCell<Entity<'a>>>;

type Observer<C, T> = fn(caller: &C, data: &T);

struct Observable<C, T> {
    observers: Vec<Observer<C, T>>,
}

impl<C, T> Observable<C, T> {
    fn new() -> Self {
        Self { observers: Vec::new() }
    }

    fn register(&mut self, observer: Observer<C, T>) {
        self.observers.push(observer);
    }

    fn trigger(&self, caller: &C, data: T) {
        for observer in &self.observers {
            observer(caller, &data);
        }
    }

    // TODO we need to be able to clean this up wtf typescript how do you work
}

struct Scene<'a> {
    id: usize,
    entities: Vec<Rc<RefCell<Entity<'a>>>>,
    entity_added: Rc<Observable<Self, Rc<RefCell<Entity<'a>>>>>,
    entity_removed: Rc<Observable<Self, Rc<RefCell<Entity<'a>>>>>,
}

impl<'a> Scene<'a> {
    fn new() -> Self {
        Self { id: 0, entities: Vec::new(), entity_added: Rc::new(Observable::new()), entity_removed: Rc::new(Observable::new()) }
    }

    fn create_entity(&'a mut self) -> WrappedEntity {
        let mut child_create: Observable<Entity<'a>, WrappedEntity<'a>> = Observable::new();
        child_create.register(|parent, child| {
            let scene = parent.scene.borrow_mut();
            scene.entity_added.trigger(&scene, child.clone());
        });

        // let wrapped_entity = Rc::new(RefCell::new(Entity::new(self)));
        //
        // let local = wrapped_entity.clone();
        // let local = local.borrow_mut();
        // local.scene.clone().borrow_mut().entities.push(wrapped_entity.clone());
        // &self.entity_added.trigger(wrapped_entity.clone());

        // Propagate child events.

        // let entity_added_listener = self.entity_added.clone();
        // let entity_removed_listener = self.entity_removed.clone();
        // local.child_added.register(|x| { entity_added_listener.trigger(x.clone()) });
        // local.child_removed.register(|x| { entity_removed_listener.trigger(x.clone()) });
        let entity = Entity {
            id: 0,
            components: Vec::new(),
            component_added: Observable::new(),
            component_removed: Observable::new(),
            child_added: child_create,
            child_removed: Observable::new(),
            scene: RefCell::new(self),
        };

        return Rc::new(RefCell::new(entity));
    }
}

struct Entity<'a> {
    id: usize,
    components: Vec<WrappedComp>,
    component_added: Observable<Self, WrappedComp>,
    component_removed: Observable<Self, WrappedComp>,
    // parent: Option<Rc<Entity>>,
    child_added: Observable<Self, WrappedEntity<'a>>,
    child_removed: Observable<Self, WrappedEntity<'a>>,
    scene: RefCell<&'a Scene<'a>>,
}

// TODO do I need this?
trait Component {}

impl<'a> Entity<'a> {
    // TODO BAN creation of this, only allow it to be created via a scene.
    fn new(scene: &'a Scene<'a>) -> Self {
        Self {
            id: 0,
            components: Vec::new(),
            component_added: Observable::new(),
            component_removed: Observable::new(),
            child_added: Observable::new(),
            child_removed: Observable::new(),
            scene: RefCell::new(scene),
        }
    }

    fn add_component<T: 'static>(&mut self, component: T) {
        let wrapped_comp = Rc::new(RefCell::new(component));
        self.components.push(wrapped_comp.clone());
        self.component_added.trigger(&self, wrapped_comp.clone());
    }
}

struct System {
    types: Vec<TypeId>,
    func: fn(Rc<Entity>, u64),
}

impl System {}

#[cfg(test)]
mod test {
    use std::cell::{Cell, RefCell};

    use crate::ecs_v3::{Entity, Scene};

    struct A;


    #[test]
    fn create_entity() {
        let mut scene = Scene::new();

        let mut e = scene.create_entity();

        let mut e = e.borrow_mut();
        e.component_added.register(|e, x| { println!("OOF") });
        e.add_component(A);
    }

    struct B<'a> {
        parent: Option<RefCell<&'a B<'a>>>,
    }

    impl<'a> B<'a> {
        fn create_one(&'a self) -> Self {
            Self { parent: Some(RefCell::new(&self)) }
        }
    }

    #[test]
    fn test_inh() {
        let mut b = B { parent: None };
        b.create_one();
        b.create_one();
        let mut c = B { parent: Some(RefCell::new(&b)) };
    }
}