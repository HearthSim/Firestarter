//! Extension traits for [`Frunk`]s Heterogeneous List type.

use frunk::prelude::HList;
use frunk::{HCons, HNil};

use rpc::system::RPCService;

/// Visitor idiom which can be applied to a container.
///
/// This trait can be implemented to visit each value within the container
/// as a common ancestor.
///
/// # Note
/// The order in which the container items are visited is determined by the
/// implementation!
pub trait Visitor<'me, Abstraction>
where
    Abstraction: 'me + ?Sized,
{
    /// Visit each item within the container and execute the provided closure
    /// with a borrow of that item as argument.
    fn visit<F>(&'me self, closure: F)
    where
        F: Fn(&'me Abstraction) -> ();

    // fn visit_mut<F: Fn(&mut Abstraction) -> ()>(&mut self, closure: F);
}

impl<'me, Service> Visitor<'me, Service> for HNil
where
    Service: 'me + ?Sized,
{
    fn visit<F>(&'me self, closure: F)
    where
        F: Fn(&'me Service) -> (),
    {
        // Do nothing
    }
}

// https://users.rust-lang.org/t/bounds-issue-on-trait/19181
/// Visitor idiom which can be applied to a container.
///
/// This trait can be implemented to visit each value within the container
/// as a common ancestor. The provided closure can optionally store calculation
/// results through the collector argument.
///
/// # Note
/// The order in which the container items are visited is determined by the
/// implementation!
pub trait Gatherer<'me, Abstraction, Result>
where
    Abstraction: 'me + ?Sized,
    Result: 'me,
{
    // Into<Result> IS NOT USED because it erases the lifetimes during
    // transformation. The closure itself is responsible for properly
    // returning the correct type.

    // Gathering always happens from back to front!

    /*
    fn gather<F>(&'me self, collector: &'me mut [Option<Result>; <Self as HList>::LEN], closure: F)
    where
        F: for<'collect> Fn(&'me Abstraction, &'collect mut Option<Result>) -> ();
*/
    /// Visit each item within the container and execute the provided closure
    /// with a borrow of that item as argument.
    /// The second argument or the closure will be a mutable borrow of the Result value.
    /// You can overwrite this value from within the closure to pass it on to caller of
    /// the gather method.
    ///
    /// # Precondition
    /// The slice value of collector must be sufficiently large to store an element
    /// for each item that's being visited.
    unsafe fn gather<F>(&'me self, collector: &'me mut [Option<Result>], closure: F)
    where
        F: for<'collect> Fn(&'me Abstraction, &'collect mut Option<Result>) -> ();
}

impl<'me, Service, Result> Gatherer<'me, Service, Result> for HNil
where
    Service: 'me + ?Sized,
    Result: 'me,
{
    /*
    fn gather<F>(&'me self, collector: &'me mut [Option<Result>; Self::LEN], closure: F)
    where
        F: for<'collect> Fn(&'me Service, &'collect mut Option<Result>) -> (),
    {
        // Do nothing
    }
*/
    unsafe fn gather<F>(&'me self, collector: &'me mut [Option<Result>], closure: F)
    where
        F: for<'collect> Fn(&'me Service, &'collect mut Option<Result>) -> (),
    {
        // Do nothing
    }
}
