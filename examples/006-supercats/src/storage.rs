macro_rules! storage {
    ($($name:ident($($param:ident),*)),* $(,)?) => {
        storage! {
            @internal
            counter: 0,
            items: [$($name($($param),*)),*]
        }
    };
    (@internal
        counter: $counter:expr,
        items: []
    ) => {};
    (@internal
        counter: $counter:expr,
        items: [$name:ident($($param:ident),*) $(, $($rest:tt)*)?]
    ) => {
        pub mod $name {
            pub(crate) use bobcat_sdk::storage::*;
            use bobcat_sdk::maths::{u, U};

            const SLOT: U = u!($counter);

            storage!(@impl [$($param),*]);
        }

        $(
            storage! {
                @internal
                counter: $counter + 1,
                items: [$($rest)*]
            }
        )?
    };
    (@impl []) => {
        pub fn get() -> U {
            storage_load(&SLOT)
        }

        pub fn set(x: impl Into<U>) {
            storage_store(&SLOT, &x.into())
        }
    };
    (@impl [$param1:ident]) => {
        pub fn get($param1: impl Into<U>) -> U {
            storage_load(&slot_map(&$param1.into(), &SLOT))
        }

        #[allow(unused)]
        pub fn set($param1: impl Into<U>, x: impl Into<U>) {
            storage_store(
                &slot_map(&$param1.into(), &SLOT),
                &x.into(),
            )
        }

        #[allow(unused)]
        pub fn decr($param1: impl Into<U>) {
            let k = slot_map(&$param1.into(), &SLOT);
            let v = storage_load(&k);
            storage_store(&k, &(v + U::ONE))
        }

        #[allow(unused)]
        pub fn incr($param1: impl Into<U>) {
            let k = slot_map(&$param1.into(), &SLOT);
            let v = storage_load(&k);
            storage_store(&k, &(v + U::ONE))
        }
    };
    (@impl [$param1:ident, $param2:ident]) => {
        pub fn get(
            $param1: impl Into<U>,
            $param2: impl Into<U>,
        ) -> U {
            storage_load(&slot_map(
                &$param1.into(),
                &slot_map(&$param2.into(), &SLOT),
            ))
        }

        pub fn set(
            $param1: impl Into<U>,
            $param2: impl Into<U>,
            x: impl Into<U>,
        ) {
            storage_store(
                &slot_map(
                    &$param1.into(),
                    &slot_map(&$param2.into(), &SLOT),
                ),
                &x.into(),
            )
        }
    };
}

storage! {
    balance(addr),
    owner_of(token_id),
    approval(token_id),
    approved_for_all(from, sender)
}
