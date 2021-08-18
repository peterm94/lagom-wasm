use std::any::TypeId;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

/// This one will be more faithful to the original ts implementation.

type WrappedComp = Rc<RefCell<dyn std::any::Any>>;
type WrappedEntity = Rc<RefCell<Entity>>;

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

struct Scene {
    id: usize,
    entities: Vec<WrappedEntity>,

    entity_added: Rc<Observable<Self, WrappedEntity>>,
    entity_removed: Rc<Observable<Self, WrappedEntity>>,

    systems: Vec<System>,
}

// TODO Is there any way we can put this in scene? or does the wacky mutability break it?
fn create_entity(scene: &Rc<RefCell<Scene>>) -> Rc<RefCell<Entity>> {
    // Create hooks into the child object.
    let mut child_added: Observable<Entity, WrappedEntity> = Observable::new();
    child_added.register(|parent, child| {
        let scene = parent.scene.borrow_mut();
        scene.entity_added.trigger(&scene, child.clone());
    });
    let mut child_removed: Observable<Entity, WrappedEntity> = Observable::new();
    child_removed.register(|parent, child| {
        let scene = parent.scene.borrow_mut();
        scene.entity_removed.trigger(&scene, child.clone());
    });

    let entity = Rc::new(RefCell::new(Entity {
        id: 0,
        components: Vec::new(),
        component_added: Observable::new(),
        component_removed: Observable::new(),
        child_added,
        child_removed,
        children: Vec::new(),
        scene: scene.clone(),
    }));

    let mut scene = scene.borrow_mut();
    scene.entities.push(entity.clone());

    // Trigger the entity added to scene event.
    scene.entity_added.trigger(&*scene, entity.clone());

    entity
}

// TODO we could actually archetype systems?
//  if not, we at least need to propagate component create/remove events with their parent. lagom was wack
fn add_system(scene: &Rc<RefCell<Scene>>, system: System) {
    let scene = scene.borrow_mut();
}

impl Scene {
    fn new() -> Self {
        Self { id: 0, entities: Vec::new(), entity_added: Rc::new(Observable::new()), entity_removed: Rc::new(Observable::new()), systems: Vec::new() }
    }
}

struct Entity {
    id: usize,
    components: Vec<WrappedComp>,
    component_added: Observable<Self, WrappedComp>,
    component_removed: Observable<Self, WrappedComp>,
    // parent: Option<Rc<Entity>>,
    child_added: Observable<Self, WrappedEntity>,
    child_removed: Observable<Self, WrappedEntity>,
    children: Vec<WrappedEntity>,
    scene: Rc<RefCell<Scene>>,
}

// TODO do I need this?
trait Component {}

impl Entity {
    fn add_component<T: 'static>(&mut self, component: T) {
        let wrapped_comp = Rc::new(RefCell::new(component));
        self.components.push(wrapped_comp.clone());
        self.component_added.trigger(&self, wrapped_comp.clone());
    }

    fn create_child(&mut self) -> Rc<RefCell<Entity>> {
        let child = create_entity(&self.scene);
        self.children.push(child.clone());
        self.child_added.trigger(self, child.clone());

        return child;
    }
}

struct System {
    types: Vec<TypeId>,
    func: fn(WrappedEntity, u64),
}

impl System {}

#[cfg(test)]
mod test {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use crate::ecs_v3::{create_entity, Entity, Scene};

    struct A;


    #[test]
    fn create_entity_test() {
        let mut scene = Rc::new(RefCell::new(Scene::new()));

        let mut e = create_entity(&scene);

        e.borrow_mut().component_added.register(|e, x| { println!(":HELLO") });

        let mut e = create_entity(&scene);
        // let f = create_entity(&scene);
        // let g = create_entity(&scene);
        //
        // // let mut e = e.borrow_mut();
        // e.component_added.register(|e, x| { println!("OOF") });
        // e.add_component(A);
    }

    #[test]
    fn test_child() {
        let mut scene = Rc::new(RefCell::new(Scene::new()));

        let mut e = create_entity(&scene);

        let child = e.borrow_mut().create_child();
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