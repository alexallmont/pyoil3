//! PyOil3: Rust lifetimes for PyO3
//!
//! A common problem for PyO3 is that it can't expose native rust classes that
//! have lifetimes because in a garbage-collected system there is no guarantee
//! on the destruction order of objects.
//! 
//! PyOil3 resolves this by using stripping the object lifetimes at compile time
//! and ensuring that they are valid at runtime by wrapping all types in an
//! Arc<Mutex> to ensure no two threads access the data at the same time.
//! 
//! Classes that have no lifetimes are registered with the `pyoil3_class!`,
//! given the public Python API name for the class, the type of the Rust class
//! it needs to store, and an 'interface handle' which is the internal type
//! (actually a module under the hood) that wraps up the internals in Rust.
//! 
//! Register lifetime classes with `pyoil3_ref_class!`. This takes a fourth
//! argument of the instance handle of the class that 'owns' this lifetime, i.e.
//! the one that this new class depends on. The type names are passed without
//! their lifetime parameters. Complex generic types may need type aliases so
//! the macro implementation can handle the `tt` type, which does not allow
//! generic parameters.
//! 
//! The implementation works by the `ref_class` cloning the Arc of the owning
//! object. The clone increases the reference count on the owning object so it
//! cannot be destroyed until the ref_class has released it; effectively, it is
//! using Arc shared pointers to build a dependency tree of reference counts.

/// Declare a PyOil3 class that has no lifetime dependencies, but which may be
/// depended upon by ref_classes.
#[macro_export]
macro_rules! pyoil3_class {
    (
        $PyApiName:expr,
        $RustType:tt,
        $InterfaceHandle:tt
    ) => {
        pub mod $InterfaceHandle {
            use std::sync::{
                Arc,
                Mutex,
            };
            use pyo3::prelude::*;

            pub struct RustInstance {
                pub instance: super::$RustType,
            }

            unsafe impl Send for RustInstance {}
            unsafe impl Sync for RustInstance {}

            pub type ArcHandle = Arc<Mutex<RustInstance>>;

            #[pyclass]
            #[pyo3(name = $PyApiName)]
            pub struct PyClass(pub ArcHandle);

            impl PyClass {
                pub fn bind_instance(
                    instance: super::$RustType
                ) -> PyClass {
                    PyClass(Arc::new(Mutex::new(
                        super::$InterfaceHandle::RustInstance {
                            instance,
                        }))
                    )
                }
            }
        }
    };
}

/// Declare a ref_class that may depend upon a PyOil3 owner.
#[macro_export]
macro_rules! pyoil3_ref_class {
    (
        $PyApiName:expr,
        $RustType:tt,
        $InterfaceHandle:tt,
        $OwnerHandle:tt
    ) => {
        pub mod $InterfaceHandle {
            use std::sync::{
                Arc,
                Mutex,
            };
            use pyo3::prelude::*;

            pub struct RustInstance {
                pub ref_static: super::$RustType<'static>,
                pub owner: Arc<Mutex<super::$OwnerHandle::RustInstance>>
            }
            unsafe impl Send for RustInstance {}
            unsafe impl Sync for RustInstance {}

            #[pyclass]
            #[pyo3(name = $PyApiName)]
            pub struct PyClass(pub Arc<Mutex<RustInstance>>);

            impl PyClass {
                pub fn bind_owned_instance<'a>(
                    reference: super::$RustType<'a>,
                    owner: Arc<Mutex<super::$OwnerHandle::RustInstance>>
                ) -> PyClass {
                    let ref_static: super::$RustType<'static> = unsafe {
                        std::mem::transmute(reference)
                    };

                    let inst = super::$InterfaceHandle::RustInstance {
                        ref_static,
                        owner
                    };

                    PyClass(Arc::new(Mutex::new(inst)))
                }
            }
        }
    };
}
