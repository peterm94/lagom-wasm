#[cfg(test)]
mod test {
    use hecs::*;

    trait JsType {
        fn get_id(&self) -> usize;
    }

    struct A {
        id: usize,
    }

    struct B {
        id: usize,
    }

    struct C {
        id: usize,
    }

    #[test]
    fn test_hecs() {
        let mut world = World::new();
        // world.entity()

        let a = world.spawn(());
    }
}